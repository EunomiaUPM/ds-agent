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

//! NegotiationAgentAgreementGrpc: auth, URN parsing, DTO mapping and paging.

use std::sync::Arc;

use common::errors::ResourceError;
use common::grpc::JsonValueExt;
use common::paginated_spec::Paginated;
use common::test_utils::grpc::{GrpcRequests, StubTokenValidator};
use negotiation_agent::grpc::agreement::NegotiationAgentAgreementGrpc;
use negotiation_agent::grpc::api::negotiation_agent::negotiation_agent_agreements_service_server::NegotiationAgentAgreementsService;
use negotiation_agent::grpc::api::negotiation_agent::{
    CreateAgreementRequest, GetAgreementByIdRequest, GetAgreementByNegotiationProcessRequest,
    ListAgreementsRequest, PutAgreementRequest,
};
use negotiation_agent::services::agreement::MockAgreementServiceTrait;
use negotiation_agent::services::agreement::views::AgreementView;
use serde_json::json;
use tonic::Code;

use crate::support::fixtures::{agreement_row, urn};

fn grpc(service: MockAgreementServiceTrait) -> NegotiationAgentAgreementGrpc {
    NegotiationAgentAgreementGrpc::new(Arc::new(service), Arc::new(StubTokenValidator))
}

fn view(n: u32) -> AgreementView {
    AgreementView::assemble(agreement_row(n))
}

fn by_id(id: &str) -> GetAgreementByIdRequest {
    GetAgreementByIdRequest { id: id.to_string() }
}

/// A call without token is Unauthenticated.
#[tokio::test]
async fn get_without_token_is_unauthenticated() {
    let g = grpc(MockAgreementServiceTrait::new());
    let err = g
        .get_agreement_by_id(GrpcRequests::with_auth(by_id(&urn(1)), None))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::Unauthenticated);
}

/// A malformed URN is InvalidArgument and the message names the field.
#[tokio::test]
async fn invalid_urns_name_their_field() {
    let g = grpc(MockAgreementServiceTrait::new());
    let err = g
        .get_agreement_by_id(GrpcRequests::owner(by_id("nope")))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(err.message().starts_with("id:"), "{}", err.message());

    let err = g
        .get_agreement_by_negotiation_process(GrpcRequests::owner(
            GetAgreementByNegotiationProcessRequest {
                process_id: "nope".into(),
            },
        ))
        .await
        .unwrap_err();
    assert!(
        err.message().starts_with("process_id:"),
        "{}",
        err.message()
    );

    let err = g
        .create_agreement(GrpcRequests::owner(CreateAgreementRequest {
            negotiation_agent_process_id: urn(100),
            negotiation_agent_message_id: urn(200),
            target: "nope".into(),
            ..Default::default()
        }))
        .await
        .unwrap_err();
    assert!(err.message().starts_with("target:"), "{}", err.message());
}

/// Create and put requests reach the service as the matching DTOs.
#[tokio::test]
async fn create_and_put_map_their_dtos() {
    let mut svc = MockAgreementServiceTrait::new();
    svc.expect_create()
        .withf(|_, dto| {
            dto.target.to_string() == urn(300)
                && dto.consumer_participant_id == "consumer"
                && dto.agreement_content == json!({"target": "urn:x"})
        })
        .returning(|_, _| Ok(view(1)));
    svc.expect_edit()
        .withf(|_, id, dto| id.to_string() == urn(1) && dto.state.as_deref() == Some("FINALIZED"))
        .returning(|_, _, _| Ok(view(1)));
    let g = grpc(svc);
    let create = CreateAgreementRequest {
        negotiation_agent_process_id: urn(100),
        negotiation_agent_message_id: urn(200),
        consumer_participant_id: "consumer".into(),
        provider_participant_id: "provider".into(),
        agreement_content: Some(json!({"target": "urn:x"}).into_prost_struct()),
        target: urn(300),
        ..Default::default()
    };
    assert!(
        g.create_agreement(GrpcRequests::owner(create))
            .await
            .is_ok()
    );
    let put = PutAgreementRequest {
        id: urn(1),
        state: Some("FINALIZED".into()),
    };
    assert!(g.put_agreement(GrpcRequests::owner(put)).await.is_ok());
}

/// A not-found from the service becomes NotFound.
#[tokio::test]
async fn domain_not_found_maps_to_not_found() {
    let mut svc = MockAgreementServiceTrait::new();
    svc.expect_get_one()
        .returning(|_, id| Err(ResourceError::not_found(id, "agreement")));
    let g = grpc(svc);
    let err = g
        .get_agreement_by_id(GrpcRequests::owner(by_id(&urn(1))))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::NotFound);
}

/// List parses the filters and page, and returns the service's cursor and total.
#[tokio::test]
async fn list_propagates_cursor_total_and_parsed_filters() {
    let mut svc = MockAgreementServiceTrait::new();
    svc.expect_get_all()
        .withf(|_, filter, page, sort| {
            filter.state.as_deref() == Some("ACTIVE")
                && filter.consumer_id.as_deref() == Some("consumer")
                && page.limit == 4
                && sort.as_str() == "created_at_desc"
        })
        .returning(|_, _, _, _| Ok(Paginated::new(vec![view(1)], Some("n".into()), Some(11))));
    let g = grpc(svc);
    let req = ListAgreementsRequest {
        state: "ACTIVE".into(),
        consumer_id: "consumer".into(),
        limit: 4,
        ..Default::default()
    };
    let resp = g
        .get_all_agreements(GrpcRequests::owner(req))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(resp.items[0].state, "ACTIVE");
    assert!(resp.items[0].agreement_content.is_some());
    assert_eq!(resp.next_cursor, "n");
    assert_eq!(resp.total, 11);
}
