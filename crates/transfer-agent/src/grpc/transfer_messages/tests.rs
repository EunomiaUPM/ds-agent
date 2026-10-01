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

use std::sync::Arc;

use chrono::Utc;
use tonic::{Code, Request};
use ymir::errors::{Errors, Outcome};

use crate::entities::ids::{MessageId, TransferProcessId};
use crate::entities::message_envelope::MessageEnvelope;
use crate::entities::protocol::{ProtocolId, ProtocolMessageType};
use crate::entities::transfer_message::Direction;
use crate::grpc::api::transfer_messages::{
    CreateTransferMessageRequest, Direction as ProtoDirection,
    ListTransferMessagesByProcessRequest, ListTransferMessagesRequest, ResourceIdRequest,
    transfer_messages_ref_server::TransferMessagesRef,
};
use crate::grpc::transfer_messages::TransferMessagesGrpc;
use crate::services::transfer_message::MockTransferMessageServiceTrait;
use crate::services::transfer_message::views::TransferMessageView;
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

fn grpc(service: MockTransferMessageServiceTrait) -> TransferMessagesGrpc {
    TransferMessagesGrpc::new(Arc::new(service), Arc::new(StubValidator))
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

fn view() -> TransferMessageView {
    TransferMessageView {
        id: MessageId::generate(),
        transfer_process_id: TransferProcessId::generate(),
        tenant_id: TENANT.to_string(),
        direction: Direction::Outbound,
        protocol: ProtocolId::Dsp2025_1,
        message_type: ProtocolMessageType("TransferRequestMessage".into()),
        state_transition_from: "".into(),
        state_transition_to: "REQUESTED".into(),
        envelope: MessageEnvelope::from_canonical(
            serde_json::json!({"@type": "TransferRequestMessage"}),
            Some("_:b0 <p> _:b1 .\n".to_string()),
        ),
        occurred_at: Utc::now(),
    }
}

fn id_request() -> ResourceIdRequest {
    ResourceIdRequest {
        id: "urn:transfer-message:1".into(),
    }
}

fn valid_create() -> CreateTransferMessageRequest {
    CreateTransferMessageRequest {
        transfer_process_id: "urn:transfer-process:1".into(),
        direction: ProtoDirection::Inbound as i32,
        protocol: "dsp2025_1".into(),
        message_type: "TransferStartMessage".into(),
        state_transition_from: "REQUESTED".into(),
        state_transition_to: "STARTED".into(),
        payload: Some(serde_json::json!({"@type": "TransferStartMessage"}).into_prost_struct()),
        canonical_form: "_:b0 <p> _:b1 .\n".into(),
    }
}

// Auth ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn get_without_token_is_unauthenticated() {
    let g = grpc(MockTransferMessageServiceTrait::new());
    let err = g
        .get_transfer_message(request(id_request(), None, Some(TENANT)))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::Unauthenticated);
}

#[tokio::test]
async fn get_foreign_tenant_without_admin_is_permission_denied() {
    let g = grpc(MockTransferMessageServiceTrait::new());
    let err = g
        .get_transfer_message(request(id_request(), Some("owner"), Some(OTHER_TENANT)))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::PermissionDenied);
}

#[tokio::test]
async fn missing_tenant_header_falls_back_to_token_tenant() {
    let mut svc = MockTransferMessageServiceTrait::new();
    svc.expect_get_one()
        .withf(|scope, _| scope.acting_tenant() == TENANT)
        .returning(|_, _| Ok(view()));
    let g = grpc(svc);
    assert!(
        g.get_transfer_message(request(id_request(), Some("owner"), None))
            .await
            .is_ok()
    );
}

// Field parsing ────────────────────────────────────────────────────────────

#[tokio::test]
async fn get_invalid_urn_is_invalid_argument_naming_field() {
    let g = grpc(MockTransferMessageServiceTrait::new());
    let err = g
        .get_transfer_message(owner(ResourceIdRequest {
            id: "not a urn".into(),
        }))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(err.message().starts_with("id:"), "{}", err.message());
}

#[tokio::test]
async fn list_by_process_rejects_invalid_process_id_and_direction() {
    let g = grpc(MockTransferMessageServiceTrait::new());
    let err = g
        .list_transfer_messages_by_process(owner(ListTransferMessagesByProcessRequest {
            process_id: "nope".into(),
            ..Default::default()
        }))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(
        err.message().starts_with("process_id:"),
        "{}",
        err.message()
    );

    let err = g
        .list_transfer_messages_by_process(owner(ListTransferMessagesByProcessRequest {
            process_id: "urn:transfer-process:1".into(),
            direction: "sideways".into(),
            ..Default::default()
        }))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(err.message().starts_with("direction:"), "{}", err.message());
}

#[tokio::test]
async fn create_rejects_unknown_direction_and_missing_protocol() {
    let g = grpc(MockTransferMessageServiceTrait::new());
    let err = g
        .create_transfer_message(owner(CreateTransferMessageRequest {
            direction: 9,
            ..valid_create()
        }))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(err.message().starts_with("direction:"), "{}", err.message());

    let err = g
        .create_transfer_message(owner(CreateTransferMessageRequest {
            protocol: String::new(),
            ..valid_create()
        }))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(err.message().starts_with("protocol:"), "{}", err.message());
}

#[tokio::test]
async fn create_builds_envelope_with_hashed_canonical_form() {
    let mut svc = MockTransferMessageServiceTrait::new();
    svc.expect_create()
        .withf(|_, cmd| {
            cmd.direction == Direction::Inbound
                && cmd.protocol == ProtocolId::Dsp2025_1
                && cmd.transfer_process_id.to_string() == "urn:transfer-process:1"
                && cmd.envelope.payload["@type"] == "TransferStartMessage"
                && cmd.envelope.canonical_form.as_deref() == Some("_:b0 <p> _:b1 .\n")
                && cmd.envelope.canonical_hash.is_some()
        })
        .returning(|_, _| Ok(view()));
    let g = grpc(svc);
    assert!(
        g.create_transfer_message(owner(valid_create()))
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn create_without_payload_or_canonical_form_yields_bare_envelope() {
    let mut svc = MockTransferMessageServiceTrait::new();
    svc.expect_create()
        .withf(|_, cmd| {
            cmd.envelope.payload.is_null()
                && cmd.envelope.canonical_form.is_none()
                && cmd.envelope.canonical_hash.is_none()
        })
        .returning(|_, _| Ok(view()));
    let g = grpc(svc);
    let req = CreateTransferMessageRequest {
        payload: None,
        canonical_form: String::new(),
        ..valid_create()
    };
    assert!(g.create_transfer_message(owner(req)).await.is_ok());
}

// Error mapping ────────────────────────────────────────────────────────────

#[tokio::test]
async fn domain_not_found_maps_to_not_found() {
    let mut svc = MockTransferMessageServiceTrait::new();
    svc.expect_get_one()
        .returning(|_, id| Err(ResourceError::not_found(id, "transfer message")));
    let g = grpc(svc);
    let err = g
        .get_transfer_message(owner(id_request()))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::NotFound);
    assert_eq!(err.message(), "transfer message not found");
}

// Response shaping ─────────────────────────────────────────────────────────

#[tokio::test]
async fn list_propagates_cursor_total_and_parsed_filters() {
    let mut svc = MockTransferMessageServiceTrait::new();
    svc.expect_get_all()
        .withf(|_, filter, page, sort| {
            filter.direction == Some(Direction::Inbound)
                && filter.protocol == Some(ProtocolId::Dsp2024)
                && filter.state_transition_to.as_ref().map(|s| s.0.as_str()) == Some("STARTED")
                && page.limit == 20
                && page.cursor.is_none()
                && sort.as_str() == "created_at_desc"
        })
        .returning(|_, _, _, _| {
            Ok(Paginated {
                items: vec![view()],
                next_cursor: Some("next".into()),
                total: Some(7),
            })
        });
    let g = grpc(svc);
    let req = ListTransferMessagesRequest {
        direction: "inbound".into(),
        protocol: "dsp2024".into(),
        state_transition_to: "STARTED".into(),
        ..Default::default()
    };
    let resp = g
        .list_transfer_messages(owner(req))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(resp.items.len(), 1);
    assert_eq!(resp.next_cursor, "next");
    assert_eq!(resp.total, 7);
}

#[tokio::test]
async fn list_by_process_scopes_to_process_and_reuses_list_parsing() {
    let mut svc = MockTransferMessageServiceTrait::new();
    svc.expect_get_all_by_process()
        .withf(|_, process, filter, page, _| {
            process.to_string() == "urn:transfer-process:1"
                && filter.direction == Some(Direction::Outbound)
                && page.limit == 3
        })
        .returning(|_, _, _, _, _| {
            Ok(Paginated {
                items: vec![],
                next_cursor: None,
                total: None,
            })
        });
    let g = grpc(svc);
    let req = ListTransferMessagesByProcessRequest {
        process_id: "urn:transfer-process:1".into(),
        direction: "outbound".into(),
        limit: 3,
        ..Default::default()
    };
    assert!(
        g.list_transfer_messages_by_process(owner(req))
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn get_serializes_envelope_with_hex_hash_and_struct_payload() {
    let mut svc = MockTransferMessageServiceTrait::new();
    svc.expect_get_one().returning(|_, _| Ok(view()));
    let g = grpc(svc);
    let resp = g
        .get_transfer_message(owner(id_request()))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(resp.direction, ProtoDirection::Outbound as i32);
    assert_eq!(resp.protocol, "dsp2025_1");
    let env = resp.envelope.unwrap();
    assert_eq!(env.canonical_hash.len(), 64);
    assert!(env.canonical_hash.chars().all(|c| c.is_ascii_hexdigit()));
    assert_eq!(
        env.payload.unwrap().fields["@type"].kind,
        Some(prost_types::value::Kind::StringValue(
            "TransferRequestMessage".into()
        ))
    );
}
