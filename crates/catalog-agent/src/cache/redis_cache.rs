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

//! The cache in Redis, as plain strings: no Redis module is needed.

use std::collections::HashMap;
use std::sync::RwLock;

use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use serde_json::Value;
use tracing::warn;
use ymir::errors::{Errors, Outcome};

use crate::cache::{CatalogCacheTrait, ProjectionKey};

#[derive(Default)]
struct L1ProjectionState {
    generation: u64,
    projections: HashMap<String, Value>,
}

pub struct RedisCatalogCache {
    connection: MultiplexedConnection,
    l1: RwLock<L1ProjectionState>,
}

impl RedisCatalogCache {
    const NAMESPACE: &'static str = "ds_agent_catalog";
    const MAX_L1_ENTRIES: usize = 512;
    /// Projections are rebuilt at most once an hour even with no write.
    const PROJECTION_TTL: u64 = 3600;
    /// What a peer published is kept a day.
    const PEER_CATALOG_TTL: u64 = 86400;

    pub fn new(connection: MultiplexedConnection) -> Self {
        Self {
            connection,
            l1: RwLock::new(L1ProjectionState::default()),
        }
    }

    fn generation_key() -> String {
        format!("{}:projections:generation", Self::NAMESPACE)
    }

    fn projection_key(key: &str) -> String {
        format!("{}:projections:{key}", Self::NAMESPACE)
    }

    fn peer_key(user_id: &str, peer_id: &str) -> String {
        format!("{}:peer-catalogs:{user_id}:{peer_id}", Self::NAMESPACE)
    }

    fn failed(e: redis::RedisError) -> Errors {
        Errors::crazy("the catalog cache cannot be reached", Some(Box::new(e)))
    }

    fn read_l1(&self, key: &str) -> Option<Value> {
        let guard = self.l1.read().ok()?;
        guard.projections.get(key).cloned()
    }

    fn known_l1_generation(&self, profile: &str, root: &str, reader: &str) -> Option<(u64, String)> {
        let guard = self.l1.read().ok()?;
        let key = ProjectionKey::of(profile, root, reader, guard.generation);
        guard
            .projections
            .contains_key(&key)
            .then_some((guard.generation, key))
    }

    fn sync_l1_generation(&self, gen: u64) {
        if let Ok(mut guard) = self.l1.write() {
            if guard.generation != gen {
                guard.generation = gen;
                guard.projections.clear();
            }
        }
    }

    fn write_l1(&self, gen: Option<u64>, key: String, value: &Value) {
        if let Ok(mut guard) = self.l1.write() {
            if let Some(g) = gen {
                if g < guard.generation {
                    return;
                }
                if g > guard.generation {
                    guard.generation = g;
                    guard.projections.clear();
                }
            }
            if guard.projections.len() >= Self::MAX_L1_ENTRIES {
                guard.projections.clear();
            }
            guard.projections.insert(key, value.clone());
        }
    }

    fn clear_l1(&self) {
        if let Ok(mut guard) = self.l1.write() {
            guard.generation = guard.generation.saturating_add(1);
            guard.projections.clear();
        }
    }
}

#[async_trait::async_trait]
impl CatalogCacheTrait for RedisCatalogCache {
    async fn generation(&self) -> u64 {
        let mut conn = self.connection.clone();
        conn.get::<_, Option<u64>>(Self::generation_key())
            .await
            .inspect_err(|e| warn!(error = %e, "catalog cache: generation unreadable"))
            .ok()
            .flatten()
            .unwrap_or(0)
    }

    async fn invalidate(&self) {
        self.clear_l1();
        let mut conn = self.connection.clone();
        if let Err(e) = conn.incr::<_, _, u64>(Self::generation_key(), 1).await {
            warn!(error = %e, "catalog cache: generation not bumped");
        }
    }

    async fn get_projection(&self, key: &str) -> Option<Value> {
        if let Some(cached) = self.read_l1(key) {
            return Some(cached);
        }
        let mut conn = self.connection.clone();
        let cached: Option<String> = conn.get(Self::projection_key(key)).await.ok().flatten();
        let parsed = cached.and_then(|json| serde_json::from_str(&json).ok());
        if let Some(value) = &parsed {
            let gen = key.rsplit_once(':').and_then(|(_, g)| g.parse::<u64>().ok());
            self.write_l1(gen, key.to_string(), value);
        }
        parsed
    }

    async fn get_versioned_projection(
        &self,
        profile: &str,
        root: &str,
        reader: &str,
    ) -> (u64, Option<Value>) {
        let known = self.known_l1_generation(profile, root, reader);
        let prefix = ProjectionKey::prefix(profile, root, reader);
        let prefix_redis_key = Self::projection_key(&prefix);
        let known_arg = known
            .as_ref()
            .map(|(gen, _)| gen.to_string())
            .unwrap_or_default();
        let script = redis::Script::new(
            r#"
            local gen = tonumber(redis.call('GET', KEYS[1]) or '0')
            if ARGV[1] ~= '' and gen == tonumber(ARGV[1]) then
                return {gen, 1, ''}
            end
            local raw = redis.call('GET', KEYS[2] .. ':' .. gen)
            if not raw then
                return {gen, 0, ''}
            end
            return {gen, 0, raw}
            "#,
        );
        let mut conn = self.connection.clone();
        let result: redis::RedisResult<(u64, u8, String)> = script
            .key(Self::generation_key())
            .key(prefix_redis_key)
            .arg(known_arg)
            .invoke_async(&mut conn)
            .await;
        match result {
            Ok((gen, 1, _)) => {
                if let Some((_, key)) = known {
                    if let Some(cached) = self.read_l1(&key) {
                        return (gen, Some(cached));
                    }
                }
                let fallback_key = ProjectionKey::of(profile, root, reader, gen);
                let val = self.get_projection(&fallback_key).await;
                (gen, val)
            }
            Ok((gen, _, raw)) => {
                self.sync_l1_generation(gen);
                if raw.is_empty() {
                    return (gen, None);
                }
                match serde_json::from_str::<Value>(&raw) {
                    Ok(value) => {
                        let full_key = ProjectionKey::of(profile, root, reader, gen);
                        self.write_l1(Some(gen), full_key, &value);
                        (gen, Some(value))
                    }
                    Err(_) => (gen, None),
                }
            }
            Err(e) => {
                warn!(error = %e, "catalog cache: versioned projection unreadable");
                (0, None)
            }
        }
    }

    async fn put_projection(&self, key: &str, projection: &Value) {
        let gen = key.rsplit_once(':').and_then(|(_, g)| g.parse::<u64>().ok());
        self.write_l1(gen, key.to_string(), projection);
        let mut conn = self.connection.clone();
        let result: redis::RedisResult<()> = conn
            .set_ex(Self::projection_key(key), projection.to_string(), Self::PROJECTION_TTL)
            .await;
        if let Err(e) = result {
            warn!(error = %e, "catalog cache: projection not stored");
        }
    }

    async fn get_peer_catalog(&self, user_id: &str, peer_id: &str) -> Outcome<Option<Value>> {
        let mut conn = self.connection.clone();
        let cached: Option<String> = conn
            .get(Self::peer_key(user_id, peer_id))
            .await
            .map_err(Self::failed)?;
        Ok(cached.and_then(|json| serde_json::from_str(&json).ok()))
    }

    async fn put_peer_catalog(&self, user_id: &str, peer_id: &str, catalog: &Value) -> Outcome<()> {
        let mut conn = self.connection.clone();
        conn.set_ex::<_, _, ()>(
            Self::peer_key(user_id, peer_id),
            catalog.to_string(),
            Self::PEER_CATALOG_TTL,
        )
        .await
        .map_err(Self::failed)
    }
}
