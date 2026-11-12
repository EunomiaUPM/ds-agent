/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

//! Atomic single-shot quota reservation (`FINITE`) and consecutive transient-failure tracking
//! (`maxRetries`) across dataplane proxy hops.
//!
//! Uses Redis (`MultiplexedConnection`) in production so horizontal replicas share quotas and
//! retry counters, with an automatic in-memory fallback for unit tests (or transient Redis blips).

use std::collections::HashMap;
use std::sync::Mutex;

use redis::AsyncCommands;

const FINITE_SLOT_TTL_SECS: u64 = 86_400;
const FAILURE_COUNTER_TTL_SECS: i64 = 3_600;

/// Outcome of attempting to claim the single-shot execution slot of a `FINITE` transfer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReserveOutcome {
    /// This request owns the single-shot slot and may proceed upstream.
    Reserved,
    /// Another request is already in flight or has already completed the `FINITE` transfer.
    Exhausted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SlotState {
    InFlight,
    Done,
}

#[derive(Default)]
struct MemoryState {
    slots: HashMap<String, SlotState>,
    failures: HashMap<String, u32>,
}

/// Tracks single-shot `FINITE` reservations and consecutive transient failures per dataplane id.
pub struct DeliveryTracker {
    redis: Option<redis::aio::MultiplexedConnection>,
    memory: Mutex<MemoryState>,
}

impl Default for DeliveryTracker {
    fn default() -> Self {
        Self::in_memory()
    }
}

impl DeliveryTracker {
    /// Pure in-memory tracker for unit tests without Redis.
    pub fn in_memory() -> Self {
        Self {
            redis: None,
            memory: Mutex::new(MemoryState::default()),
        }
    }

    /// Redis-backed tracker safe across horizontally scaled dataplane instances.
    pub fn with_redis(conn: redis::aio::MultiplexedConnection) -> Self {
        Self {
            redis: Some(conn),
            memory: Mutex::new(MemoryState::default()),
        }
    }

    /// Synchronous fast path for `complete_finite_slot` when running without Redis.
    pub fn try_complete_finite_slot_sync(&self, dataplane_id: &str) -> Option<bool> {
        if self.redis.is_some() {
            return None;
        }
        let mut guard = self.memory.lock().unwrap_or_else(|e| e.into_inner());
        if guard.slots.get(dataplane_id) == Some(&SlotState::Done) {
            Some(false)
        } else {
            guard
                .slots
                .insert(dataplane_id.to_string(), SlotState::Done);
            Some(true)
        }
    }

    /// Synchronous fast path for `release_finite_slot` when running without Redis.
    pub fn try_release_finite_slot_sync(&self, dataplane_id: &str) -> bool {
        if self.redis.is_some() {
            return false;
        }
        let mut guard = self.memory.lock().unwrap_or_else(|e| e.into_inner());
        if guard.slots.get(dataplane_id) == Some(&SlotState::InFlight) {
            guard.slots.remove(dataplane_id);
        }
        true
    }

    /// Synchronous fast path for `record_transient_failure` when running without Redis.
    pub fn try_record_transient_failure_sync(
        &self,
        dataplane_id: &str,
        max_retries: u32,
    ) -> Option<bool> {
        if self.redis.is_some() {
            return None;
        }
        let mut guard = self.memory.lock().unwrap_or_else(|e| e.into_inner());
        let entry = guard
            .failures
            .entry(dataplane_id.to_string())
            .or_insert(0);
        *entry = entry.saturating_add(1);
        Some(*entry > max_retries)
    }

    fn slot_key(dataplane_id: &str) -> String {
        format!("dp:finite:slot:{dataplane_id}")
    }

    fn fail_key(dataplane_id: &str) -> String {
        format!("dp:fail:count:{dataplane_id}")
    }

    /// Atomically claims the single-shot slot for `dataplane_id`.
    pub async fn reserve_finite_slot(&self, dataplane_id: &str) -> ReserveOutcome {
        if let Some(mut conn) = self.redis.clone() {
            let key = Self::slot_key(dataplane_id);
            let res: redis::RedisResult<Option<String>> = redis::cmd("SET")
                .arg(&key)
                .arg("inflight")
                .arg("NX")
                .arg("EX")
                .arg(FINITE_SLOT_TTL_SECS)
                .query_async(&mut conn)
                .await;
            match res {
                Ok(Some(_)) => return ReserveOutcome::Reserved,
                Ok(None) => return ReserveOutcome::Exhausted,
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        dataplane_id,
                        "redis reserve_finite_slot failed; falling back to local memory"
                    );
                }
            }
        }

        let mut guard = self.memory.lock().unwrap_or_else(|e| e.into_inner());
        if guard.slots.contains_key(dataplane_id) {
            ReserveOutcome::Exhausted
        } else {
            guard
                .slots
                .insert(dataplane_id.to_string(), SlotState::InFlight);
            ReserveOutcome::Reserved
        }
    }

    /// Releases an in-flight `FINITE` slot when a retryable failure happens and retries remain,
    /// or when a client disconnects before EOF. Never reopens a slot already marked `Done`.
    pub async fn release_finite_slot(&self, dataplane_id: &str) {
        if let Some(mut conn) = self.redis.clone() {
            let key = Self::slot_key(dataplane_id);
            let script = redis::Script::new(
                r#"
                if redis.call('GET', KEYS[1]) == 'inflight' then
                    return redis.call('DEL', KEYS[1])
                end
                return 0
                "#,
            );
            if script.key(&key).invoke_async::<i32>(&mut conn).await.is_ok() {
                return;
            }
        }

        let mut guard = self.memory.lock().unwrap_or_else(|e| e.into_inner());
        if guard.slots.get(dataplane_id) == Some(&SlotState::InFlight) {
            guard.slots.remove(dataplane_id);
        }
    }

    /// Marks the `FINITE` slot permanently `Done`. Returns `true` exactly once so callers emit
    /// `DataplaneSignal::Completed` at most once per transfer.
    pub async fn complete_finite_slot(&self, dataplane_id: &str) -> bool {
        if let Some(mut conn) = self.redis.clone() {
            let key = Self::slot_key(dataplane_id);
            let script = redis::Script::new(
                r#"
                if redis.call('GET', KEYS[1]) == 'done' then
                    return 0
                end
                redis.call('SET', KEYS[1], 'done', 'EX', ARGV[1])
                return 1
                "#,
            );
            match script
                .key(&key)
                .arg(FINITE_SLOT_TTL_SECS)
                .invoke_async::<i32>(&mut conn)
                .await
            {
                Ok(v) => return v == 1,
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        dataplane_id,
                        "redis complete_finite_slot failed; falling back to local memory"
                    );
                }
            }
        }

        let mut guard = self.memory.lock().unwrap_or_else(|e| e.into_inner());
        if guard.slots.get(dataplane_id) == Some(&SlotState::Done) {
            false
        } else {
            guard
                .slots
                .insert(dataplane_id.to_string(), SlotState::Done);
            true
        }
    }

    /// Resets consecutive transient failures after a healthy (`2xx`) upstream exchange.
    pub async fn record_success(&self, dataplane_id: &str) {
        if let Some(mut conn) = self.redis.clone() {
            let key = Self::fail_key(dataplane_id);
            let _: redis::RedisResult<()> = conn.del(&key).await;
        }
        let mut guard = self.memory.lock().unwrap_or_else(|e| e.into_inner());
        guard.failures.remove(dataplane_id);
    }

    /// Increments consecutive transient failures for `dataplane_id` and returns `true` once the
    /// retry budget `max_retries` has been exceeded (`consecutive_failures > max_retries`).
    pub async fn record_transient_failure(&self, dataplane_id: &str, max_retries: u32) -> bool {
        if let Some(mut conn) = self.redis.clone() {
            let key = Self::fail_key(dataplane_id);
            let script = redis::Script::new(
                r#"
                local n = redis.call('INCR', KEYS[1])
                if n == 1 then
                    redis.call('EXPIRE', KEYS[1], ARGV[1])
                end
                return n
                "#,
            );
            match script
                .key(&key)
                .arg(FAILURE_COUNTER_TTL_SECS)
                .invoke_async::<u64>(&mut conn)
                .await
            {
                Ok(count) => return count > u64::from(max_retries),
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        dataplane_id,
                        "redis record_transient_failure failed; falling back to local memory"
                    );
                }
            }
        }

        let mut guard = self.memory.lock().unwrap_or_else(|e| e.into_inner());
        let entry = guard
            .failures
            .entry(dataplane_id.to_string())
            .or_insert(0);
        *entry = entry.saturating_add(1);
        *entry > max_retries
    }
}
