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

//! Per-transfer distributed lease (`dataplane:kafka:lease:{dataplane_id}`) ensuring at most one
//! horizontally scaled dataplane replica drives a `KafkaRelayWorker` for a given transfer at a
//! time, with ≤15 s crash failover and an in-memory fallback for unit tests.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use uuid::Uuid;

/// Default lease TTL in milliseconds (`15 s`). Renewed every `ttl_ms / 3` (`5 s`).
pub const DEFAULT_KAFKA_LEASE_TTL_MS: u64 = 15_000;

struct MemoryLease {
    owner: String,
    expires_at: Instant,
}

/// Coordinates exclusive per-transfer ownership across dataplane replicas.
pub struct KafkaLeaseManager {
    replica_id: String,
    ttl_ms: u64,
    redis: Option<redis::aio::MultiplexedConnection>,
    memory: Mutex<HashMap<String, MemoryLease>>,
}

impl Default for KafkaLeaseManager {
    fn default() -> Self {
        Self::in_memory()
    }
}

impl KafkaLeaseManager {
    /// Pure in-memory lease manager for unit tests without Redis.
    pub fn in_memory() -> Self {
        Self {
            replica_id: Uuid::new_v4().to_string(),
            ttl_ms: DEFAULT_KAFKA_LEASE_TTL_MS,
            redis: None,
            memory: Mutex::new(HashMap::new()),
        }
    }

    /// Redis-backed lease manager safe across horizontally scaled dataplane replicas.
    pub fn with_redis(conn: redis::aio::MultiplexedConnection) -> Self {
        Self {
            replica_id: Uuid::new_v4().to_string(),
            ttl_ms: DEFAULT_KAFKA_LEASE_TTL_MS,
            redis: Some(conn),
            memory: Mutex::new(HashMap::new()),
        }
    }

    /// Overrides the replica identifier (useful for deterministic multi-replica failover tests).
    pub fn with_replica_id(mut self, replica_id: impl Into<String>) -> Self {
        self.replica_id = replica_id.into();
        self
    }

    /// Overrides the lease TTL in milliseconds (useful for sub-second failover tests).
    pub fn with_ttl_ms(mut self, ttl_ms: u64) -> Self {
        self.ttl_ms = ttl_ms.max(60);
        self
    }

    /// Unique identifier of this dataplane replica.
    pub fn replica_id(&self) -> &str {
        &self.replica_id
    }

    /// Heartbeat cadence (`ttl / 3`, floored at `20 ms`).
    pub fn renew_interval(&self) -> Duration {
        Duration::from_millis((self.ttl_ms / 3).max(20))
    }

    fn lease_key(dataplane_id: &str) -> String {
        format!("dataplane:kafka:lease:{dataplane_id}")
    }

    /// Atomically acquires the lease for `dataplane_id` (or refreshes it if already held by this
    /// replica). Returns `false` if another live replica holds it.
    pub async fn acquire(&self, dataplane_id: &str) -> bool {
        if let Some(mut conn) = self.redis.clone() {
            let key = Self::lease_key(dataplane_id);
            let script = redis::Script::new(
                r#"
                local cur = redis.call('GET', KEYS[1])
                if (not cur) or (cur == ARGV[1]) then
                    redis.call('SET', KEYS[1], ARGV[1], 'PX', ARGV[2])
                    return 1
                end
                return 0
                "#,
            );
            match script
                .key(&key)
                .arg(&self.replica_id)
                .arg(self.ttl_ms)
                .invoke_async::<i32>(&mut conn)
                .await
            {
                Ok(v) => return v == 1,
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        dataplane_id,
                        "redis kafka lease acquire failed; falling back to local memory"
                    );
                }
            }
        }

        let now = Instant::now();
        let mut guard = self.memory.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(existing) = guard.get(dataplane_id) {
            if existing.expires_at > now && existing.owner != self.replica_id {
                return false;
            }
        }
        guard.insert(
            dataplane_id.to_string(),
            MemoryLease {
                owner: self.replica_id.clone(),
                expires_at: now + Duration::from_millis(self.ttl_ms),
            },
        );
        true
    }

    /// Extends the lease TTL if and only if this replica is still its owner.
    pub async fn renew(&self, dataplane_id: &str) -> bool {
        if let Some(mut conn) = self.redis.clone() {
            let key = Self::lease_key(dataplane_id);
            let script = redis::Script::new(
                r#"
                if redis.call('GET', KEYS[1]) == ARGV[1] then
                    return redis.call('PEXPIRE', KEYS[1], ARGV[2])
                end
                return 0
                "#,
            );
            match script
                .key(&key)
                .arg(&self.replica_id)
                .arg(self.ttl_ms)
                .invoke_async::<i32>(&mut conn)
                .await
            {
                Ok(v) => return v == 1,
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        dataplane_id,
                        "redis kafka lease renew failed; falling back to local memory"
                    );
                }
            }
        }

        let now = Instant::now();
        let mut guard = self.memory.lock().unwrap_or_else(|e| e.into_inner());
        match guard.get_mut(dataplane_id) {
            Some(entry) if entry.owner == self.replica_id && entry.expires_at > now => {
                entry.expires_at = now + Duration::from_millis(self.ttl_ms);
                true
            }
            _ => false,
        }
    }

    /// Releases the lease if owned by this replica (`Suspend` / `Terminate` / `Complete`).
    pub async fn release(&self, dataplane_id: &str) {
        if let Some(mut conn) = self.redis.clone() {
            let key = Self::lease_key(dataplane_id);
            let script = redis::Script::new(
                r#"
                if redis.call('GET', KEYS[1]) == ARGV[1] then
                    return redis.call('DEL', KEYS[1])
                end
                return 0
                "#,
            );
            if script
                .key(&key)
                .arg(&self.replica_id)
                .invoke_async::<i32>(&mut conn)
                .await
                .is_ok()
            {
                return;
            }
        }

        let mut guard = self.memory.lock().unwrap_or_else(|e| e.into_inner());
        if guard
            .get(dataplane_id)
            .map(|e| e.owner == self.replica_id)
            .unwrap_or(false)
        {
            guard.remove(dataplane_id);
        }
    }
}
