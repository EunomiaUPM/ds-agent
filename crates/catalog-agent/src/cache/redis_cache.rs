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

use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use serde_json::Value;
use tracing::warn;
use ymir::errors::{Errors, Outcome};

use crate::cache::CatalogCacheTrait;

pub struct RedisCatalogCache {
    connection: MultiplexedConnection,
}

impl RedisCatalogCache {
    const NAMESPACE: &'static str = "ds_agent_catalog";
    /// Projections are rebuilt at most once an hour even with no write.
    const PROJECTION_TTL: u64 = 3600;
    /// What a peer published is kept a day.
    const PEER_CATALOG_TTL: u64 = 86400;

    pub fn new(connection: MultiplexedConnection) -> Self {
        Self { connection }
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
        let mut conn = self.connection.clone();
        if let Err(e) = conn.incr::<_, _, u64>(Self::generation_key(), 1).await {
            warn!(error = %e, "catalog cache: generation not bumped");
        }
    }

    async fn get_projection(&self, key: &str) -> Option<Value> {
        let mut conn = self.connection.clone();
        let cached: Option<String> = conn.get(Self::projection_key(key)).await.ok().flatten();
        cached.and_then(|json| serde_json::from_str(&json).ok())
    }

    async fn put_projection(&self, key: &str, projection: &Value) {
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
