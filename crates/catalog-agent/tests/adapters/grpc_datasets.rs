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

//! DatasetGrpc: auth, URN parsing, error mapping and paging.

use std::sync::Arc;

use catalog_agent::grpc::api::catalog_agent::dataset_entity_service_server::DatasetEntityService;
use catalog_agent::grpc::api::catalog_agent::{
    CreateDatasetRequest, GetByIdRequest, ListDatasetsRequest, PutDatasetRequest,
};
use catalog_agent::grpc::datasets::DatasetEntityGrpc;
use catalog_agent::services::datasets::MockDatasetServiceTrait;
use common::errors::ResourceError;
use common::paginated_spec::Paginated;
use common::test_utils::grpc::{GrpcRequests, StubTokenValidator, OTHER_TENANT, TENANT};
use tonic::Code;

use crate::support::builders::dataset_dto;
use crate::support::fixtures::urn;

fn grpc(service: MockDatasetServiceTrait) -> DatasetEntityGrpc {
    DatasetEntityGrpc::new(Arc::new(service), Arc::new(StubTokenValidator))
}

fn by_id(id: &str) -> GetByIdRequest {
    GetByIdRequest { id: id.to_string() }
}

/// A call without token is Unauthenticated.
#[tokio::test]
async fn get_without_token_is_unauthenticated() {
    let g = grpc(MockDatasetServiceTrait::new());
    let err = g
        .get_dataset_by_id(GrpcRequests::with_auth(by_id(&urn(1)), None, Some(TENANT)))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::Unauthenticated);
}

/// A non-admin naming another tenant is PermissionDenied.
#[tokio::test]
async fn get_foreign_tenant_without_admin_is_permission_denied() {
    let g = grpc(MockDatasetServiceTrait::new());
    let err = g
        .get_dataset_by_id(GrpcRequests::with_auth(
            by_id(&urn(1)),
            Some("owner"),
            Some(OTHER_TENANT),
        ))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::PermissionDenied);
}

/// A malformed URN is InvalidArgument and the message names the field.
#[tokio::test]
async fn invalid_urns_name_their_field() {
    let g = grpc(MockDatasetServiceTrait::new());
    let err = g
        .get_dataset_by_id(GrpcRequests::owner(by_id("nope")))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(err.message().starts_with("id:"), "{}", err.message());

    let err = g
        .put_dataset_by_id(GrpcRequests::owner(PutDatasetRequest {
            id: "nope".into(),
            ..Default::default()
        }))
        .await
        .unwrap_err();
    assert!(err.message().starts_with("id:"), "{}", err.message());

    let err = g
        .create_dataset(GrpcRequests::owner(CreateDatasetRequest {
            catalog_id: "nope".into(),
            ..Default::default()
        }))
        .await
        .unwrap_err();
    assert!(
        err.message().starts_with("catalog_id:"),
        "{}",
        err.message()
    );
}

/// A not-found from the service becomes NotFound.
#[tokio::test]
async fn domain_not_found_maps_to_not_found() {
    let mut svc = MockDatasetServiceTrait::new();
    svc.expect_get_dataset_by_id()
        .returning(|_, id| Err(ResourceError::not_found(id, "dataset")));
    let g = grpc(svc);
    let err = g
        .get_dataset_by_id(GrpcRequests::owner(by_id(&urn(1))))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::NotFound);
}

/// List parses the filters and page, and returns the service's cursor and total.
#[tokio::test]
async fn list_propagates_cursor_total_and_parsed_filters() {
    let mut svc = MockDatasetServiceTrait::new();
    svc.expect_get_all_datasets()
        .withf(|_, filter, page, sort| {
            filter.conforms_to.as_deref() == Some("spec")
                && filter.catalog_id.is_none()
                && page.limit == 2
                && sort.as_str() == "updated_at_desc"
        })
        .returning(|_, _, _, _| {
            Ok(Paginated::new(
                vec![dataset_dto(1), dataset_dto(2)],
                Some("next".into()),
                Some(10),
            ))
        });
    let g = grpc(svc);
    let req = ListDatasetsRequest {
        conforms_to: "spec".into(),
        limit: 2,
        sort: "updated_at_desc".into(),
        ..Default::default()
    };
    let resp = g
        .get_all_datasets(GrpcRequests::owner(req))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(resp.items.len(), 2);
    assert_eq!(resp.next_cursor, "next");
    assert_eq!(resp.total, 10);
    assert_eq!(resp.items[1].dct_title.as_deref(), Some("dataset-2"));
}
