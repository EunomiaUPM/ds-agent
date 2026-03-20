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
use serde::de::DeserializeOwned;
use urn::Urn;
use ymir::errors::{Errors, Outcome};

#[async_trait::async_trait]
pub trait LookupCacheTrait<D>: Send + Sync {
    async fn get_by_relation(
        &self,
        parent_name: &str,
        parent_id: &Urn,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Outcome<Vec<D>>;
    async fn add_to_relation(
        &self,
        parent_name: &str,
        parent_id: &Urn,
        child_id: &Urn,
        score: f64,
    ) -> Outcome<()>;
    async fn remove_from_relation(
        &self,
        parent_name: &str,
        parent_id: &Urn,
        child_id: &Urn,
    ) -> Outcome<()>;
}

#[async_trait::async_trait]
impl<T, D> LookupCacheTrait<D> for T
where
    T: RedisCacheConnectorTrait<Dto = D> + UtilsCacheTrait<Dto = D> + Send + Sync,
    D: serde::Serialize + DeserializeOwned + Send + Sync + Clone + 'static,
{
    async fn get_by_relation(
        &self,
        parent_name: &str,
        parent_id: &Urn,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Outcome<Vec<D>> {
        tracing::debug!("cache: get by relation");
        let lookup_key =
            self.format_key_name_lookup(self.get_entity_name(), parent_name, parent_id);
        let (start, stop) = self.compute_pagination_range(limit, page);

        let keys: Vec<String> = redis::cmd("ZREVRANGE")
            .arg(&lookup_key)
            .arg(start)
            .arg(stop)
            .query_async(&mut self.get_conn())
            .await
            .map_err(|e| Errors::crazy("Not able to query cache", Some(Box::new(e))))?;

        Self::hydrate_from_multiple_keys(self.get_conn(), keys).await
    }

    async fn add_to_relation(
        &self,
        parent_name: &str,
        parent_id: &Urn,
        child_id: &Urn,
        score: f64,
    ) -> Outcome<()> {
        tracing::debug!("cache: add to relation");
        let lookup_key =
            self.format_key_name_lookup(self.get_entity_name(), parent_name, parent_id);
        let child_key = self.format_key_name_with_id(self.get_entity_name(), child_id);

        let _: () = redis::cmd("ZADD")
            .arg(lookup_key)
            .arg(score)
            .arg(child_key)
            .query_async(&mut self.get_conn())
            .await
            .map_err(|e| Errors::crazy("Not able to query cache", Some(Box::new(e))))?;
        Ok(())
    }

    async fn remove_from_relation(
        &self,
        parent_name: &str,
        parent_id: &Urn,
        child_id: &Urn,
    ) -> Outcome<()> {
        tracing::debug!("cache: remove from relation");
        let lookup_key =
            self.format_key_name_lookup(self.get_entity_name(), parent_name, parent_id);
        let child_key = self.format_key_name_with_id(self.get_entity_name(), child_id);

        let _: () = redis::cmd("ZREM")
            .arg(lookup_key)
            .arg(child_key)
            .query_async(&mut self.get_conn())
            .await
            .map_err(|e| Errors::crazy("Not able to query cache", Some(Box::new(e))))?;
        Ok(())
    }
}
