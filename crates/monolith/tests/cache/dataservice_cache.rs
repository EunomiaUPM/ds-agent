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

//! DataServiceCacheForRedis: data services indexed under their catalog.

use catalog_agent::cache::cache_redis::dataservice_cache::DataServiceCacheForRedis;
use catalog_agent::data::entities::dataservice::Model;
use catalog_agent::entities::data_services::DataServiceDto;
use common::cache::{EntityCacheTrait, LookupCacheTrait};
use urn::UrnBuilder;
use uuid::Uuid;

/// A data service indexed under a catalog is found through that relation.
#[tokio::test]
#[ignore = "needs REDIS_URL"]
async fn data_service_is_found_by_its_catalog() {
    let cache = DataServiceCacheForRedis::new(super::redis().await);
    let catalog_id = UrnBuilder::new("catalog", &Uuid::new_v4().to_string())
        .build()
        .unwrap();
    let ds_id = UrnBuilder::new("data-service", &Uuid::new_v4().to_string())
        .build()
        .unwrap();
    let dto = DataServiceDto {
        inner: Model {
            id: ds_id.to_string(),
            user_id: "default".to_string(),
            user_role: common::oauth::RolePath::root(),
            visibility: common::oauth::Visibility::Private,
            dcat_endpoint_description: None,
            dcat_endpoint_url: "https://svc.example".to_string(),
            dct_conforms_to: None,
            dct_creator: None,
            dct_title: Some("Internal Data Service".into()),
            dct_description: None,
            catalog_id: catalog_id.to_string(),
            dct_issued: chrono::Utc::now().into(),
            dct_identifier: Some(ds_id.to_string()),
            dct_modified: None,
            dspace_main_data_service: false,
        },
    };

    cache.set_single(&ds_id, &dto).await.unwrap();
    let score = dto.inner.dct_issued.timestamp() as f64;
    cache
        .add_to_relation("catalogs", &catalog_id, &ds_id, score)
        .await
        .unwrap();

    let found = cache
        .get_by_relation("catalogs", &catalog_id, None, None)
        .await
        .unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].inner.id, ds_id.to_string());
}
