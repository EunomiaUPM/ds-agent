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

//! gRPC adapter tests for the data-service service.

mod grpc_fixtures;

use std::sync::Arc;

use catalog_agent::data::entities::dataservice;
use catalog_agent::entities::data_services::{DataServiceDto, MockDataServiceEntityTrait};
use catalog_agent::grpc::api::catalog_agent::data_service_entity_service_server::DataServiceEntityService;
use catalog_agent::grpc::api::catalog_agent::{
    CreateDataServiceRequest, GetByIdRequest, GetByParentIdRequest, ListDataServicesRequest,
};
use catalog_agent::grpc::data_services::DataServiceEntityGrpc;
use chrono::Utc;
use common::errors::ResourceError;
use common::paginated_spec::Paginated;
use grpc_fixtures::{owner, request, urn, StubValidator, OTHER_TENANT, TENANT};
use tonic::Code;

fn grpc(service: MockDataServiceEntityTrait) -> DataServiceEntityGrpc {
    DataServiceEntityGrpc::new(Arc::new(service), Arc::new(StubValidator))
}

fn dto(n: u32) -> DataServiceDto {
    DataServiceDto {
        inner: dataservice::Model {
            id: urn(n),
            tenant_id: TENANT.to_string(),
            dcat_endpoint_description: None,
            dcat_endpoint_url: "https://svc.example".into(),
            dct_conforms_to: None,
            dct_creator: None,
            dct_identifier: None,
            dct_issued: Utc::now().into(),
            dct_modified: None,
            dct_title: None,
            dct_description: None,
            catalog_id: urn(100),
            dspace_main_data_service: true,
        },
    }
}

fn by_id(id: &str) -> GetByIdRequest {
    GetByIdRequest { id: id.to_string() }
}

#[tokio::test]
async fn get_without_token_is_unauthenticated() {
    let g = grpc(MockDataServiceEntityTrait::new());
    let err = g
        .get_data_service_by_id(request(by_id(&urn(1)), None, Some(TENANT)))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::Unauthenticated);
}

#[tokio::test]
async fn get_foreign_tenant_without_admin_is_permission_denied() {
    let g = grpc(MockDataServiceEntityTrait::new());
    let err = g
        .get_data_service_by_id(request(by_id(&urn(1)), Some("owner"), Some(OTHER_TENANT)))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::PermissionDenied);
}

#[tokio::test]
async fn invalid_urns_name_their_field() {
    let g = grpc(MockDataServiceEntityTrait::new());
    let err = g
        .get_data_service_by_id(owner(by_id("nope")))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(err.message().starts_with("id:"), "{}", err.message());

    let err = g
        .get_data_services_by_catalog_id(owner(GetByParentIdRequest {
            parent_id: "nope".into(),
        }))
        .await
        .unwrap_err();
    assert!(err.message().starts_with("parent_id:"), "{}", err.message());

    let err = g
        .create_data_service(owner(CreateDataServiceRequest {
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
    let mut svc = MockDataServiceEntityTrait::new();
    svc.expect_get_data_service_by_id()
        .returning(|_, id| Err(ResourceError::not_found(id, "data service")));
    let g = grpc(svc);
    let err = g
        .get_data_service_by_id(owner(by_id(&urn(1))))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::NotFound);
}

#[tokio::test]
async fn list_propagates_cursor_total_and_parsed_filters() {
    let mut svc = MockDataServiceEntityTrait::new();
    svc.expect_get_all_data_services()
        .withf(|_, filter, page, _| {
            filter.catalog_id.as_deref() == Some(&urn(100)[..])
                && filter.main_data_service == Some(false)
                && page.limit == 20
        })
        .returning(|_, _, _, _| Ok(Paginated::new(vec![dto(1)], Some("c".into()), Some(3))));
    let g = grpc(svc);
    let req = ListDataServicesRequest {
        catalog_id: urn(100),
        main_data_service: Some(false),
        ..Default::default()
    };
    let resp = g
        .get_all_data_services(owner(req))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(resp.items.len(), 1);
    assert_eq!(resp.next_cursor, "c");
    assert_eq!(resp.total, 3);
    assert!(resp.items[0].dspace_main_data_service);
}

#[tokio::test]
async fn by_parent_returns_full_set_with_total() {
    let mut svc = MockDataServiceEntityTrait::new();
    svc.expect_get_data_services_by_catalog_id()
        .withf(|_, id| id.to_string() == urn(100))
        .returning(|_, _| Ok(vec![dto(1), dto(2), dto(3)]));
    let g = grpc(svc);
    let resp = g
        .get_data_services_by_catalog_id(owner(GetByParentIdRequest {
            parent_id: urn(100),
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(resp.total, 3);
    assert!(resp.next_cursor.is_empty());
}
