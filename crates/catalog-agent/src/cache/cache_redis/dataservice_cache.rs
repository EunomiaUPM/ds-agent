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

use crate::cache::cache_traits::redis_cache_connector_trait::RedisCacheConnectorTrait;
use crate::cache::cache_traits::utils_trait::UtilsCacheTrait;
use crate::{CatalogDto, DataServiceDto};
use std::str::FromStr;
use urn::Urn;

pub struct DataServiceCacheForRedis {
    redis_connection: redis::aio::MultiplexedConnection,
}

impl DataServiceCacheForRedis {
    pub fn new(redis_connection: redis::aio::MultiplexedConnection) -> Self {
        Self { redis_connection }
    }
}

impl UtilsCacheTrait for DataServiceCacheForRedis {
    type Dto = DataServiceDto;
}

impl RedisCacheConnectorTrait for DataServiceCacheForRedis {
    type Dto = DataServiceDto;
    fn get_conn(&self) -> redis::aio::MultiplexedConnection {
        self.redis_connection.clone()
    }
    fn get_entity_name(&self) -> &str {
        "data-services"
    }
}

#[cfg(test)]
mod test_dataservice {
    use super::*;
    use crate::cache::cache_traits::entity_cache_trait::EntityCacheTrait;
    use crate::cache::cache_traits::lookup_cache_trait::LookupCacheTrait;
    use crate::data::entities::dataservice::Model as DataServiceModel;
    use urn::UrnBuilder;
    use uuid::Uuid;

    async fn setup() -> DataServiceCacheForRedis {
        let redis_url = "redis://default:ds_core_provider_redis@127.0.0.1:6379";
        let client = redis::Client::open(redis_url).unwrap();
        let conn = client.get_multiplexed_async_connection().await.unwrap();
        DataServiceCacheForRedis::new(conn)
    }

    #[tokio::test]
    async fn test_dataservice_lookup_by_catalog() {
        let mut cache = setup().await;

        let catalog_id = UrnBuilder::new("catalog", &Uuid::new_v4().to_string())
            .build()
            .unwrap();
        let ds_id = UrnBuilder::new("data-service", &Uuid::new_v4().to_string())
            .build()
            .unwrap();

        let ds_dto = DataServiceDto {
            inner: DataServiceModel {
                id: ds_id.to_string(),
                dcat_endpoint_description: Some("Some description".to_string()),
                dcat_endpoint_url: "".to_string(),
                dct_conforms_to: Some("https://dct.es".to_string()),
                dct_creator: Some("asd".to_string()),
                dct_title: Some("Internal Data Service".into()),
                dct_description: Some("Some description".to_string()),
                catalog_id: catalog_id.to_string(),
                dct_issued: chrono::Utc::now().into(),
                dct_identifier: Some(ds_id.to_string()),
                dct_modified: None,
                dspace_main_data_service: false,
            },
        };

        // Construct keys for debug
        let ds_single_key = cache.format_key_name_with_id("data-services", &ds_id);
        let lookup_key = cache.format_key_name_lookup("data-services", "catalogs", &catalog_id);

        dbg!(&ds_single_key);
        dbg!(&lookup_key);

        // 1. Persist the single DataService
        cache.set_single(&ds_id, &ds_dto).await.unwrap();

        // 2. Index it under the catalog (Lookup)
        let score = ds_dto.inner.dct_issued.timestamp() as f64;
        cache
            .add_to_relation("catalogs", &catalog_id, &ds_id, score)
            .await
            .unwrap();

        // 3. Retrieve using the relation
        let results = cache
            .get_by_relation("catalogs", &catalog_id, None, None)
            .await
            .unwrap();
        dbg!(&results);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].inner.id, ds_id.to_string());

        println!(
            "Test finished. Inspect in Redis: ZRANGE {} 0 -1",
            lookup_key
        );
    }
}
