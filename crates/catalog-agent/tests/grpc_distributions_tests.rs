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

//! gRPC adapter tests for the distribution service.

mod grpc_fixtures;

use std::sync::Arc;

use catalog_agent::data::entities::distribution;
use catalog_agent::entities::distributions::{DistributionDto, MockDistributionEntityTrait};
use catalog_agent::grpc::api::catalog_agent::distribution_entity_service_server::DistributionEntityService;
use catalog_agent::grpc::api::catalog_agent::{
    CreateDistributionRequest, GetByIdRequest, GetDistributionByFormatRequest,
    ListDistributionsRequest,
};
use catalog_agent::grpc::distributions::DistributionEntityGrpc;
use chrono::Utc;
use common::errors::ResourceError;
use common::paginated_spec::Paginated;
use grpc_fixtures::{owner, request, urn, StubValidator, OTHER_TENANT, TENANT};
use tonic::Code;

fn grpc(service: MockDistributionEntityTrait) -> DistributionEntityGrpc {
    DistributionEntityGrpc::new(Arc::new(service), Arc::new(StubValidator))
}

fn dto(n: u32) -> DistributionDto {
    DistributionDto {
        inner: distribution::Model {
            id: urn(n),
            tenant_id: TENANT.to_string(),
            dct_issued: Utc::now().into(),
            dct_modified: None,
            dct_title: None,
            dct_description: None,
            dcat_access_service: urn(200),
            dataset_id: urn(100),
            dct_format: Some("HTTP_PULL".into()),
        },
    }
}

fn by_id(id: &str) -> GetByIdRequest {
    GetByIdRequest { id: id.to_string() }
}

#[tokio::test]
async fn get_without_token_is_unauthenticated() {
    let g = grpc(MockDistributionEntityTrait::new());
    let err = g
        .get_distribution_by_id(request(by_id(&urn(1)), None, Some(TENANT)))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::Unauthenticated);
}

#[tokio::test]
async fn get_foreign_tenant_without_admin_is_permission_denied() {
    let g = grpc(MockDistributionEntityTrait::new());
    let err = g
        .get_distribution_by_id(request(by_id(&urn(1)), Some("owner"), Some(OTHER_TENANT)))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::PermissionDenied);
}

#[tokio::test]
async fn invalid_urns_name_their_field() {
    let g = grpc(MockDistributionEntityTrait::new());
    let err = g
        .get_distribution_by_id(owner(by_id("nope")))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(err.message().starts_with("id:"), "{}", err.message());

    let err = g
        .get_distribution_by_dataset_and_format(owner(GetDistributionByFormatRequest {
            dataset_id: "nope".into(),
            dct_formats: "HTTP_PULL".into(),
        }))
        .await
        .unwrap_err();
    assert!(
        err.message().starts_with("dataset_id:"),
        "{}",
        err.message()
    );
}

#[tokio::test]
async fn create_maps_empty_format_to_none() {
    let mut svc = MockDistributionEntityTrait::new();
    svc.expect_create_distribution()
        .withf(|_, dto| dto.dct_formats.is_none() && dto.dataset_id.to_string() == urn(100))
        .returning(|_, _| Ok(dto(1)));
    let g = grpc(svc);
    let req = CreateDistributionRequest {
        dataset_id: urn(100),
        dcat_access_service: urn(200),
        ..Default::default()
    };
    assert!(g.create_distribution(owner(req)).await.is_ok());
}

#[tokio::test]
async fn domain_not_found_maps_to_not_found() {
    let mut svc = MockDistributionEntityTrait::new();
    svc.expect_get_distribution_by_id()
        .returning(|_, id| Err(ResourceError::not_found(id, "distribution")));
    let g = grpc(svc);
    let err = g
        .get_distribution_by_id(owner(by_id(&urn(1))))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::NotFound);
}

#[tokio::test]
async fn list_propagates_cursor_total_and_parsed_filters() {
    let mut svc = MockDistributionEntityTrait::new();
    svc.expect_get_all_distributions()
        .withf(|_, filter, page, _| {
            filter.format.as_deref() == Some("HTTP_PULL")
                && filter.dataset_id.as_deref() == Some(&urn(100)[..])
                && page.cursor.as_deref() == Some("c0")
        })
        .returning(|_, _, _, _| Ok(Paginated::new(vec![dto(1)], None, Some(1))));
    let g = grpc(svc);
    let req = ListDistributionsRequest {
        format: "HTTP_PULL".into(),
        dataset_id: urn(100),
        cursor: "c0".into(),
        ..Default::default()
    };
    let resp = g
        .get_all_distributions(owner(req))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(resp.items.len(), 1);
    assert!(resp.next_cursor.is_empty());
    assert_eq!(resp.total, 1);
    assert_eq!(resp.items[0].dct_format.as_deref(), Some("HTTP_PULL"));
}
