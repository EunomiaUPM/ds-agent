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

//! DspCatalogService over mocked closures and cache: projections served and cached per peer.

use catalog_agent::cache::MockCatalogCacheTrait;
use catalog_agent::entities::resource_kind::ResourceKind;
use catalog_agent::protocols::dsp::entities::protocol_fields::CatalogProtocolFields;
use catalog_agent::protocols::dsp::services::dsp_catalog::service::DspCatalogService;
use catalog_agent::protocols::dsp::services::dsp_catalog::DspCatalogServiceTrait;
use catalog_agent::services::closure::MockClosureServiceTrait;
use common::test_utils::scopes::TestUsers;
use serde_json::json;

use crate::support::builders::{id, provider_set, provider_set_with_draft, urn};
use crate::support::context::TestContext;

fn closures() -> MockClosureServiceTrait {
    let mut closures = MockClosureServiceTrait::new();
    closures
        .expect_main_catalog()
        .returning(|_| Ok(provider_set()));
    closures.expect_resource().returning(|_, id| {
        let (_, set) = provider_set();
        Ok(set.get(id.as_ref()).cloned().map(|root| (root, set)))
    });
    closures
}

/// A miss projects the catalog through the DSP adapter and stores it under the generation.
#[tokio::test]
async fn catalog_is_projected_and_cached() {
    let mut cache = MockCatalogCacheTrait::new();
    cache.expect_generation().returning(|| 4);
    cache.expect_get_projection().returning(|_| None);
    cache
        .expect_put_projection()
        .withf(|key, _| key.starts_with("dsp:main:peer|") && key.ends_with(":4"))
        .times(1)
        .return_const(());
    let ctx = TestContext::new().closures(closures()).cache(cache).build();

    let catalog = DspCatalogService::new(ctx)
        .catalog(&TestUsers::alone("peer"), &CatalogProtocolFields::default())
        .await
        .unwrap();

    assert_eq!(catalog["@type"], "Catalog");
    assert_eq!(catalog["dataset"][0]["@id"], id(ResourceKind::Dataset, 3));
}

/// A hit is served from the cache, the catalog is not loaded.
#[tokio::test]
async fn cached_catalog_is_served_without_loading() {
    let mut cache = MockCatalogCacheTrait::new();
    cache.expect_generation().returning(|| 4);
    cache
        .expect_get_projection()
        .returning(|_| Some(json!({"@type": "Catalog", "cached": true})));
    let ctx = TestContext::new().cache(cache).build();

    let catalog = DspCatalogService::new(ctx)
        .catalog(&TestUsers::alone("peer"), &CatalogProtocolFields::default())
        .await
        .unwrap();

    assert_eq!(catalog["cached"], true);
}

/// Only datasets answer a dataset request; anything else is a missing dataset.
#[tokio::test]
async fn dataset_request_answers_datasets_only() {
    let ctx = TestContext::new().closures(closures()).build();
    let service = DspCatalogService::new(ctx);
    let peer = TestUsers::alone("peer");

    let dataset = service
        .dataset(&peer, &urn(ResourceKind::Dataset, 3))
        .await
        .unwrap();
    let not_one = service
        .dataset(&peer, &urn(ResourceKind::DataService, 2))
        .await
        .unwrap_err();

    assert_eq!(dataset["@type"], "Dataset");
    assert_eq!(dataset["hasPolicy"][0]["@type"], "Offer");
    assert_eq!(not_one.info().status_code.as_u16(), 404);
}

/// A draft dataset is left out of the catalog and answers a dataset request like a missing one.
#[tokio::test]
async fn drafts_are_not_offered_to_peers() {
    let mut closures = MockClosureServiceTrait::new();
    closures
        .expect_main_catalog()
        .returning(|_| Ok(provider_set_with_draft()));
    closures.expect_resource().returning(|_, id| {
        let (_, set) = provider_set_with_draft();
        Ok(set.get(id.as_ref()).cloned().map(|root| (root, set)))
    });
    let service = DspCatalogService::new(TestContext::new().closures(closures).build());
    let peer = TestUsers::alone("peer");

    let catalog = service
        .catalog(&peer, &CatalogProtocolFields::default())
        .await
        .unwrap();
    let missing = service
        .dataset(&peer, &urn(ResourceKind::Dataset, 3))
        .await
        .unwrap_err();

    assert!(!catalog.to_string().contains(&id(ResourceKind::Dataset, 3)));
    assert!(!catalog.to_string().contains("publicationState"));
    assert_eq!(missing.info().status_code.as_u16(), 404);
}
