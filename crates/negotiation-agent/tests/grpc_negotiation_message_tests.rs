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

//! gRPC adapter tests for negotiation messages: auth, parsing, by-process paging, nesting.

mod grpc_fixtures;

use std::sync::Arc;

use common::errors::ResourceError;
use common::paginated_spec::Paginated;
use grpc_fixtures::{
    OTHER_TENANT, StubValidator, TENANT, agreement_row, message_row, offer_row, owner, request, urn,
};
use negotiation_agent::grpc::api::negotiation_agent::negotiation_agent_messages_service_server::NegotiationAgentMessagesService;
use negotiation_agent::grpc::api::negotiation_agent::{
    CreateNegotiationMessageRequest, GetMessagesByProcessIdRequest,
    GetNegotiationMessageByIdRequest, ListNegotiationMessagesRequest,
};
use negotiation_agent::grpc::negotiation_message::NegotiationAgentMessagesGrpc;
use negotiation_agent::services::negotiation_message::MockNegotiationMessageServiceTrait;
use negotiation_agent::services::negotiation_message::views::NegotiationMessageView;
use serde_json::json;
use tonic::Code;

fn grpc(service: MockNegotiationMessageServiceTrait) -> NegotiationAgentMessagesGrpc {
    NegotiationAgentMessagesGrpc::new(Arc::new(service), Arc::new(StubValidator))
}

fn view(n: u32) -> NegotiationMessageView {
    NegotiationMessageView::assemble(message_row(n), Some(offer_row(3)), Some(agreement_row(4)))
}

fn by_id(id: &str) -> GetNegotiationMessageByIdRequest {
    GetNegotiationMessageByIdRequest { id: id.to_string() }
}

#[tokio::test]
async fn get_without_token_is_unauthenticated() {
    let g = grpc(MockNegotiationMessageServiceTrait::new());
    let err = g
        .get_negotiation_message_by_id(request(by_id(&urn(1)), None, Some(TENANT)))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::Unauthenticated);
}

#[tokio::test]
async fn get_foreign_tenant_without_admin_is_permission_denied() {
    let g = grpc(MockNegotiationMessageServiceTrait::new());
    let err = g
        .get_negotiation_message_by_id(request(by_id(&urn(1)), Some("owner"), Some(OTHER_TENANT)))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::PermissionDenied);
}

#[tokio::test]
async fn invalid_urns_name_their_field() {
    let g = grpc(MockNegotiationMessageServiceTrait::new());
    let err = g
        .get_negotiation_message_by_id(owner(by_id("nope")))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(err.message().starts_with("id:"), "{}", err.message());

    let err = g
        .get_messages_by_process_id(owner(GetMessagesByProcessIdRequest {
            process_id: "nope".into(),
            ..Default::default()
        }))
        .await
        .unwrap_err();
    assert!(
        err.message().starts_with("process_id:"),
        "{}",
        err.message()
    );

    let err = g
        .create_negotiation_message(owner(CreateNegotiationMessageRequest {
            negotiation_agent_process_id: "nope".into(),
            ..Default::default()
        }))
        .await
        .unwrap_err();
    assert!(
        err.message().starts_with("negotiation_agent_process_id:"),
        "{}",
        err.message()
    );
}

#[tokio::test]
async fn create_defaults_missing_payload_to_empty_object() {
    let mut svc = MockNegotiationMessageServiceTrait::new();
    svc.expect_create()
        .withf(|_, dto| {
            dto.negotiation_agent_process_id.to_string() == urn(100)
                && dto.payload == json!({})
                && dto.message_type == "ContractRequestMessage"
        })
        .returning(|_, _| Ok(view(1)));
    let g = grpc(svc);
    let req = CreateNegotiationMessageRequest {
        negotiation_agent_process_id: urn(100),
        message_type: "ContractRequestMessage".into(),
        payload: None,
        ..Default::default()
    };
    assert!(g.create_negotiation_message(owner(req)).await.is_ok());
}

#[tokio::test]
async fn domain_not_found_maps_to_not_found() {
    let mut svc = MockNegotiationMessageServiceTrait::new();
    svc.expect_get_one()
        .returning(|_, id| Err(ResourceError::not_found(id, "negotiation message")));
    let g = grpc(svc);
    let err = g
        .get_negotiation_message_by_id(owner(by_id(&urn(1))))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::NotFound);
}

#[tokio::test]
async fn get_nests_offer_and_agreement_from_the_view() {
    let mut svc = MockNegotiationMessageServiceTrait::new();
    svc.expect_get_one().returning(|_, _| Ok(view(1)));
    let g = grpc(svc);
    let msg = g
        .get_negotiation_message_by_id(owner(by_id(&urn(1))))
        .await
        .unwrap()
        .into_inner()
        .message
        .unwrap();
    assert_eq!(msg.offer.unwrap().offer_id, "offer-3");
    assert_eq!(msg.agreement.unwrap().target, urn(300));
    assert_eq!(
        msg.payload.unwrap().fields["@type"].kind,
        Some(prost_types::value::Kind::StringValue(
            "ContractRequestMessage".into()
        ))
    );
}

#[tokio::test]
async fn list_propagates_cursor_total_and_parsed_filters() {
    let mut svc = MockNegotiationMessageServiceTrait::new();
    svc.expect_get_all()
        .withf(|_, filter, page, _| {
            filter.direction.as_deref() == Some("inbound")
                && filter.process_id.is_none()
                && page.limit == 3
                && page.cursor.as_deref() == Some("c")
        })
        .returning(|_, _, _, _| Ok(Paginated::new(vec![view(1)], Some("n".into()), Some(9))));
    let g = grpc(svc);
    let req = ListNegotiationMessagesRequest {
        direction: "inbound".into(),
        limit: 3,
        cursor: "c".into(),
        ..Default::default()
    };
    let resp = g
        .get_all_negotiation_messages(owner(req))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(resp.items.len(), 1);
    assert_eq!(resp.next_cursor, "n");
    assert_eq!(resp.total, 9);
}

#[tokio::test]
async fn by_process_scopes_filter_and_pages() {
    let mut svc = MockNegotiationMessageServiceTrait::new();
    svc.expect_get_all()
        .withf(|_, filter, page, sort| {
            filter.process_id.as_deref() == Some(&urn(100)[..])
                && page.limit == 20
                && sort.as_str() == "created_at_asc"
        })
        .returning(|_, _, _, _| Ok(Paginated::new(vec![view(1), view(2)], None, Some(2))));
    let g = grpc(svc);
    let req = GetMessagesByProcessIdRequest {
        process_id: urn(100),
        sort: "created_at_asc".into(),
        ..Default::default()
    };
    let resp = g
        .get_messages_by_process_id(owner(req))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(resp.items.len(), 2);
    assert!(resp.next_cursor.is_empty());
    assert_eq!(resp.total, 2);
}

#[tokio::test]
async fn list_with_bad_sort_is_invalid_argument() {
    let g = grpc(MockNegotiationMessageServiceTrait::new());
    let err = g
        .get_all_negotiation_messages(owner(ListNegotiationMessagesRequest {
            sort: "sideways".into(),
            ..Default::default()
        }))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(err.message().starts_with("sort:"), "{}", err.message());
}
