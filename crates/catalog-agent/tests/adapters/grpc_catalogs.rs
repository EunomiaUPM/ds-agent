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

//! CatalogGrpc: auth, field parsing, error mapping and paging.

use std::sync::Arc;

use catalog_agent::grpc::api::catalog_agent::catalog_entity_service_server::CatalogEntityService;
use catalog_agent::grpc::api::catalog_agent::{
    CreateCatalogRequest, GetBatchRequest, GetByIdRequest, ListCatalogsRequest, PutCatalogRequest,
};
use catalog_agent::grpc::catalogs::CatalogEntityGrpc;
use catalog_agent::services::catalogs::MockCatalogServiceTrait;
use common::errors::ResourceError;
use common::paginated_spec::Paginated;
use common::test_utils::grpc::{GrpcRequests, StubTokenValidator, TENANT};
use tonic::Code;
use ymir::errors::Errors;

use crate::support::builders::catalog_dto;
use crate::support::fixtures::urn;

fn grpc(service: MockCatalogServiceTrait) -> CatalogEntityGrpc {
    CatalogEntityGrpc::new(Arc::new(service), Arc::new(StubTokenValidator))
}

fn by_id(id: &str) -> GetByIdRequest {
    GetByIdRequest { id: id.to_string() }
}

/// A call without token is Unauthenticated.
#[tokio::test]
async fn get_without_token_is_unauthenticated() {
    let g = grpc(MockCatalogServiceTrait::new());
    let err = g
        .get_catalog_by_id(GrpcRequests::with_auth(by_id(&urn(1)), None))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::Unauthenticated);
}

/// Without tenant header the caller acts on its token's tenant.
#[tokio::test]
async fn missing_tenant_header_falls_back_to_token_tenant() {
    let mut svc = MockCatalogServiceTrait::new();
    svc.expect_get_catalog_by_id()
        .withf(|scope, _| scope.id() == TENANT)
        .returning(|_, _| Ok(catalog_dto(1)));
    let g = grpc(svc);
    assert!(g
        .get_catalog_by_id(GrpcRequests::with_auth(by_id(&urn(1)), Some("user")))
        .await
        .is_ok());
}

/// A malformed id is InvalidArgument on `id`.
#[tokio::test]
async fn get_invalid_urn_is_invalid_argument_naming_field() {
    let g = grpc(MockCatalogServiceTrait::new());
    let err = g
        .get_catalog_by_id(GrpcRequests::owner(by_id("not a urn")))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(err.message().starts_with("id:"), "{}", err.message());
}

/// A malformed id in a batch is reported with its index.
#[tokio::test]
async fn batch_invalid_urn_reports_index() {
    let g = grpc(MockCatalogServiceTrait::new());
    let err = g
        .get_batch_catalogs(GrpcRequests::owner(GetBatchRequest {
            ids: vec![urn(1), "nope".into()],
        }))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(err.message().starts_with("ids[1]:"), "{}", err.message());
}

/// An unknown sort or a non-RFC 3339 date is InvalidArgument naming the field.
#[tokio::test]
async fn list_rejects_bad_sort_and_date() {
    let g = grpc(MockCatalogServiceTrait::new());
    let err = g
        .get_all_catalogs(GrpcRequests::owner(ListCatalogsRequest {
            sort: "sideways".into(),
            ..Default::default()
        }))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(err.message().starts_with("sort:"), "{}", err.message());

    let err = g
        .get_all_catalogs(GrpcRequests::owner(ListCatalogsRequest {
            created_before: "yesterday".into(),
            ..Default::default()
        }))
        .await
        .unwrap_err();
    assert!(
        err.message().starts_with("created_before:"),
        "{}",
        err.message()
    );
}

/// Create parses the optional id and passes the fields; a malformed id names `id`.
#[tokio::test]
async fn create_parses_optional_id_and_passes_fields() {
    let mut svc = MockCatalogServiceTrait::new();
    svc.expect_create_catalog()
        .withf(|_, dto| {
            dto.id.as_ref().map(|u| u.to_string()) == Some(urn(9))
                && dto.dct_title.as_deref() == Some("t")
                && dto.owner.is_none()
        })
        .returning(|_, _| Ok(catalog_dto(9)));
    let g = grpc(svc);
    let req = CreateCatalogRequest {
        id: Some(urn(9)),
        dct_title: Some("t".into()),
        ..Default::default()
    };
    assert!(g.create_catalog(GrpcRequests::owner(req)).await.is_ok());

    let g = grpc(MockCatalogServiceTrait::new());
    let err = g
        .create_catalog(GrpcRequests::owner(CreateCatalogRequest {
            id: Some("bad".into()),
            ..Default::default()
        }))
        .await
        .unwrap_err();
    assert!(err.message().starts_with("id:"), "{}", err.message());
}

/// Put parses the id and reaches the service as the edit DTO.
#[tokio::test]
async fn put_parses_id_and_maps_edit_dto() {
    let mut svc = MockCatalogServiceTrait::new();
    svc.expect_put_catalog_by_id()
        .withf(|_, id, edit| id.to_string() == urn(1) && edit.dct_title.as_deref() == Some("new"))
        .returning(|_, _, _| Ok(catalog_dto(1)));
    let g = grpc(svc);
    let req = PutCatalogRequest {
        id: urn(1),
        dct_title: Some("new".into()),
        ..Default::default()
    };
    assert!(g.put_catalog_by_id(GrpcRequests::owner(req)).await.is_ok());
}

/// A not-found from the service becomes NotFound.
#[tokio::test]
async fn domain_not_found_maps_to_not_found() {
    let mut svc = MockCatalogServiceTrait::new();
    svc.expect_get_catalog_by_id()
        .returning(|_, id| Err(ResourceError::not_found(id, "catalog")));
    let g = grpc(svc);
    let err = g
        .get_catalog_by_id(GrpcRequests::owner(by_id(&urn(1))))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::NotFound);
    assert_eq!(err.message(), "catalog not found");
}

/// A forbidden from the service becomes PermissionDenied.
#[tokio::test]
async fn domain_forbidden_maps_to_permission_denied() {
    let mut svc = MockCatalogServiceTrait::new();
    svc.expect_delete_catalog_by_id()
        .returning(|_, _| Err(Errors::forbidden("read-only", None)));
    let g = grpc(svc);
    let err = g
        .delete_catalog_by_id(GrpcRequests::owner(
            catalog_agent::grpc::api::catalog_agent::DeleteByIdRequest { id: urn(1) },
        ))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::PermissionDenied);
}

/// A tenant without main catalog gets NotFound.
#[tokio::test]
async fn missing_main_catalog_is_not_found() {
    let mut svc = MockCatalogServiceTrait::new();
    svc.expect_get_main_catalog().returning(|_| Ok(None));
    let g = grpc(svc);
    let err = g
        .get_main_catalog(GrpcRequests::owner(()))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::NotFound);
}

/// List parses the filters and page, and returns the service's cursor and total.
#[tokio::test]
async fn list_propagates_cursor_total_and_parsed_filters() {
    let mut svc = MockCatalogServiceTrait::new();
    svc.expect_get_all_catalogs()
        .withf(|_, filter, page, sort| {
            filter.title.as_deref() == Some("x")
                && filter.with_main_catalog == Some(true)
                && filter.creator.is_none()
                && filter.user_id.is_none()
                && page.limit == 5
                && page.cursor.as_deref() == Some("abc")
                && sort.as_str() == "created_at_asc"
        })
        .returning(|_, _, _, _| {
            Ok(Paginated::new(
                vec![catalog_dto(1), catalog_dto(2)],
                Some("next".into()),
                Some(42),
            ))
        });
    let g = grpc(svc);
    let req = ListCatalogsRequest {
        title: "x".into(),
        with_main_catalog: Some(true),
        limit: 5,
        cursor: "abc".into(),
        sort: "created_at_asc".into(),
        ..Default::default()
    };
    let resp = g
        .get_all_catalogs(GrpcRequests::owner(req))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(resp.items.len(), 2);
    assert_eq!(resp.next_cursor, "next");
    assert_eq!(resp.total, 42);
    assert!(resp.items[0].dspace_main_catalog);
    assert_eq!(resp.items[1].dct_title.as_deref(), Some("catalog-2"));
}

/// Batch returns every requested catalog, with total and no cursor.
#[tokio::test]
async fn batch_returns_full_set_without_cursor() {
    let mut svc = MockCatalogServiceTrait::new();
    svc.expect_get_batch_catalogs()
        .withf(|_, ids| ids.len() == 2)
        .returning(|_, _| Ok(vec![catalog_dto(1), catalog_dto(2)]));
    let g = grpc(svc);
    let resp = g
        .get_batch_catalogs(GrpcRequests::owner(GetBatchRequest {
            ids: vec![urn(1), urn(2)],
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(resp.items.len(), 2);
    assert_eq!(resp.total, 2);
    assert!(resp.next_cursor.is_empty());
}
