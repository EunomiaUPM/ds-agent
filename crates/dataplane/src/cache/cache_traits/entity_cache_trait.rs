/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

use crate::cache::cache_traits::redis_cache_connector_trait::RedisCacheConnectorTrait;
use crate::cache::cache_traits::utils_trait::UtilsCacheTrait;
use crate::cache::cache_traits::DESIRED_CACHE_TTL;
use serde::de::DeserializeOwned;
use serde::Serialize;
use urn::Urn;
use ymir::errors::{Errors, Outcome};

#[async_trait::async_trait]
pub trait EntityCacheTrait<D>: Send + Sync {
    async fn get_single(&self, id: &Urn) -> Outcome<Option<D>>;
    async fn set_single(&self, id: &Urn, model: &D) -> Outcome<()>;
    async fn delete_single(&self, id: &Urn) -> Outcome<()>;
    async fn get_main(&self) -> Outcome<Option<D>>;
    async fn set_main(&self, id: &Urn, model: &D) -> Outcome<()>;
    async fn get_collection(&self, limit: Option<u64>, page: Option<u64>) -> Outcome<Vec<D>>;
    async fn add_to_collection(&self, id: &Urn, score: f64) -> Outcome<()>;
    async fn remove_from_collection(&self, id: &Urn) -> Outcome<()>;
    async fn get_batch(&self, ids: &Vec<Urn>) -> Outcome<Vec<D>>;
}

#[async_trait::async_trait]
impl<T, D> EntityCacheTrait<D> for T
where
    T: RedisCacheConnectorTrait<Dto = D> + UtilsCacheTrait<Dto = D> + Send + Sync,
    D: Serialize + DeserializeOwned + Send + Sync + Clone + 'static,
{
    async fn get_single(&self, id: &Urn) -> Outcome<Option<D>> {
        tracing::debug!("cache: get single");
        let key = self.format_key_name_with_id(self.get_entity_name(), id);
        Self::hydrate_from_single_key(self.get_conn(), key).await
    }

    async fn set_single(&self, id: &Urn, model: &D) -> Outcome<()> {
        tracing::debug!("cache: set single");
        let key = self.format_key_name_with_id(self.get_entity_name(), id);
        let json = serde_json::to_string(model)?;
        redis::pipe()
            .atomic()
            .cmd("JSON.SET")
            .arg(&key)
            .arg("$")
            .arg(json)
            .cmd("EXPIRE")
            .arg(&key)
            .arg(DESIRED_CACHE_TTL)
            .query_async::<()>(&mut self.get_conn())
            .await
            .map_err(|e| Errors::crazy("Redis not able to query", Some(Box::new(e))))?;
        Ok(())
    }

    async fn delete_single(&self, id: &Urn) -> Outcome<()> {
        tracing::debug!("cache: delete single");
        let key = self.format_key_name_with_id(self.get_entity_name(), id);
        let _: () = redis::cmd("DEL")
            .arg(&key)
            .query_async(&mut self.get_conn())
            .await
            .map_err(|e| Errors::crazy("Redis not able to query", Some(Box::new(e))))?;
        Ok(())
    }

    async fn get_main(&self) -> Outcome<Option<D>> {
        tracing::debug!("cache: get main");
        let main_key = self.format_key_name_main(self.get_entity_name());
        let target_key: Option<String> = redis::cmd("GET")
            .arg(main_key)
            .query_async(&mut self.get_conn())
            .await
            .map_err(|e| Errors::crazy("Redis not able to query", Some(Box::new(e))))?;
        if let Some(key) = target_key {
            return Self::hydrate_from_single_key(self.get_conn(), key).await;
        }
        Ok(None)
    }

    async fn set_main(&self, id: &Urn, model: &D) -> Outcome<()> {
        tracing::debug!("cache: set main");
        let main_key = self.format_key_name_main(self.get_entity_name());
        let key = self.format_key_name_with_id(self.get_entity_name(), id);
        self.set_single(id, model).await?;
        let _: () = redis::cmd("SET")
            .arg(main_key)
            .arg(&key)
            .query_async(&mut self.get_conn())
            .await
            .map_err(|e| Errors::crazy("Redis not able to query", Some(Box::new(e))))?;
        Ok(())
    }

    async fn get_collection(&self, limit: Option<u64>, page: Option<u64>) -> Outcome<Vec<D>> {
        tracing::debug!("cache: get collection all");
        let collection_key = self.format_key_name_all(self.get_entity_name());
        let (start, stop) = self.compute_pagination_range(limit, page);
        let keys: Vec<String> = redis::cmd("ZREVRANGE")
            .arg(collection_key)
            .arg(start)
            .arg(stop)
            .query_async(&mut self.get_conn())
            .await
            .map_err(|e| Errors::crazy("Redis not able to query", Some(Box::new(e))))?;

        Self::hydrate_from_multiple_keys(self.get_conn(), keys).await
    }

    async fn add_to_collection(&self, id: &Urn, score: f64) -> Outcome<()> {
        tracing::debug!("cache: add to collection all");
        let key = self.format_key_name_with_id(self.get_entity_name(), id);
        let collection_key = self.format_key_name_all(self.get_entity_name());
        let _: () = redis::cmd("ZADD")
            .arg(collection_key)
            .arg(score)
            .arg(key)
            .query_async(&mut self.get_conn())
            .await
            .map_err(|e| Errors::crazy("Redis not able to query", Some(Box::new(e))))?;
        Ok(())
    }

    async fn remove_from_collection(&self, id: &Urn) -> Outcome<()> {
        tracing::debug!("cache: remove from collection all");
        let key = self.format_key_name_with_id(self.get_entity_name(), id);
        let collection_key = self.format_key_name_all(self.get_entity_name());
        let _: () = redis::cmd("ZREM")
            .arg(collection_key)
            .arg(key)
            .query_async(&mut self.get_conn())
            .await
            .map_err(|e| Errors::crazy("Redis not able to query", Some(Box::new(e))))?;
        Ok(())
    }

    async fn get_batch(&self, ids: &Vec<Urn>) -> Outcome<Vec<D>> {
        tracing::debug!("cache: get batch");
        let keys: Vec<String> = ids
            .iter()
            .map(|id| self.format_key_name_with_id(self.get_entity_name(), id))
            .collect();
        Self::hydrate_from_multiple_keys(self.get_conn(), keys).await
    }
}
