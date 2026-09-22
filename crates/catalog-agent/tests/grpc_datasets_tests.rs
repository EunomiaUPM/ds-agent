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

//! gRPC adapter tests for the dataset service.

mod grpc_fixtures;

use std::sync::Arc;

use catalog_agent::data::entities::dataset;
use catalog_agent::entities::datasets::{DatasetDto, MockDatasetEntityTrait};
use catalog_agent::grpc::api::catalog_agent::dataset_entity_service_server::DatasetEntityService;
use catalog_agent::grpc::api::catalog_agent::{
    CreateDatasetRequest, GetByIdRequest, ListDatasetsRequest, PutDatasetRequest,
};
use catalog_agent::grpc::datasets::DatasetEntityGrpc;
use chrono::Utc;
use common::errors::ResourceError;
use common::paginated_spec::Paginated;
use grpc_fixtures::{owner, request, urn, StubValidator, OTHER_TENANT, TENANT};
use tonic::Code;

fn grpc(service: MockDatasetEntityTrait) -> DatasetEntityGrpc {
    DatasetEntityGrpc::new(Arc::new(service), Arc::new(StubValidator))
}

fn dto(n: u32) -> DatasetDto {
    DatasetDto {
        inner: dataset::Model {
            id: urn(n),
            tenant_id: TENANT.to_string(),
            dct_conforms_to: None,
            dct_creator: None,
            dct_identifier: None,
            dct_issued: Utc::now().into(),
            dct_modified: None,
            dct_title: Some(format!("dataset-{n}")),
            dct_description: None,
            catalog_id: urn(100),
        },
    }
}

fn by_id(id: &str) -> GetByIdRequest {
    GetByIdRequest { id: id.to_string() }
}

#[tokio::test]
async fn get_without_token_is_unauthenticated() {
    let g = grpc(MockDatasetEntityTrait::new());
    let err = g
        .get_dataset_by_id(request(by_id(&urn(1)), None, Some(TENANT)))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::Unauthenticated);
}

#[tokio::test]
async fn get_foreign_tenant_without_admin_is_permission_denied() {
    let g = grpc(MockDatasetEntityTrait::new());
    let err = g
        .get_dataset_by_id(request(by_id(&urn(1)), Some("owner"), Some(OTHER_TENANT)))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::PermissionDenied);
}

#[tokio::test]
async fn invalid_urns_name_their_field() {
    let g = grpc(MockDatasetEntityTrait::new());
    let err = g.get_dataset_by_id(owner(by_id("nope"))).await.unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(err.message().starts_with("id:"), "{}", err.message());

    let err = g
        .put_dataset_by_id(owner(PutDatasetRequest {
            id: "nope".into(),
            ..Default::default()
        }))
        .await
        .unwrap_err();
    assert!(err.message().starts_with("id:"), "{}", err.message());

    let err = g
        .create_dataset(owner(CreateDatasetRequest {
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

#[tokio::test]
async fn domain_not_found_maps_to_not_found() {
    let mut svc = MockDatasetEntityTrait::new();
    svc.expect_get_dataset_by_id()
        .returning(|_, id| Err(ResourceError::not_found(id, "dataset")));
    let g = grpc(svc);
    let err = g
        .get_dataset_by_id(owner(by_id(&urn(1))))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::NotFound);
}

#[tokio::test]
async fn list_propagates_cursor_total_and_parsed_filters() {
    let mut svc = MockDatasetEntityTrait::new();
    svc.expect_get_all_datasets()
        .withf(|_, filter, page, sort| {
            filter.conforms_to.as_deref() == Some("spec")
                && filter.catalog_id.is_none()
                && page.limit == 2
                && sort.as_str() == "updated_at_desc"
        })
        .returning(|_, _, _, _| {
            Ok(Paginated::new(
                vec![dto(1), dto(2)],
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
    let resp = g.get_all_datasets(owner(req)).await.unwrap().into_inner();
    assert_eq!(resp.items.len(), 2);
    assert_eq!(resp.next_cursor, "next");
    assert_eq!(resp.total, 10);
    assert_eq!(resp.items[1].dct_title.as_deref(), Some("dataset-2"));
}
