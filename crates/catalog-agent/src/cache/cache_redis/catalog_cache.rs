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
use crate::CatalogDto;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use urn::Urn;

pub struct CatalogCacheForRedis {
    pub redis_connection: redis::aio::MultiplexedConnection,
}

impl CatalogCacheForRedis {
    pub fn new(redis_connection: redis::aio::MultiplexedConnection) -> Self {
        Self { redis_connection }
    }
}

impl UtilsCacheTrait for CatalogCacheForRedis {
    type Dto = CatalogDto;
}
impl RedisCacheConnectorTrait for CatalogCacheForRedis {
    type Dto = CatalogDto;
    fn get_conn(&self) -> redis::aio::MultiplexedConnection {
        self.redis_connection.clone()
    }
    fn get_entity_name(&self) -> &str {
        "catalogs"
    }
}

#[cfg(test)]
mod test_catalog_complete {
    use super::*;
    use crate::cache::cache_traits::entity_cache_trait::EntityCacheTrait;
    use crate::cache::cache_traits::utils_trait::UtilsCacheTrait;
    use crate::data::entities::catalog::Model;
    use urn::UrnBuilder;
    use uuid::Uuid;

    async fn setup() -> CatalogCacheForRedis {
        let redis_url = "redis://default:ds_core_provider_redis@127.0.0.1:6379";
        let client = redis::Client::open(redis_url).unwrap();
        let conn = client
            .get_multiplexed_async_connection()
            .await
            .expect("Redis connection failed");
        CatalogCacheForRedis::new(conn)
    }

    fn mock_catalog(title: &str) -> (Urn, CatalogDto) {
        let id = UrnBuilder::new("catalog", &Uuid::new_v4().to_string())
            .build()
            .unwrap();
        let dto = CatalogDto {
            inner: Model {
                id: id.to_string(),
                foaf_home_page: None,
                dct_conforms_to: None,
                dct_title: Some(title.to_string()),
                dspace_participant_id: None,
                dct_issued: chrono::Utc::now().into(),
                dct_identifier: Some(id.to_string()),
                dct_creator: None,
                dct_modified: None,
                dspace_main_catalog: false,
            },
        };
        (id, dto)
    }

    #[tokio::test]
    async fn test_main_pointer_flow() {
        let mut cache = setup().await;
        let (id, dto) = mock_catalog("Main Entry");

        // Debug keys
        dbg!(cache.format_key_name_with_id("catalogs", &id));
        dbg!(cache.format_key_name_main("catalogs"));

        // Set main entry
        cache.set_main(&id, &dto).await.unwrap();

        // Retrieve via main pointer
        let result = cache.get_main().await.unwrap();
        dbg!(&result);

        assert!(result.is_some());
        assert_eq!(result.unwrap().inner.id, id.to_string());
    }

    #[tokio::test]
    async fn test_batch_hydration() {
        let mut cache = setup().await;
        let (id1, dto1) = mock_catalog("Batch 1");
        let (id2, dto2) = mock_catalog("Batch 2");

        cache.set_single(&id1, &dto1).await.unwrap();
        cache.set_single(&id2, &dto2).await.unwrap();

        // Fetch multiple entities in one Round Trip
        let batch = cache
            .get_batch(&vec![id1.clone(), id2.clone()])
            .await
            .unwrap();
        dbg!(&batch);

        assert_eq!(batch.len(), 2);
        assert_eq!(batch[0].inner.id, id1.to_string());
    }

    #[tokio::test]
    async fn test_collection_pagination() {
        let mut cache = setup().await;
        let (id1, dto1) = mock_catalog("Oldest");
        let (id2, dto2) = mock_catalog("Newest");

        // Register in 'all' collection with scores
        cache.set_single(&id1, &dto1).await.unwrap();
        cache.add_to_collection(&id1, 1000.0).await.unwrap();

        cache.set_single(&id2, &dto2).await.unwrap();
        cache.add_to_collection(&id2, 2000.0).await.unwrap();

        // Test newest-first pagination (ZREVRANGE)
        let collection = cache.get_collection(Some(1), Some(1)).await.unwrap();
        dbg!(&collection);

        assert_eq!(collection.len(), 1);
        assert_eq!(collection[0].inner.id, id2.to_string()); // Highest score first

        // Manual verification key
        dbg!(cache.format_key_name_all("catalogs"));
    }

    #[tokio::test]
    async fn test_deletion_integrity() {
        let mut cache = setup().await;
        let (id, dto) = mock_catalog("To Be Deleted");

        cache.set_single(&id, &dto).await.unwrap();
        cache.add_to_collection(&id, 500.0).await.unwrap();

        // Remove from single storage
        cache.delete_single(&id).await.unwrap();
        let check = cache.get_single(&id).await.unwrap();
        dbg!(&check);
        assert!(check.is_none());

        // Pointer remains in ZSET until explicitly removed (or index repair)
        cache.remove_from_collection(&id).await.unwrap();
    }
}
