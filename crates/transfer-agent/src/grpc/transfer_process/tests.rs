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

//! gRPC adapter tests: auth, field parsing, error mapping and response shaping.

use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use prost_types::Struct;
use tonic::{Code, Request};
use ymir::errors::{Errors, Outcome};

use crate::entities::ids::{ParticipantId, TransferProcessId};
use crate::entities::protocol::{
    ProtocolId, ProtocolState, StateMetadata, TransferCorrelation, TransferRole,
};
use crate::grpc::api::transfer_processes::{
    BatchTransferProcessesRequest, CreateTransferProcessRequest, EditTransferProcessRequest,
    ListTransferProcessesRequest, ProtocolId as ProtoProtocolId, ResourceIdRequest,
    TransferRole as ProtoTransferRole, transfer_processes_ref_server::TransferProcessesRef,
};
use crate::grpc::transfer_process::TransferProcessGrpc;
use crate::services::transfer_process::MockTransferProcessServiceTrait;
use crate::services::transfer_process::views::TransferProcessView;
use common::auth::claims::RbacRole;
use common::auth::{Claims, OauthTokenValidator};
use common::errors::ResourceError;
use common::grpc::JsonValueExt;
use common::query::Paginated;

const TENANT: &str = "tenant-1";
const OTHER_TENANT: &str = "tenant-2";

/// Token validator keyed by literal token: `owner` → tenant-1 owner, `admin` → admin.
struct StubValidator;

#[async_trait::async_trait]
impl OauthTokenValidator for StubValidator {
    async fn validate_token(&self, token: &str) -> Outcome<Claims> {
        let role = match token {
            "owner" => RbacRole::Owner,
            "admin" => RbacRole::Admin,
            _ => return Err(Errors::unauthorized("invalid token", None)),
        };
        Ok(Claims {
            sub: TENANT.to_string(),
            role,
            iat: 1000,
            exp: 9_999_999_999,
        })
    }
}

fn grpc(service: MockTransferProcessServiceTrait) -> TransferProcessGrpc {
    TransferProcessGrpc::new(Arc::new(service), Arc::new(StubValidator))
}

fn request<T>(body: T, token: Option<&str>, tenant: Option<&str>) -> Request<T> {
    let mut req = Request::new(body);
    if let Some(t) = token {
        req.metadata_mut()
            .insert("authorization", format!("Bearer {t}").parse().unwrap());
    }
    if let Some(t) = tenant {
        req.metadata_mut().insert("x-tenant-id", t.parse().unwrap());
    }
    req
}

fn owner<T>(body: T) -> Request<T> {
    request(body, Some("owner"), Some(TENANT))
}

fn view() -> TransferProcessView {
    TransferProcessView {
        id: TransferProcessId::generate(),
        tenant_id: TENANT.to_string(),
        role: TransferRole::Provider,
        protocol: ProtocolId::Dsp2025_1,
        state: ProtocolState("REQUESTED".into()),
        state_metadata: StateMetadata::empty(),
        correlation: TransferCorrelation::empty(),
        properties: serde_json::json!({"k": "v", "n": 3}),
        error_details: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        version: 1,
    }
}

fn id_request() -> ResourceIdRequest {
    ResourceIdRequest {
        id: "urn:transfer-process:1".into(),
    }
}

fn valid_create() -> CreateTransferProcessRequest {
    CreateTransferProcessRequest {
        role: ProtoTransferRole::Consumer as i32,
        protocol: ProtoProtocolId::Dsp20251 as i32,
        initial_state: "REQUESTED".into(),
        agreement_id: "urn:agreement:1".into(),
        peer_participant_id: "urn:participant:1".into(),
        callback_address: "https://peer.example/cb".into(),
        identifiers: HashMap::from([("consumerPid".to_string(), "c-1".to_string())]),
        properties: Some(serde_json::json!({"a": 1}).into_prost_struct()),
        initial_state_attribute: String::new(),
        initial_state_code: String::new(),
        initial_state_reasons: vec![],
    }
}

// Auth ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn get_without_token_is_unauthenticated() {
    let g = grpc(MockTransferProcessServiceTrait::new());
    let err = g
        .get_transfer_process(request(id_request(), None, Some(TENANT)))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::Unauthenticated);
}

#[tokio::test]
async fn get_with_invalid_token_is_unauthenticated() {
    let g = grpc(MockTransferProcessServiceTrait::new());
    let err = g
        .get_transfer_process(request(id_request(), Some("bogus"), Some(TENANT)))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::Unauthenticated);
}

#[tokio::test]
async fn get_foreign_tenant_without_admin_is_permission_denied() {
    let g = grpc(MockTransferProcessServiceTrait::new());
    let err = g
        .get_transfer_process(request(id_request(), Some("owner"), Some(OTHER_TENANT)))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::PermissionDenied);
}

#[tokio::test]
async fn admin_may_act_on_foreign_tenant() {
    let mut svc = MockTransferProcessServiceTrait::new();
    svc.expect_get_one()
        .withf(|scope, _| scope.acting_tenant() == OTHER_TENANT && scope.is_admin())
        .returning(|_, _| Ok(view()));
    let g = grpc(svc);
    assert!(
        g.get_transfer_process(request(id_request(), Some("admin"), Some(OTHER_TENANT)))
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn missing_tenant_header_falls_back_to_token_tenant() {
    let mut svc = MockTransferProcessServiceTrait::new();
    svc.expect_get_one()
        .withf(|scope, _| scope.acting_tenant() == TENANT)
        .returning(|_, _| Ok(view()));
    let g = grpc(svc);
    assert!(
        g.get_transfer_process(request(id_request(), Some("owner"), None))
            .await
            .is_ok()
    );
}

// Field parsing ────────────────────────────────────────────────────────────

#[tokio::test]
async fn get_invalid_urn_is_invalid_argument_naming_field() {
    let g = grpc(MockTransferProcessServiceTrait::new());
    let err = g
        .get_transfer_process(owner(ResourceIdRequest {
            id: "not a urn".into(),
        }))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(err.message().starts_with("id:"), "{}", err.message());
}

#[tokio::test]
async fn batch_invalid_urn_reports_index() {
    let g = grpc(MockTransferProcessServiceTrait::new());
    let err = g
        .batch_get_transfer_processes(owner(BatchTransferProcessesRequest {
            ids: vec!["urn:transfer-process:1".into(), "nope".into()],
        }))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(err.message().starts_with("ids[1]:"), "{}", err.message());
}

#[tokio::test]
async fn list_rejects_unknown_role_sort_and_date() {
    let g = grpc(MockTransferProcessServiceTrait::new());
    let cases = [
        (
            ListTransferProcessesRequest {
                role: "broker".into(),
                ..Default::default()
            },
            "role:",
        ),
        (
            ListTransferProcessesRequest {
                sort: "sideways".into(),
                ..Default::default()
            },
            "sort:",
        ),
        (
            ListTransferProcessesRequest {
                created_after: "yesterday".into(),
                ..Default::default()
            },
            "created_after:",
        ),
    ];
    for (req, prefix) in cases {
        let err = g.list_transfer_processes(owner(req)).await.unwrap_err();
        assert_eq!(err.code(), Code::InvalidArgument);
        assert!(err.message().starts_with(prefix), "{}", err.message());
    }
}

#[tokio::test]
async fn create_rejects_unknown_proto_enum() {
    let g = grpc(MockTransferProcessServiceTrait::new());
    let err = g
        .create_transfer_process(owner(CreateTransferProcessRequest {
            role: 42,
            ..valid_create()
        }))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(err.message().starts_with("role:"), "{}", err.message());
}

#[tokio::test]
async fn create_maps_enums_struct_and_identifiers() {
    let mut svc = MockTransferProcessServiceTrait::new();
    svc.expect_create()
        .withf(|_, cmd| {
            cmd.role == TransferRole::Consumer
                && cmd.protocol == ProtocolId::Dsp2025_1
                && cmd.agreement_id.to_string() == "urn:agreement:1"
                && cmd.peer_participant_id
                    == ParticipantId::new("urn:participant:1".parse().unwrap())
                && cmd.callback_address.as_ref().map(|u| u.as_str())
                    == Some("https://peer.example/cb")
                && cmd.identifiers.as_ref().and_then(|m| m.get("consumerPid"))
                    == Some(&"c-1".to_string())
                && cmd.properties == Some(serde_json::json!({"a": 1}))
        })
        .returning(|_, _| Ok(view()));
    let g = grpc(svc);
    assert!(
        g.create_transfer_process(owner(valid_create()))
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn edit_treats_absent_fields_as_no_change() {
    let mut svc = MockTransferProcessServiceTrait::new();
    svc.expect_edit()
        .withf(|_, id, cmd| {
            id.to_string() == "urn:transfer-process:1"
                && cmd.state.as_ref().map(|s| s.0.as_str()) == Some("STARTED")
                && cmd.state_metadata.is_none()
                && cmd.identifiers.is_none()
                && cmd.properties.is_none()
                && cmd.error_details == Some(serde_json::json!({}))
        })
        .returning(|_, _, _| Ok(view()));
    let g = grpc(svc);
    let req = EditTransferProcessRequest {
        id: "urn:transfer-process:1".into(),
        state: "STARTED".into(),
        error_details: Some(Struct::default()),
        ..Default::default()
    };
    assert!(g.edit_transfer_process(owner(req)).await.is_ok());
}

// Error mapping ────────────────────────────────────────────────────────────

#[tokio::test]
async fn domain_not_found_maps_to_not_found() {
    let mut svc = MockTransferProcessServiceTrait::new();
    svc.expect_get_one()
        .returning(|_, id| Err(ResourceError::not_found(id, "transfer process")));
    let g = grpc(svc);
    let err = g
        .get_transfer_process(owner(id_request()))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::NotFound);
    assert_eq!(err.message(), "transfer process not found");
}

#[tokio::test]
async fn domain_forbidden_maps_to_permission_denied() {
    let mut svc = MockTransferProcessServiceTrait::new();
    svc.expect_delete()
        .returning(|_, _| Err(Errors::forbidden("read-only role", None)));
    let g = grpc(svc);
    let err = g
        .delete_transfer_process(owner(id_request()))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::PermissionDenied);
}

// Response shaping ─────────────────────────────────────────────────────────

#[tokio::test]
async fn list_propagates_cursor_total_and_parsed_filters() {
    let mut svc = MockTransferProcessServiceTrait::new();
    svc.expect_get_all()
        .withf(|_, filter, page, sort| {
            filter.role == Some(TransferRole::Relay)
                && filter.protocol == Some(ProtocolId::Dsp2024)
                && filter.tenant_id.is_none()
                && page.limit == 5
                && page.cursor.as_deref() == Some("abc")
                && sort.as_str() == "updated_at_asc"
        })
        .returning(|_, _, _, _| {
            Ok(Paginated {
                items: vec![view(), view()],
                next_cursor: Some("next".into()),
                total: Some(42),
            })
        });
    let g = grpc(svc);
    let req = ListTransferProcessesRequest {
        role: "relay".into(),
        protocol: "dsp2024".into(),
        limit: 5,
        cursor: "abc".into(),
        sort: "updated_at_asc".into(),
        ..Default::default()
    };
    let resp = g
        .list_transfer_processes(owner(req))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(resp.items.len(), 2);
    assert_eq!(resp.next_cursor, "next");
    assert_eq!(resp.total, 42);
}

#[tokio::test]
async fn get_serializes_view_with_struct_properties() {
    let mut svc = MockTransferProcessServiceTrait::new();
    svc.expect_get_one().returning(|_, _| Ok(view()));
    let g = grpc(svc);
    let resp = g
        .get_transfer_process(owner(id_request()))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(resp.role, ProtoTransferRole::Provider as i32);
    assert_eq!(resp.protocol, ProtoProtocolId::Dsp20251 as i32);
    assert_eq!(resp.state, "REQUESTED");
    assert!(resp.error_details.is_none());
    let props = resp.properties.unwrap();
    assert_eq!(
        props.fields["k"].kind,
        Some(prost_types::value::Kind::StringValue("v".into()))
    );
    assert_eq!(
        props.fields["n"].kind,
        Some(prost_types::value::Kind::NumberValue(3.0))
    );
}

#[tokio::test]
async fn batch_returns_full_set_without_cursor() {
    let mut svc = MockTransferProcessServiceTrait::new();
    svc.expect_batch()
        .withf(|_, batch| batch.ids.len() == 2)
        .returning(|_, _| Ok(vec![view(), view()]));
    let g = grpc(svc);
    let resp = g
        .batch_get_transfer_processes(owner(BatchTransferProcessesRequest {
            ids: vec![
                "urn:transfer-process:1".into(),
                "urn:transfer-process:2".into(),
            ],
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(resp.items.len(), 2);
    assert_eq!(resp.total, 2);
    assert!(resp.next_cursor.is_empty());
}
