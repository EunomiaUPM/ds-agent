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

//! CatalogCacheForRedis: single entries, the per-tenant main pointer, batches and the
//! newest-first collection.

use catalog_agent::cache::cache_redis::catalog_cache::CatalogCacheForRedis;
use catalog_agent::data::entities::catalog::Model;
use catalog_agent::entities::catalogs::CatalogDto;
use common::cache::EntityCacheTrait;
use urn::{Urn, UrnBuilder};
use uuid::Uuid;

async fn cache() -> CatalogCacheForRedis {
    CatalogCacheForRedis::new(super::redis().await)
}

fn catalog(title: &str) -> (Urn, CatalogDto) {
    let id = UrnBuilder::new("catalog", &Uuid::new_v4().to_string())
        .build()
        .unwrap();
    let dto = CatalogDto {
        inner: Model {
            id: id.to_string(),
            tenant_id: "default".to_string(),
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

/// The main catalog is found through the tenant's main pointer.
#[tokio::test]
#[ignore = "needs REDIS_URL"]
async fn main_pointer_resolves_to_the_catalog() {
    let cache = cache().await;
    let tenant = format!("tenant-{}", Uuid::new_v4().simple());
    let (id, dto) = catalog("Main Entry");

    cache.set_main(&tenant, &id, &dto).await.unwrap();

    let found = cache.get_main(&tenant).await.unwrap();
    assert_eq!(found.unwrap().inner.id, id.to_string());
}

/// Several entries come back in one round trip, in the order asked.
#[tokio::test]
#[ignore = "needs REDIS_URL"]
async fn batch_returns_entries_in_order() {
    let cache = cache().await;
    let (id1, dto1) = catalog("Batch 1");
    let (id2, dto2) = catalog("Batch 2");
    cache.set_single(&id1, &dto1).await.unwrap();
    cache.set_single(&id2, &dto2).await.unwrap();

    let batch = cache
        .get_batch(&vec![id1.clone(), id2.clone()])
        .await
        .unwrap();

    assert_eq!(batch.len(), 2);
    assert_eq!(batch[0].inner.id, id1.to_string());
}

/// The collection pages newest first by score.
#[tokio::test]
#[ignore = "needs REDIS_URL"]
async fn collection_pages_newest_first() {
    let cache = cache().await;
    let (older, older_dto) = catalog("Older");
    let (newer, newer_dto) = catalog("Newer");
    // Scores above any timestamp, so dev entries in the shared collection rank below.
    cache.set_single(&older, &older_dto).await.unwrap();
    cache.add_to_collection(&older, 1e15).await.unwrap();
    cache.set_single(&newer, &newer_dto).await.unwrap();
    cache.add_to_collection(&newer, 1e15 + 1.0).await.unwrap();

    let first = cache.get_collection(Some(1), Some(1)).await.unwrap();

    cache.remove_from_collection(&older).await.unwrap();
    cache.remove_from_collection(&newer).await.unwrap();
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].inner.id, newer.to_string());
}

/// A deleted entry is gone, and can then leave the collection.
#[tokio::test]
#[ignore = "needs REDIS_URL"]
async fn deleted_entry_is_gone() {
    let cache = cache().await;
    let (id, dto) = catalog("To Be Deleted");
    cache.set_single(&id, &dto).await.unwrap();
    cache.add_to_collection(&id, 500.0).await.unwrap();

    cache.delete_single(&id).await.unwrap();

    assert!(cache.get_single(&id).await.unwrap().is_none());
    cache.remove_from_collection(&id).await.unwrap();
}
