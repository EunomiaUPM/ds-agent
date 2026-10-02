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

//! Key formatting and Redis hydration helpers shared by entity caches.

use serde::de::DeserializeOwned;
use urn::Urn;
use ymir::errors::{Errors, Outcome};

/// Key layout and RedisJSON hydration shared by every entity cache.
#[async_trait::async_trait]
pub trait UtilsCacheTrait: Send + Sync {
    /// Type stored in the cache.
    type Dto: DeserializeOwned + Send + Sync;

    /// Prefix scoping every key this cache writes, e.g. "ds_agent_dataplane".
    fn key_namespace(&self) -> &str;

    /// Key for a single entity: <namespace>:entity_name:urn
    fn format_key_name_with_id(&self, entity: &str, id: &Urn) -> String {
        format!("{}:{}:{}", self.key_namespace(), entity, id)
    }

    /// Key for a single entity whose id is not a URN.
    fn format_key_name_with_string(&self, entity: &str, id: &String) -> String {
        format!("{}:{}:{}", self.key_namespace(), entity, id)
    }

    /// Key for the main pointer: <namespace>:entity_name:main
    fn format_key_name_main(&self, entity: &str, tenant_id: &str) -> String {
        format!("{}:{}:main:{}", self.key_namespace(), entity, tenant_id)
    }

    /// Key for the all-entities set: <namespace>:entity_name:all
    fn format_key_name_all(&self, entity: &str) -> String {
        format!("{}:{}:all", self.key_namespace(), entity)
    }

    /// Key for relational lookups: <namespace>:child_entity:parent_entity:parent_id
    fn format_key_name_lookup(
        &self,
        child_entity: &str,
        parent_entity: &str,
        parent_id: &Urn,
    ) -> String {
        format!(
            "{}:{}:{}:{}",
            self.key_namespace(),
            child_entity,
            parent_entity,
            parent_id
        )
    }

    /// Removes the prefix from a key to recover the raw ID/URN
    fn remove_key_name(&self, key: &str, entity: &str) -> String {
        key.replace(&format!("{}:{}:", self.key_namespace(), entity), "")
    }

    /// Redis `ZRANGE` bounds for a 1-based page; no limit and no page mean the whole set.
    fn compute_pagination_range(&self, limit: Option<u64>, page: Option<u64>) -> (isize, isize) {
        match (limit, page) {
            (None, None) => (0, -1),
            _ => {
                let l = limit.unwrap_or(25);
                let p = page.unwrap_or(1);
                let start = ((p.max(1) - 1) * l) as isize;
                let stop = (start + l as isize) - 1;
                (start, stop)
            }
        }
    }

    // --- Hydration logic (Static methods for the Blanket Implementation) ---

    async fn hydrate_from_multiple_keys(
        mut connection: redis::aio::MultiplexedConnection,
        keys: Vec<String>,
    ) -> Outcome<Vec<Self::Dto>> {
        if keys.is_empty() {
            return Ok(vec![]);
        }

        let data: Vec<Option<String>> = redis::cmd("JSON.MGET")
            .arg(&keys)
            .arg("$")
            .query_async(&mut connection)
            .await
            .map_err(|e| Errors::crazy("Redis cache operation failed", Some(Box::new(e))))?;

        let mut results = Vec::with_capacity(data.len());
        for entry in data.into_iter().flatten() {
            let mut models: Vec<Self::Dto> = serde_json::from_str(&entry)?;
            if let Some(m) = models.pop() {
                results.push(m);
            }
        }
        Ok(results)
    }

    async fn hydrate_from_single_key(
        mut connection: redis::aio::MultiplexedConnection,
        key: String,
    ) -> Outcome<Option<Self::Dto>> {
        let data: Option<String> = redis::cmd("JSON.GET")
            .arg(&key)
            .arg("$")
            .query_async(&mut connection)
            .await
            .map_err(|e| Errors::crazy("Redis cache operation failed", Some(Box::new(e))))?;

        if let Some(json_str) = data {
            let mut models: Vec<Self::Dto> = serde_json::from_str(&json_str)?;
            return Ok(models.pop());
        }
        Ok(None)
    }
}
