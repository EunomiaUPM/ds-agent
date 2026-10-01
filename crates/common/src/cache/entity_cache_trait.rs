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

//! Blanket entity cache implemented for any Redis connector with key helpers.

use crate::cache::lookup_cache_trait::LookupCacheTrait;
use crate::cache::redis_cache_connector_trait::RedisCacheConnectorTrait;
use crate::cache::utils_trait::UtilsCacheTrait;
use serde::de::DeserializeOwned;
use serde::Serialize;
use urn::Urn;
use ymir::errors::{Errors, Outcome};

#[async_trait::async_trait]
pub trait EntityCacheTrait<D>: LookupCacheTrait<D> + Send + Sync {
    async fn get_single(&self, id: &Urn) -> Outcome<Option<D>>;
    async fn set_single(&self, id: &Urn, model: &D) -> Outcome<()>;
    async fn delete_single(&self, id: &Urn) -> Outcome<()>;
    /// Each tenant has its own main entity.
    async fn get_main(&self, tenant_id: &str) -> Outcome<Option<D>>;
    async fn set_main(&self, tenant_id: &str, id: &Urn, model: &D) -> Outcome<()>;
    async fn get_collection(&self, limit: Option<u64>, page: Option<u64>) -> Outcome<Vec<D>>;
    async fn add_to_collection(&self, id: &Urn, score: f64) -> Outcome<()>;
    async fn remove_from_collection(&self, id: &Urn) -> Outcome<()>;
    async fn get_batch(&self, ids: &Vec<Urn>) -> Outcome<Vec<D>>;
}

#[async_trait::async_trait]
impl<T, D> EntityCacheTrait<D> for T
where
    T: RedisCacheConnectorTrait<Dto = D>
        + UtilsCacheTrait<Dto = D>
        + LookupCacheTrait<D>
        + Send
        + Sync,
    D: Serialize + DeserializeOwned + Send + Sync + Clone + 'static,
{
    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_single(&self, id: &Urn) -> Outcome<Option<D>> {
        tracing::debug!(entity = self.get_entity_name(), id = %id, "cache: get single");
        let key = self.format_key_name_with_id(self.get_entity_name(), id);
        Self::hydrate_from_single_key(self.get_conn(), key).await
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn set_single(&self, id: &Urn, model: &D) -> Outcome<()> {
        tracing::debug!(entity = self.get_entity_name(), id = %id, "cache: set single");
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
            .arg(self.cache_ttl())
            .query_async::<()>(&mut self.get_conn())
            .await
            .map_err(|e| Errors::crazy("Redis cache operation failed", Some(Box::new(e))))?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn delete_single(&self, id: &Urn) -> Outcome<()> {
        tracing::debug!(entity = self.get_entity_name(), id = %id, "cache: delete single");
        let key = self.format_key_name_with_id(self.get_entity_name(), id);
        let _: () = redis::cmd("DEL")
            .arg(&key)
            .query_async(&mut self.get_conn())
            .await
            .map_err(|e| Errors::crazy("Redis cache operation failed", Some(Box::new(e))))?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_main(&self, tenant_id: &str) -> Outcome<Option<D>> {
        tracing::debug!(entity = self.get_entity_name(), "cache: get main");
        let main_key = self.format_key_name_main(self.get_entity_name(), tenant_id);
        let target_key: Option<String> = redis::cmd("GET")
            .arg(main_key)
            .query_async(&mut self.get_conn())
            .await
            .map_err(|e| Errors::crazy("Redis cache operation failed", Some(Box::new(e))))?;
        if let Some(key) = target_key {
            return Self::hydrate_from_single_key(self.get_conn(), key).await;
        }
        Ok(None)
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn set_main(&self, tenant_id: &str, id: &Urn, model: &D) -> Outcome<()> {
        tracing::debug!(entity = self.get_entity_name(), id = %id, "cache: set main");
        let main_key = self.format_key_name_main(self.get_entity_name(), tenant_id);
        let key = self.format_key_name_with_id(self.get_entity_name(), id);
        self.set_single(id, model).await?;
        let _: () = redis::cmd("SET")
            .arg(main_key)
            .arg(&key)
            .query_async(&mut self.get_conn())
            .await
            .map_err(|e| Errors::crazy("Redis cache operation failed", Some(Box::new(e))))?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_collection(&self, limit: Option<u64>, page: Option<u64>) -> Outcome<Vec<D>> {
        tracing::debug!(entity = self.get_entity_name(), limit = ?limit, page = ?page, "cache: get collection");
        let collection_key = self.format_key_name_all(self.get_entity_name());
        let (start, stop) = self.compute_pagination_range(limit, page);
        let keys: Vec<String> = redis::cmd("ZREVRANGE")
            .arg(collection_key)
            .arg(start)
            .arg(stop)
            .query_async(&mut self.get_conn())
            .await
            .map_err(|e| Errors::crazy("Redis cache operation failed", Some(Box::new(e))))?;

        Self::hydrate_from_multiple_keys(self.get_conn(), keys).await
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn add_to_collection(&self, id: &Urn, score: f64) -> Outcome<()> {
        tracing::debug!(entity = self.get_entity_name(), id = %id, "cache: add to collection");
        let key = self.format_key_name_with_id(self.get_entity_name(), id);
        let collection_key = self.format_key_name_all(self.get_entity_name());
        let _: () = redis::cmd("ZADD")
            .arg(collection_key)
            .arg(score)
            .arg(key)
            .query_async(&mut self.get_conn())
            .await
            .map_err(|e| Errors::crazy("Redis cache operation failed", Some(Box::new(e))))?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn remove_from_collection(&self, id: &Urn) -> Outcome<()> {
        tracing::debug!(entity = self.get_entity_name(), id = %id, "cache: remove from collection");
        let key = self.format_key_name_with_id(self.get_entity_name(), id);
        let collection_key = self.format_key_name_all(self.get_entity_name());
        let _: () = redis::cmd("ZREM")
            .arg(collection_key)
            .arg(key)
            .query_async(&mut self.get_conn())
            .await
            .map_err(|e| Errors::crazy("Redis cache operation failed", Some(Box::new(e))))?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_batch(&self, ids: &Vec<Urn>) -> Outcome<Vec<D>> {
        tracing::debug!(
            entity = self.get_entity_name(),
            count = ids.len(),
            "cache: get batch"
        );
        let keys: Vec<String> = ids
            .iter()
            .map(|id| self.format_key_name_with_id(self.get_entity_name(), id))
            .collect();
        Self::hydrate_from_multiple_keys(self.get_conn(), keys).await
    }
}

#[derive(Default, Clone)]
pub struct NoopCache<D> {
    _phantom: std::marker::PhantomData<D>,
}

impl<D> NoopCache<D> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<D: Send + Sync + 'static> EntityCacheTrait<D> for NoopCache<D> {
    async fn get_single(&self, _id: &Urn) -> Outcome<Option<D>> {
        Ok(None)
    }

    async fn set_single(&self, _id: &Urn, _model: &D) -> Outcome<()> {
        Ok(())
    }

    async fn delete_single(&self, _id: &Urn) -> Outcome<()> {
        Ok(())
    }

    async fn get_main(&self, _tenant_id: &str) -> Outcome<Option<D>> {
        Ok(None)
    }

    async fn set_main(&self, _tenant_id: &str, _id: &Urn, _model: &D) -> Outcome<()> {
        Ok(())
    }

    async fn get_collection(&self, _limit: Option<u64>, _page: Option<u64>) -> Outcome<Vec<D>> {
        Ok(vec![])
    }

    async fn add_to_collection(&self, _id: &Urn, _score: f64) -> Outcome<()> {
        Ok(())
    }

    async fn remove_from_collection(&self, _id: &Urn) -> Outcome<()> {
        Ok(())
    }

    async fn get_batch(&self, _ids: &Vec<Urn>) -> Outcome<Vec<D>> {
        Ok(vec![])
    }
}

#[async_trait::async_trait]
impl<D: Send + Sync + 'static> LookupCacheTrait<D> for NoopCache<D> {
    async fn get_by_relation(
        &self,
        _parent_name: &str,
        _parent_id: &Urn,
        _limit: Option<u64>,
        _page: Option<u64>,
    ) -> Outcome<Vec<D>> {
        Ok(vec![])
    }

    async fn add_to_relation(
        &self,
        _parent_name: &str,
        _parent_id: &Urn,
        _child_id: &Urn,
        _score: f64,
    ) -> Outcome<()> {
        Ok(())
    }

    async fn remove_from_relation(
        &self,
        _parent_name: &str,
        _parent_id: &Urn,
        _child_id: &Urn,
    ) -> Outcome<()> {
        Ok(())
    }
}
