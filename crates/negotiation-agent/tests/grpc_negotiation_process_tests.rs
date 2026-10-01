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

//! gRPC adapter tests for negotiation processes: auth, parsing, error mapping, nesting, paging.

mod grpc_fixtures;

use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use common::errors::ResourceError;
use common::grpc::JsonValueExt;
use common::paginated_spec::Paginated;
use grpc_fixtures::{
    OTHER_TENANT, StubValidator, TENANT, agreement_row, message_row, offer_row, owner, request, urn,
};
use negotiation_agent::data::entities::negotiation_process;
use negotiation_agent::grpc::api::negotiation_agent::negotiation_agent_processes_service_server::NegotiationAgentProcessesService;
use negotiation_agent::grpc::api::negotiation_agent::{
    CreateNegotiationProcessRequest, DeleteNegotiationProcessRequest,
    GetBatchNegotiationProcessesRequest, GetNegotiationProcessByIdRequest,
    ListNegotiationProcessesRequest, PutNegotiationProcessRequest,
};
use negotiation_agent::grpc::negotiation_process::NegotiationAgentProcessesGrpc;
use negotiation_agent::services::negotiation_process::MockNegotiationProcessServiceTrait;
use negotiation_agent::services::negotiation_process::views::NegotiationProcessView;
use serde_json::json;
use tonic::Code;
use ymir::errors::Errors;

fn grpc(service: MockNegotiationProcessServiceTrait) -> NegotiationAgentProcessesGrpc {
    NegotiationAgentProcessesGrpc::new(Arc::new(service), Arc::new(StubValidator))
}

fn view(n: u32) -> NegotiationProcessView {
    NegotiationProcessView::assemble(
        negotiation_process::Model {
            id: urn(n),
            tenant_id: TENANT.to_string(),
            state: "REQUESTED".into(),
            state_attribute: None,
            associated_agent_peer: "peer".into(),
            protocol: "dsp".into(),
            callback_address: None,
            role: "provider".into(),
            properties: json!({"k": "v"}),
            error_details: None,
            created_at: Utc::now().into(),
            updated_at: None,
        },
        HashMap::from([("consumerPid".to_string(), "c-1".to_string())]),
        vec![message_row(1), message_row(2)],
        vec![offer_row(3)],
        Some(agreement_row(4)),
    )
}

fn by_id(id: &str) -> GetNegotiationProcessByIdRequest {
    GetNegotiationProcessByIdRequest { id: id.to_string() }
}

// Auth ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn get_without_token_is_unauthenticated() {
    let g = grpc(MockNegotiationProcessServiceTrait::new());
    let err = g
        .get_negotiation_process_by_id(request(by_id(&urn(1)), None, Some(TENANT)))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::Unauthenticated);
}

#[tokio::test]
async fn get_foreign_tenant_without_admin_is_permission_denied() {
    let g = grpc(MockNegotiationProcessServiceTrait::new());
    let err = g
        .get_negotiation_process_by_id(request(by_id(&urn(1)), Some("owner"), Some(OTHER_TENANT)))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::PermissionDenied);
}

#[tokio::test]
async fn admin_may_act_on_foreign_tenant_and_missing_header_falls_back() {
    let mut svc = MockNegotiationProcessServiceTrait::new();
    svc.expect_get_one()
        .withf(|scope, _| scope.acting_tenant() == OTHER_TENANT && scope.is_admin())
        .returning(|_, _| Ok(view(1)));
    svc.expect_get_one()
        .withf(|scope, _| scope.acting_tenant() == TENANT && !scope.is_admin())
        .returning(|_, _| Ok(view(1)));
    let g = grpc(svc);
    assert!(
        g.get_negotiation_process_by_id(request(by_id(&urn(1)), Some("admin"), Some(OTHER_TENANT)))
            .await
            .is_ok()
    );
    assert!(
        g.get_negotiation_process_by_id(request(by_id(&urn(1)), Some("owner"), None))
            .await
            .is_ok()
    );
}

// Field parsing ────────────────────────────────────────────────────────────

#[tokio::test]
async fn invalid_urns_name_their_field() {
    let g = grpc(MockNegotiationProcessServiceTrait::new());
    let err = g
        .get_negotiation_process_by_id(owner(by_id("nope")))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(err.message().starts_with("id:"), "{}", err.message());

    let err = g
        .get_batch_negotiation_processes(owner(GetBatchNegotiationProcessesRequest {
            ids: vec![urn(1), "nope".into()],
        }))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(err.message().starts_with("ids[1]:"), "{}", err.message());

    let err = g
        .create_negotiation_process(owner(CreateNegotiationProcessRequest {
            id: Some("nope".into()),
            ..Default::default()
        }))
        .await
        .unwrap_err();
    assert!(err.message().starts_with("id:"), "{}", err.message());
}

#[tokio::test]
async fn list_rejects_bad_sort_and_date() {
    let g = grpc(MockNegotiationProcessServiceTrait::new());
    for (req, prefix) in [
        (
            ListNegotiationProcessesRequest {
                sort: "sideways".into(),
                ..Default::default()
            },
            "sort:",
        ),
        (
            ListNegotiationProcessesRequest {
                created_after: "yesterday".into(),
                ..Default::default()
            },
            "created_after:",
        ),
    ] {
        let err = g
            .get_all_negotiation_processes(owner(req))
            .await
            .unwrap_err();
        assert_eq!(err.code(), Code::InvalidArgument);
        assert!(err.message().starts_with(prefix), "{}", err.message());
    }
}

#[tokio::test]
async fn create_maps_struct_and_identifiers() {
    let mut svc = MockNegotiationProcessServiceTrait::new();
    svc.expect_create()
        .withf(|_, dto| {
            dto.id.is_none()
                && dto.role == "consumer"
                && dto.properties == Some(json!({"a": 1}))
                && dto.identifiers.as_ref().and_then(|m| m.get("consumerPid"))
                    == Some(&"c-1".to_string())
        })
        .returning(|_, _| Ok(view(1)));
    let g = grpc(svc);
    let req = CreateNegotiationProcessRequest {
        role: "consumer".into(),
        properties: Some(json!({"a": 1}).into_prost_struct()),
        identifiers: HashMap::from([("consumerPid".to_string(), "c-1".to_string())]),
        ..Default::default()
    };
    assert!(g.create_negotiation_process(owner(req)).await.is_ok());
}

#[tokio::test]
async fn put_treats_absent_fields_as_no_change() {
    let mut svc = MockNegotiationProcessServiceTrait::new();
    svc.expect_edit()
        .withf(|_, id, dto| {
            id.to_string() == urn(1)
                && dto.state.as_deref() == Some("STARTED")
                && dto.properties.is_none()
                && dto.error_details == Some(json!({}))
                && dto.identifiers.is_none()
        })
        .returning(|_, _, _| Ok(view(1)));
    let g = grpc(svc);
    let req = PutNegotiationProcessRequest {
        id: urn(1),
        state: Some("STARTED".into()),
        error_details: Some(prost_types::Struct::default()),
        ..Default::default()
    };
    assert!(g.put_negotiation_process(owner(req)).await.is_ok());
}

// Error mapping ────────────────────────────────────────────────────────────

#[tokio::test]
async fn domain_errors_map_to_grpc_codes() {
    let mut svc = MockNegotiationProcessServiceTrait::new();
    svc.expect_get_one()
        .returning(|_, id| Err(ResourceError::not_found(id, "negotiation process")));
    svc.expect_delete()
        .returning(|_, _| Err(Errors::forbidden("read-only", None)));
    let g = grpc(svc);
    let err = g
        .get_negotiation_process_by_id(owner(by_id(&urn(1))))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::NotFound);
    assert_eq!(err.message(), "negotiation process not found");

    let err = g
        .delete_negotiation_process(owner(DeleteNegotiationProcessRequest { id: urn(1) }))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::PermissionDenied);
}

// Response shaping ─────────────────────────────────────────────────────────

#[tokio::test]
async fn get_nests_messages_offers_and_agreement_from_the_view() {
    let mut svc = MockNegotiationProcessServiceTrait::new();
    svc.expect_get_one().returning(|_, _| Ok(view(1)));
    let g = grpc(svc);
    let resp = g
        .get_negotiation_process_by_id(owner(by_id(&urn(1))))
        .await
        .unwrap()
        .into_inner();
    let process = resp.process.unwrap();
    assert_eq!(process.id, urn(1));
    assert_eq!(process.identifiers["consumerPid"], "c-1");
    assert_eq!(process.messages.len(), 2);
    assert!(process.messages[0].payload.is_some());
    assert!(process.messages[0].offer.is_none());
    assert_eq!(process.offers.len(), 1);
    assert_eq!(process.offers[0].offer_id, "offer-3");
    assert_eq!(process.agreement.as_ref().unwrap().state, "ACTIVE");
    assert_eq!(
        process.properties.unwrap().fields["k"].kind,
        Some(prost_types::value::Kind::StringValue("v".into()))
    );
    assert!(process.error_details.is_none());
}

#[tokio::test]
async fn list_propagates_cursor_total_and_parsed_filters() {
    let mut svc = MockNegotiationProcessServiceTrait::new();
    svc.expect_get_all()
        .withf(|_, filter, page, sort| {
            filter.state.as_deref() == Some("REQUESTED")
                && filter.role.is_none()
                && filter.tenant_id.is_none()
                && page.limit == 5
                && page.cursor.as_deref() == Some("abc")
                && sort.as_str() == "updated_at_asc"
        })
        .returning(|_, _, _, _| {
            Ok(Paginated::new(
                vec![view(1), view(2)],
                Some("next".into()),
                Some(42),
            ))
        });
    let g = grpc(svc);
    let req = ListNegotiationProcessesRequest {
        state: "REQUESTED".into(),
        limit: 5,
        cursor: "abc".into(),
        sort: "updated_at_asc".into(),
        ..Default::default()
    };
    let resp = g
        .get_all_negotiation_processes(owner(req))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(resp.items.len(), 2);
    assert_eq!(resp.next_cursor, "next");
    assert_eq!(resp.total, 42);
}

#[tokio::test]
async fn batch_returns_full_set_without_cursor() {
    let mut svc = MockNegotiationProcessServiceTrait::new();
    svc.expect_batch()
        .withf(|_, batch| batch.ids.len() == 2)
        .returning(|_, _| Ok(vec![view(1), view(2)]));
    let g = grpc(svc);
    let resp = g
        .get_batch_negotiation_processes(owner(GetBatchNegotiationProcessesRequest {
            ids: vec![urn(1), urn(2)],
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(resp.items.len(), 2);
    assert_eq!(resp.total, 2);
    assert!(resp.next_cursor.is_empty());
}
