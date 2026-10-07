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

//! OdrlPolicyGrpc: entity type enum mapping, Struct payloads and paging.

use std::sync::Arc;

use catalog_agent::entities::odrl_policies::CatalogEntityTypes;
use catalog_agent::grpc::api::catalog_agent::odrl_policy_entity_service_server::OdrlPolicyEntityService;
use catalog_agent::grpc::api::catalog_agent::{
    CatalogEntityType, CreateOdrlPolicyRequest, GetByEntityIdRequest, GetByIdRequest,
    ListOdrlPoliciesRequest,
};
use catalog_agent::grpc::odrl_policies::OdrlPolicyEntityGrpc;
use catalog_agent::services::odrl_policies::MockOdrlPolicyServiceTrait;
use common::errors::ResourceError;
use common::grpc::JsonValueExt;
use common::paginated_spec::Paginated;
use common::test_utils::grpc::{GrpcRequests, StubTokenValidator};
use prost_types::Struct;
use serde_json::json;
use tonic::Code;

use crate::support::builders::odrl_policy_dto;
use crate::support::fixtures::urn;

fn grpc(service: MockOdrlPolicyServiceTrait) -> OdrlPolicyEntityGrpc {
    OdrlPolicyEntityGrpc::new(Arc::new(service), Arc::new(StubTokenValidator))
}

fn by_id(id: &str) -> GetByIdRequest {
    GetByIdRequest { id: id.to_string() }
}

fn valid_create() -> CreateOdrlPolicyRequest {
    CreateOdrlPolicyRequest {
        id: None,
        odrl_offer: Some(json!({"permission": [{"action": "use"}]}).into_prost_struct()),
        entity_id: urn(100),
        entity_type: CatalogEntityType::Dataset as i32,
        source_template_id: Some("tpl".into()),
        source_template_version: Some("1".into()),
        instantiation_parameters: Some(json!({"n": 3}).into_prost_struct()),
        description: None,
    }
}

/// A call without token is Unauthenticated.
#[tokio::test]
async fn get_without_token_is_unauthenticated() {
    let g = grpc(MockOdrlPolicyServiceTrait::new());
    let err = g
        .get_odrl_offer_by_id(GrpcRequests::with_auth(by_id(&urn(1)), None))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::Unauthenticated);
}

/// A malformed URN is InvalidArgument and the message names the field.
#[tokio::test]
async fn invalid_urns_name_their_field() {
    let g = grpc(MockOdrlPolicyServiceTrait::new());
    let err = g
        .get_odrl_offer_by_id(GrpcRequests::owner(by_id("nope")))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(err.message().starts_with("id:"), "{}", err.message());

    let err = g
        .get_all_odrl_offers_by_entity(GrpcRequests::owner(GetByEntityIdRequest {
            entity_id: "nope".into(),
        }))
        .await
        .unwrap_err();
    assert!(err.message().starts_with("entity_id:"), "{}", err.message());
}

/// A missing, empty or malformed offer, or an unspecified or unknown entity type, is
/// InvalidArgument naming the field.
#[tokio::test]
async fn create_rejects_missing_empty_or_malformed_offer_and_unspecified_type() {
    let g = grpc(MockOdrlPolicyServiceTrait::new());
    let cases = [
        (
            CreateOdrlPolicyRequest {
                odrl_offer: None,
                ..valid_create()
            },
            "odrl_offer: is required",
        ),
        (
            CreateOdrlPolicyRequest {
                odrl_offer: Some(Struct::default()),
                ..valid_create()
            },
            "odrl_offer: cannot be empty",
        ),
        (
            CreateOdrlPolicyRequest {
                odrl_offer: Some(json!({"permission": "not-a-list"}).into_prost_struct()),
                ..valid_create()
            },
            "odrl_offer:",
        ),
        (
            CreateOdrlPolicyRequest {
                entity_type: CatalogEntityType::Unspecified as i32,
                ..valid_create()
            },
            "entity_type: must be specified",
        ),
        (
            CreateOdrlPolicyRequest {
                entity_type: 99,
                ..valid_create()
            },
            "entity_type: unknown enum value 99",
        ),
    ];
    for (req, prefix) in cases {
        let err = g
            .create_odrl_offer(GrpcRequests::owner(req))
            .await
            .unwrap_err();
        assert_eq!(err.code(), Code::InvalidArgument);
        assert!(err.message().starts_with(prefix), "{}", err.message());
    }
}

/// The source template, its version and parameters reach the service on create.
#[tokio::test]
async fn create_passes_template_provenance_through() {
    let mut svc = MockOdrlPolicyServiceTrait::new();
    svc.expect_create_odrl_offer()
        .withf(|_, dto| {
            dto.entity_type == CatalogEntityTypes::Dataset
                && dto.source_template_id.as_deref() == Some("tpl")
                && dto.source_template_version.as_deref() == Some("1")
                && dto.instantiation_parameters == Some(json!({"n": 3}))
                && dto.odrl_offer.permission.as_ref().map(|p| p.len()) == Some(1)
        })
        .returning(|_, _| Ok(odrl_policy_dto(1, "Dataset")));
    let g = grpc(svc);
    assert!(g
        .create_odrl_offer(GrpcRequests::owner(valid_create()))
        .await
        .is_ok());
}

/// A not-found from the service becomes NotFound.
#[tokio::test]
async fn domain_not_found_maps_to_not_found() {
    let mut svc = MockOdrlPolicyServiceTrait::new();
    svc.expect_get_odrl_offer_by_id()
        .returning(|_, id| Err(ResourceError::not_found(id, "odrl policy")));
    let g = grpc(svc);
    let err = g
        .get_odrl_offer_by_id(GrpcRequests::owner(by_id(&urn(1))))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::NotFound);
}

/// The entity type filter and paging pass through, and an unknown stored type reads as
/// Unspecified.
#[tokio::test]
async fn list_maps_entity_type_filter_and_propagates_paging() {
    let mut svc = MockOdrlPolicyServiceTrait::new();
    svc.expect_get_all_odrl_offers()
        .withf(|_, filter, page, _| {
            filter.entity_type.as_deref() == Some("Catalog")
                && filter.entity.as_deref() == Some(&urn(100)[..])
                && page.limit == 4
        })
        .returning(|_, _, _, _| {
            Ok(Paginated::new(
                vec![odrl_policy_dto(1, "Catalog"), odrl_policy_dto(2, "garbage")],
                Some("n".into()),
                Some(2),
            ))
        });
    let g = grpc(svc);
    let req = ListOdrlPoliciesRequest {
        entity_id: urn(100),
        entity_type: CatalogEntityType::Catalog as i32,
        limit: 4,
        ..Default::default()
    };
    let resp = g
        .get_all_odrl_offers(GrpcRequests::owner(req))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(resp.next_cursor, "n");
    assert_eq!(resp.total, 2);
    assert_eq!(resp.items[0].entity_type, CatalogEntityType::Catalog as i32);
    assert_eq!(
        resp.items[1].entity_type,
        CatalogEntityType::Unspecified as i32
    );
    let offer = resp.items[0].odrl_offer.as_ref().unwrap();
    assert!(offer.fields.contains_key("permission"));
    let params = resp.items[0].instantiation_parameters.as_ref().unwrap();
    assert_eq!(
        params.fields["n"].kind,
        Some(prost_types::value::Kind::NumberValue(3.0))
    );
}

/// An unspecified entity type means no type filter.
#[tokio::test]
async fn list_with_unspecified_type_means_any() {
    let mut svc = MockOdrlPolicyServiceTrait::new();
    svc.expect_get_all_odrl_offers()
        .withf(|_, filter, _, _| filter.entity_type.is_none())
        .returning(|_, _, _, _| Ok(Paginated::new(vec![], None, Some(0))));
    let g = grpc(svc);
    assert!(g
        .get_all_odrl_offers(GrpcRequests::owner(ListOdrlPoliciesRequest::default()))
        .await
        .is_ok());
}
