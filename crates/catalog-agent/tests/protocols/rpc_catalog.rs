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

//! RpcCatalogService over a mocked cache and well-known resolution: what it asks before going
//! to a peer, and the cache it answers from.

use std::sync::Arc;

use catalog_agent::cache::MockCatalogCacheTrait;
use catalog_agent::protocols::dsp::entities::message_types::CatalogDspMessageType;
use catalog_agent::protocols::dsp::entities::rdf_extractor_rpc::EunomiaCatalog;
use catalog_agent::protocols::dsp::facades::well_known_rpc_facade::MockWellKnownRPCFacadeTrait;
use catalog_agent::protocols::dsp::services::rpc_catalog::service::RpcCatalogService;
use catalog_agent::protocols::dsp::services::rpc_catalog::RpcCatalogServiceTrait;
use common::eunomia::EUNOMIA_CONTEXT_URL;
use common::rdf::RdfMessage;
use common::test_utils::scopes::TestUsers;
use serde_json::{json, Value};

use crate::support::context::TestContext;

async fn request(body: Value) -> RdfMessage<EunomiaCatalog> {
    RdfMessage::<EunomiaCatalog>::read_as(&CatalogDspMessageType::CatalogRequestMessage, body).await.unwrap()
}

/// A cached peer catalog answers without resolving or calling the peer.
#[tokio::test]
async fn cached_peer_catalog_answers_without_the_network() {
    let mut cache = MockCatalogCacheTrait::new();
    cache.expect_get_peer_catalog().returning(|_, _| Ok(Some(json!({"@type": "Catalog"}))));
    let ctx = TestContext::new().cache(cache).build();
    let service = RpcCatalogService::new(ctx, Arc::new(MockWellKnownRPCFacadeTrait::new()));
    let body = json!({
        "@context": EUNOMIA_CONTEXT_URL,
        "@type": "RpcCatalogRequestMessage",
        "associatedAgentPeer": "did:web:provider.example"
    });

    let result = service.request_catalog(&TestUsers::alone("ana"), &request(body.clone()).await).await.unwrap();

    assert_eq!(result.kind, "RpcCatalogResult");
    assert_eq!(result.request, body);
    assert_eq!(result.response["@type"], "Catalog");
}

/// A request naming no peer is refused before anything is asked.
#[tokio::test]
async fn request_needs_a_peer() {
    let ctx = TestContext::new().build();
    let service = RpcCatalogService::new(ctx, Arc::new(MockWellKnownRPCFacadeTrait::new()));
    let body = json!({"@context": EUNOMIA_CONTEXT_URL, "@type": "RpcCatalogRequestMessage"});

    let refused = service.request_catalog(&TestUsers::alone("ana"), &request(body).await).await.unwrap_err();

    assert_eq!(refused.info().status_code.as_u16(), 400);
}
