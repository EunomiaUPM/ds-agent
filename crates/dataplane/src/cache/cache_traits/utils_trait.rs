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

use serde::de::DeserializeOwned;
use urn::Urn;
use ymir::errors::{Errors, Outcome};

#[async_trait::async_trait]
pub trait UtilsCacheTrait: Send + Sync {
    type Dto: DeserializeOwned + Send + Sync;

    /// Key for a single entity: ds_agent_dataplane:entity_name:urn
    fn format_key_name_with_id(&self, entity: &str, id: &Urn) -> String {
        format!("ds_agent_dataplane:{}:{}", entity, id)
    }

    fn format_key_name_with_string(&self, entity: &str, id: &String) -> String {
        format!("ds_agent_dataplane:{}:{}", entity, id)
    }

    /// Key for the main pointer: ds_agent_dataplane:entity_name:main
    fn format_key_name_main(&self, entity: &str) -> String {
        format!("ds_agent_dataplane:{}:main", entity)
    }

    /// Key for the all-entities set: ds_agent_dataplane:entity_name:all
    fn format_key_name_all(&self, entity: &str) -> String {
        format!("ds_agent_dataplane:{}:all", entity)
    }

    /// Key for relational lookups: ds_agent_dataplane:child_entity:parent_entity:parent_id
    /// Example: ds_agent_dataplane:dataset:catalog:urn:catalog:123
    fn format_key_name_lookup(
        &self,
        child_entity: &str,
        parent_entity: &str,
        parent_id: &Urn,
    ) -> String {
        format!(
            "ds_agent_dataplane:{}:{}:{}",
            child_entity, parent_entity, parent_id
        )
    }

    /// Removes the prefix from a key to recover the raw ID/URN
    fn remove_key_name(&self, key: &str, entity: &str) -> String {
        key.replace(&format!("ds_agent_dataplane:{}:", entity), "")
    }

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
            .map_err(|e| Errors::crazy("Redis not able to query", Some(Box::new(e))))?;

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
            .map_err(|e| Errors::crazy("Redis not able to query", Some(Box::new(e))))?;

        if let Some(json_str) = data {
            let mut models: Vec<Self::Dto> = serde_json::from_str(&json_str)?;
            return Ok(models.pop());
        }
        Ok(None)
    }
}
