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

//! CatalogResourceGrpc over a mocked resource service: documents travel as JSON-LD strings.

use catalog_agent::entities::resource_kind::ResourceKind;
use catalog_agent::grpc::api::catalog_agent::catalog_resource_service_server::CatalogResourceService;
use catalog_agent::grpc::api::catalog_agent::{CreateResourceRequest, ResourceRef};
use catalog_agent::grpc::resources::CatalogResourceGrpc;
use catalog_agent::services::resources::MockResourceServiceTrait;
use common::test_utils::grpc::GrpcRequests;
use tonic::Code;

use crate::support::builders::{dataset, id};
use crate::support::context::TestContext;

/// A resource answers its kind, version and JSON-LD document.
#[tokio::test]
async fn get_answers_the_document() {
    let mut service = MockResourceServiceTrait::new();
    service.expect_get().returning(|_, _, _| Ok(dataset(3, 1)));
    let grpc = CatalogResourceGrpc::new(TestContext::new().resources(service).build());

    let resource = grpc
        .get(GrpcRequests::user(ResourceRef { kind: "dataset".into(), id: id(ResourceKind::Dataset, 3) }))
        .await
        .unwrap()
        .into_inner();

    assert_eq!(resource.kind, "dataset");
    assert_eq!(resource.version, 1);
    let document: serde_json::Value = serde_json::from_str(&resource.document).unwrap();
    assert_eq!(document["@id"], id(ResourceKind::Dataset, 3));
}

/// A request without a token, an unknown kind or a document that is not JSON is refused.
#[tokio::test]
async fn bad_requests_are_refused() {
    let grpc = CatalogResourceGrpc::new(TestContext::new().resources(MockResourceServiceTrait::new()).build());
    let create = |kind: &str, document: &str| CreateResourceRequest {
        kind: kind.into(),
        document: document.into(),
        visibility: None,
    };

    let anonymous = grpc
        .create(GrpcRequests::with_auth(create("dataset", "{}"), None))
        .await
        .unwrap_err();
    let unknown = grpc.create(GrpcRequests::user(create("table", "{}"))).await.unwrap_err();
    let not_json = grpc.create(GrpcRequests::user(create("dataset", "{"))).await.unwrap_err();

    assert_eq!(anonymous.code(), Code::Unauthenticated);
    assert_eq!(unknown.code(), Code::InvalidArgument);
    assert_eq!(not_json.code(), Code::InvalidArgument);
}
