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

//! TransferProcessGrpc: auth, field parsing, enum and identifier mapping, error mapping and
//! response shaping.

use common::test_utils::grpc::{GrpcRequests, StubTokenValidator, TENANT};
use std::sync::Arc;
use transfer_agent::grpc::api::transfer_processes::{
    BatchTransferProcessesRequest, CreateTransferProcessRequest, EditTransferProcessRequest,
    ListTransferProcessesRequest, ResourceIdRequest,
    transfer_processes_ref_server::TransferProcessesRef,
};
use ymir::errors::Errors;

use std::collections::HashMap;

use chrono::Utc;
use prost_types::Struct;
use tonic::Code;

use common::errors::ResourceError;
use common::grpc::JsonValueExt;
use common::query::Paginated;
use transfer_agent::entities::ids::{ParticipantId, TransferProcessId};
use transfer_agent::entities::protocol::{
    ProtocolId, ProtocolState, StateMetadata, TransferCorrelation, TransferRole,
};
use transfer_agent::grpc::api::transfer_processes::{
    ProtocolId as ProtoProtocolId, TransferRole as ProtoTransferRole,
};
use transfer_agent::grpc::transfer_process::TransferProcessGrpc;
use transfer_agent::services::transfer_process::MockTransferProcessServiceTrait;
use transfer_agent::services::transfer_process::views::TransferProcessView;

fn grpc(service: MockTransferProcessServiceTrait) -> TransferProcessGrpc {
    TransferProcessGrpc::new(Arc::new(service), Arc::new(StubTokenValidator))
}

fn view() -> TransferProcessView {
    TransferProcessView {
        id: TransferProcessId::generate(),
        user_id: TENANT.to_string(),
        user_role: common::oauth::RolePath::root(),
        visibility: common::oauth::Visibility::Private,
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

/// A call without token is Unauthenticated.
#[tokio::test]
async fn get_without_token_is_unauthenticated() {
    let g = grpc(MockTransferProcessServiceTrait::new());
    let err = g
        .get_transfer_process(GrpcRequests::with_auth(id_request(), None))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::Unauthenticated);
}

/// A token the validator rejects is Unauthenticated.
#[tokio::test]
async fn get_with_invalid_token_is_unauthenticated() {
    let g = grpc(MockTransferProcessServiceTrait::new());
    let err = g
        .get_transfer_process(GrpcRequests::with_auth(id_request(), Some("bogus")))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::Unauthenticated);
}

/// Without tenant header the caller acts on its token's tenant.
#[tokio::test]
async fn missing_tenant_header_falls_back_to_token_tenant() {
    let mut svc = MockTransferProcessServiceTrait::new();
    svc.expect_get_one()
        .withf(|scope, _| scope.id() == TENANT)
        .returning(|_, _| Ok(view()));
    let g = grpc(svc);
    assert!(
        g.get_transfer_process(GrpcRequests::with_auth(id_request(), Some("user")))
            .await
            .is_ok()
    );
}

/// A malformed id is InvalidArgument on `id`.
#[tokio::test]
async fn get_invalid_urn_is_invalid_argument_naming_field() {
    let g = grpc(MockTransferProcessServiceTrait::new());
    let err = g
        .get_transfer_process(GrpcRequests::owner(ResourceIdRequest {
            id: "not a urn".into(),
        }))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(err.message().starts_with("id:"), "{}", err.message());
}

/// A malformed id in a batch is reported with its index.
#[tokio::test]
async fn batch_invalid_urn_reports_index() {
    let g = grpc(MockTransferProcessServiceTrait::new());
    let err = g
        .batch_get_transfer_processes(GrpcRequests::owner(BatchTransferProcessesRequest {
            ids: vec!["urn:transfer-process:1".into(), "nope".into()],
        }))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(err.message().starts_with("ids[1]:"), "{}", err.message());
}

/// An unknown role or sort, or a non-RFC 3339 date, is InvalidArgument naming the
/// field.
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
        let err = g
            .list_transfer_processes(GrpcRequests::owner(req))
            .await
            .unwrap_err();
        assert_eq!(err.code(), Code::InvalidArgument);
        assert!(err.message().starts_with(prefix), "{}", err.message());
    }
}

/// An unknown enum value on create is InvalidArgument.
#[tokio::test]
async fn create_rejects_unknown_proto_enum() {
    let g = grpc(MockTransferProcessServiceTrait::new());
    let err = g
        .create_transfer_process(GrpcRequests::owner(CreateTransferProcessRequest {
            role: 42,
            ..valid_create()
        }))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(err.message().starts_with("role:"), "{}", err.message());
}

/// Create maps the proto enums, the properties Struct and the identifiers.
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
        g.create_transfer_process(GrpcRequests::owner(valid_create()))
            .await
            .is_ok()
    );
}

/// Fields absent from an edit are left unchanged.
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
    assert!(
        g.edit_transfer_process(GrpcRequests::owner(req))
            .await
            .is_ok()
    );
}

/// A not-found from the service becomes NotFound.
#[tokio::test]
async fn domain_not_found_maps_to_not_found() {
    let mut svc = MockTransferProcessServiceTrait::new();
    svc.expect_get_one()
        .returning(|_, id| Err(ResourceError::not_found(id, "transfer process")));
    let g = grpc(svc);
    let err = g
        .get_transfer_process(GrpcRequests::owner(id_request()))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::NotFound);
    assert_eq!(err.message(), "transfer process not found");
}

/// A forbidden from the service becomes PermissionDenied.
#[tokio::test]
async fn domain_forbidden_maps_to_permission_denied() {
    let mut svc = MockTransferProcessServiceTrait::new();
    svc.expect_delete()
        .returning(|_, _| Err(Errors::forbidden("read-only role", None)));
    let g = grpc(svc);
    let err = g
        .delete_transfer_process(GrpcRequests::owner(id_request()))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::PermissionDenied);
}

/// List parses the filters and page, and returns the service's cursor and total.
#[tokio::test]
async fn list_propagates_cursor_total_and_parsed_filters() {
    let mut svc = MockTransferProcessServiceTrait::new();
    svc.expect_get_all()
        .withf(|_, filter, page, sort| {
            filter.role == Some(TransferRole::Relay)
                && filter.protocol == Some(ProtocolId::Dsp2024)
                && filter.user_id.is_none()
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
        .list_transfer_processes(GrpcRequests::owner(req))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(resp.items.len(), 2);
    assert_eq!(resp.next_cursor, "next");
    assert_eq!(resp.total, 42);
}

/// Get returns the process with its properties as a Struct.
#[tokio::test]
async fn get_serializes_view_with_struct_properties() {
    let mut svc = MockTransferProcessServiceTrait::new();
    svc.expect_get_one().returning(|_, _| Ok(view()));
    let g = grpc(svc);
    let resp = g
        .get_transfer_process(GrpcRequests::owner(id_request()))
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

/// Batch returns every requested process, with total and no cursor.
#[tokio::test]
async fn batch_returns_full_set_without_cursor() {
    let mut svc = MockTransferProcessServiceTrait::new();
    svc.expect_batch()
        .withf(|_, batch| batch.ids.len() == 2)
        .returning(|_, _| Ok(vec![view(), view()]));
    let g = grpc(svc);
    let resp = g
        .batch_get_transfer_processes(GrpcRequests::owner(BatchTransferProcessesRequest {
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
