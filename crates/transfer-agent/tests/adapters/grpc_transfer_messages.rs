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

//! TransferMessagesGrpc: auth, field parsing, envelope building, error mapping and response
//! shaping.

use common::test_utils::grpc::{GrpcRequests, OTHER_TENANT, StubTokenValidator, TENANT};
use std::sync::Arc;
use transfer_agent::grpc::api::transfer_messages::{
    CreateTransferMessageRequest, ListTransferMessagesByProcessRequest,
    ListTransferMessagesRequest, ResourceIdRequest,
    transfer_messages_ref_server::TransferMessagesRef,
};

use chrono::Utc;
use tonic::Code;

use common::errors::ResourceError;
use common::grpc::JsonValueExt;
use common::query::Paginated;
use transfer_agent::entities::ids::{MessageId, TransferProcessId};
use transfer_agent::entities::message_envelope::MessageEnvelope;
use transfer_agent::entities::protocol::{ProtocolId, ProtocolMessageType};
use transfer_agent::entities::transfer_message::Direction;
use transfer_agent::grpc::api::transfer_messages::Direction as ProtoDirection;
use transfer_agent::grpc::transfer_messages::TransferMessagesGrpc;
use transfer_agent::services::transfer_message::MockTransferMessageServiceTrait;
use transfer_agent::services::transfer_message::views::TransferMessageView;

fn grpc(service: MockTransferMessageServiceTrait) -> TransferMessagesGrpc {
    TransferMessagesGrpc::new(Arc::new(service), Arc::new(StubTokenValidator))
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

/// A call without token is Unauthenticated.
#[tokio::test]
async fn get_without_token_is_unauthenticated() {
    let g = grpc(MockTransferMessageServiceTrait::new());
    let err = g
        .get_transfer_message(GrpcRequests::with_auth(id_request(), None, Some(TENANT)))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::Unauthenticated);
}

/// A non-admin naming another tenant is PermissionDenied.
#[tokio::test]
async fn get_foreign_tenant_without_admin_is_permission_denied() {
    let g = grpc(MockTransferMessageServiceTrait::new());
    let err = g
        .get_transfer_message(GrpcRequests::with_auth(
            id_request(),
            Some("owner"),
            Some(OTHER_TENANT),
        ))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::PermissionDenied);
}

/// Without tenant header the caller acts on its token's tenant.
#[tokio::test]
async fn missing_tenant_header_falls_back_to_token_tenant() {
    let mut svc = MockTransferMessageServiceTrait::new();
    svc.expect_get_one()
        .withf(|scope, _| scope.acting_tenant() == TENANT)
        .returning(|_, _| Ok(view()));
    let g = grpc(svc);
    assert!(
        g.get_transfer_message(GrpcRequests::with_auth(id_request(), Some("owner"), None))
            .await
            .is_ok()
    );
}

/// A malformed id is InvalidArgument on `id`.
#[tokio::test]
async fn get_invalid_urn_is_invalid_argument_naming_field() {
    let g = grpc(MockTransferMessageServiceTrait::new());
    let err = g
        .get_transfer_message(GrpcRequests::owner(ResourceIdRequest {
            id: "not a urn".into(),
        }))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(err.message().starts_with("id:"), "{}", err.message());
}

/// Listing by process rejects a malformed process id or an unknown direction.
#[tokio::test]
async fn list_by_process_rejects_invalid_process_id_and_direction() {
    let g = grpc(MockTransferMessageServiceTrait::new());
    let err = g
        .list_transfer_messages_by_process(GrpcRequests::owner(
            ListTransferMessagesByProcessRequest {
                process_id: "nope".into(),
                ..Default::default()
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(
        err.message().starts_with("process_id:"),
        "{}",
        err.message()
    );

    let err = g
        .list_transfer_messages_by_process(GrpcRequests::owner(
            ListTransferMessagesByProcessRequest {
                process_id: "urn:transfer-process:1".into(),
                direction: "sideways".into(),
                ..Default::default()
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(err.message().starts_with("direction:"), "{}", err.message());
}

/// Create rejects an unknown direction or a missing protocol.
#[tokio::test]
async fn create_rejects_unknown_direction_and_missing_protocol() {
    let g = grpc(MockTransferMessageServiceTrait::new());
    let err = g
        .create_transfer_message(GrpcRequests::owner(CreateTransferMessageRequest {
            direction: 9,
            ..valid_create()
        }))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(err.message().starts_with("direction:"), "{}", err.message());

    let err = g
        .create_transfer_message(GrpcRequests::owner(CreateTransferMessageRequest {
            protocol: String::new(),
            ..valid_create()
        }))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(err.message().starts_with("protocol:"), "{}", err.message());
}

/// Create builds the message envelope and hashes its canonical form.
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
        g.create_transfer_message(GrpcRequests::owner(valid_create()))
            .await
            .is_ok()
    );
}

/// Without payload or canonical form, create builds a bare envelope.
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
    assert!(
        g.create_transfer_message(GrpcRequests::owner(req))
            .await
            .is_ok()
    );
}

/// A not-found from the service becomes NotFound.
#[tokio::test]
async fn domain_not_found_maps_to_not_found() {
    let mut svc = MockTransferMessageServiceTrait::new();
    svc.expect_get_one()
        .returning(|_, id| Err(ResourceError::not_found(id, "transfer message")));
    let g = grpc(svc);
    let err = g
        .get_transfer_message(GrpcRequests::owner(id_request()))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::NotFound);
    assert_eq!(err.message(), "transfer message not found");
}

/// List parses the filters and page, and returns the service's cursor and total.
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
        .list_transfer_messages(GrpcRequests::owner(req))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(resp.items.len(), 1);
    assert_eq!(resp.next_cursor, "next");
    assert_eq!(resp.total, 7);
}

/// Listing by process filters on the process and parses the rest like a list.
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
        g.list_transfer_messages_by_process(GrpcRequests::owner(req))
            .await
            .is_ok()
    );
}

/// Get returns the envelope with a hex hash and the payload as a Struct.
#[tokio::test]
async fn get_serializes_envelope_with_hex_hash_and_struct_payload() {
    let mut svc = MockTransferMessageServiceTrait::new();
    svc.expect_get_one().returning(|_, _| Ok(view()));
    let g = grpc(svc);
    let resp = g
        .get_transfer_message(GrpcRequests::owner(id_request()))
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
