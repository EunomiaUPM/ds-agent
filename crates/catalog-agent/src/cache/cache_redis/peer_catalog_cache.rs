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

use crate::cache::cache_redis::dataservice_cache::DataServiceCacheForRedis;
use crate::cache::cache_traits::peer_catalog_cache_trait::PeerCatalogCacheTrait;
use crate::cache::cache_traits::redis_cache_connector_trait::RedisCacheConnectorTrait;
use crate::cache::cache_traits::utils_trait::UtilsCacheTrait;
use crate::cache::cache_traits::{DESIRED_CACHE_TTL, PEER_CATALOG_DESIRED_CACHE_TTL};
use crate::protocols::dsp::types::catalog_definition::Catalog;
use crate::{CatalogDto, DataServiceDto};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use urn::Urn;
use ymir::errors::{Errors, Outcome};

pub struct DcatCatalogCacheForRedis {
    pub redis_connection: redis::aio::MultiplexedConnection,
}

impl DcatCatalogCacheForRedis {
    pub fn new(redis_connection: redis::aio::MultiplexedConnection) -> Self {
        Self { redis_connection }
    }
}

#[async_trait::async_trait]
impl PeerCatalogCacheTrait for DcatCatalogCacheForRedis {
    async fn get_catalog(&self, participant_id: &String) -> Outcome<Option<Catalog>> {
        tracing::debug!("cache: get peer catalog");
        let key = self.format_key_name_with_string(self.get_entity_name(), participant_id);
        Self::hydrate_from_single_key(self.get_conn(), key).await
    }

    async fn set_catalog(&self, participant_id: &String, catalog: &Catalog) -> Outcome<()> {
        tracing::debug!("cache: set peer catalog");
        let key = self.format_key_name_with_string(self.get_entity_name(), participant_id);
        let json = serde_json::to_string(catalog)?;
        redis::pipe()
            .atomic()
            .cmd("JSON.SET")
            .arg(&key)
            .arg("$")
            .arg(json)
            .cmd("EXPIRE")
            .arg(&key)
            .arg(PEER_CATALOG_DESIRED_CACHE_TTL)
            .query_async::<()>(&mut self.get_conn())
            .await
            .map_err(|e| Errors::crazy("Not able to query cache", Some(Box::new(e))))?;
        Ok(())
    }
}

impl UtilsCacheTrait for DcatCatalogCacheForRedis {
    type Dto = Catalog;
}

impl RedisCacheConnectorTrait for DcatCatalogCacheForRedis {
    type Dto = Catalog;
    fn get_conn(&self) -> redis::aio::MultiplexedConnection {
        self.redis_connection.clone()
    }
    fn get_entity_name(&self) -> &str {
        "peer-catalog"
    }
}
