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

//! RedisCatalogCache: projections stamped with a generation, and the catalogs of peers.

use catalog_agent::cache::{CatalogCacheTrait, ProjectionKey, RedisCatalogCache};
use serde_json::json;
use uuid::Uuid;

async fn cache() -> RedisCatalogCache {
    RedisCatalogCache::new(super::redis().await)
}

/// A projection stored under the current generation is read back; after an invalidation the
/// key of the new generation misses it.
#[tokio::test]
#[ignore = "needs REDIS_URL"]
async fn invalidation_moves_the_generation() {
    let cache = cache().await;
    let root = format!("urn:catalog:{}", Uuid::new_v4());
    let before = cache.generation().await;
    let key = ProjectionKey::of("dcat3", &root, "public", before);
    cache.put_projection(&key, &json!({"@graph": []})).await;

    assert_eq!(cache.get_projection(&key).await, Some(json!({"@graph": []})));
    cache.invalidate().await;
    let after = cache.generation().await;
    assert!(after > before);
    assert_eq!(cache.get_projection(&ProjectionKey::of("dcat3", &root, "public", after)).await, None);
}

/// A peer's catalog is kept per user and peer, as it was sent.
#[tokio::test]
#[ignore = "needs REDIS_URL"]
async fn peer_catalogs_are_kept_per_user() {
    let cache = cache().await;
    let peer = format!("did:web:{}.example", Uuid::new_v4().simple());
    let catalog = json!({"@type": "Catalog", "dataset": [{"@id": "urn:dataset:1"}]});

    cache.put_peer_catalog("ana", &peer, &catalog).await.unwrap();

    assert_eq!(cache.get_peer_catalog("ana", &peer).await.unwrap(), Some(catalog));
    assert_eq!(cache.get_peer_catalog("bob", &peer).await.unwrap(), None);
}
