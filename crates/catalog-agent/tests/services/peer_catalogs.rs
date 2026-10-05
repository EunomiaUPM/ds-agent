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

//! PeerCatalogService over a mocked peer catalog cache and participant facade: catalogs are
//! cached per tenant and listed for the participants the tenant knows.

use std::sync::Arc;

use catalog_agent::cache::cache_traits::peer_catalog_cache_trait::MockPeerCatalogCacheTrait;
use catalog_agent::cache::factory_noop::CatalogAgentCacheNoop;
use catalog_agent::cache::factory_trait::MockCatalogAgentCacheTrait;
use catalog_agent::services::peer_catalogs::service::PeerCatalogService;
use catalog_agent::services::peer_catalogs::PeerCatalogServiceTrait;
use common::facades::mates_facade::MockMatesFacadeTrait;
use common::test_utils::scopes::TestUsers;
use ymir::errors::Errors;

use crate::support::builders::{mate, peer_catalog};

fn service(peers: MockPeerCatalogCacheTrait, mates: MockMatesFacadeTrait) -> PeerCatalogService {
    let peers = Arc::new(peers);
    let mut cache = MockCatalogAgentCacheTrait::new();
    cache
        .expect_get_peer_catalog_cache()
        .returning(move || peers.clone());
    PeerCatalogService::new(Arc::new(cache), Arc::new(mates))
}

/// Only peers with a cached catalog are listed; a cache failure skips that peer.
#[tokio::test]
async fn lists_cached_catalogs_of_known_peers() {
    let mut mates = MockMatesFacadeTrait::new();
    mates
        .expect_get_all_mates()
        .withf(|user| user.id() == "tenant-1")
        .returning(|_| Ok(vec![mate("did:a"), mate("did:b"), mate("did:c")]));
    let mut peers = MockPeerCatalogCacheTrait::new();
    peers.expect_get_catalog().returning(|_, peer| match peer {
        "did:a" => Ok(Some(peer_catalog("urn:catalog:a"))),
        "did:b" => Ok(None),
        _ => Err(Errors::crazy("redis down", None)),
    });

    let listed = service(peers, mates)
        .get_all_peer_catalogs(&TestUsers::user("tenant-1", "/admin/tenant-1"))
        .await
        .unwrap();

    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].0.participant_id, "did:a");
    assert_eq!(listed[0].1.id.to_string(), "urn:catalog:a");
}

/// Reads and writes go to the caller's tenant: a peer may show each tenant another catalog.
#[tokio::test]
async fn catalogs_are_cached_per_tenant() {
    let mut peers = MockPeerCatalogCacheTrait::new();
    peers
        .expect_get_catalog()
        .withf(|user_id, peer| user_id == "tenant-2" && peer == "did:a")
        .times(1)
        .returning(|_, _| Ok(None));
    peers
        .expect_set_catalog()
        .withf(|user_id, peer, catalog| {
            user_id == "tenant-2" && peer == "did:a" && catalog.id.to_string() == "urn:catalog:a"
        })
        .times(1)
        .returning(|_, _, _| Ok(()));
    let svc = service(peers, MockMatesFacadeTrait::new());
    let user = TestUsers::user("tenant-2", "/admin/tenant-2");

    assert!(svc
        .get_peer_catalog(&user, "did:a")
        .await
        .unwrap()
        .is_none());
    svc.set_peer_catalog(&user, "did:a", &peer_catalog("urn:catalog:a"))
        .await
        .unwrap();
}

/// A failure reading the participants fails the listing.
#[tokio::test]
async fn participant_lookup_failure_fails_the_listing() {
    let mut mates = MockMatesFacadeTrait::new();
    mates
        .expect_get_all_mates()
        .returning(|_| Err(Errors::crazy("ssi-auth down", None)));
    let result = service(MockPeerCatalogCacheTrait::new(), mates)
        .get_all_peer_catalogs(&TestUsers::user("tenant-1", "/admin/tenant-1"))
        .await;
    assert!(result.is_err());
}

/// Without a cache a stored peer catalog is dropped and never read back.
#[tokio::test]
async fn noop_cache_never_keeps_a_peer_catalog() {
    let svc = PeerCatalogService::new(
        Arc::new(CatalogAgentCacheNoop),
        Arc::new(MockMatesFacadeTrait::new()),
    );
    let user = TestUsers::user("tenant-1", "/admin/tenant-1");

    svc.set_peer_catalog(&user, "did:a", &peer_catalog("urn:catalog:a"))
        .await
        .unwrap();
    assert!(svc
        .get_peer_catalog(&user, "did:a")
        .await
        .unwrap()
        .is_none());
}
