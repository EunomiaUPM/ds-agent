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

//! gRPC adapter tests for offers.

mod grpc_fixtures;

use std::sync::Arc;

use common::errors::ResourceError;
use common::grpc::JsonValueExt;
use common::paginated_spec::Paginated;
use grpc_fixtures::{OTHER_TENANT, StubValidator, TENANT, offer_row, owner, request, urn};
use negotiation_agent::grpc::api::negotiation_agent::negotiation_agent_offers_service_server::NegotiationAgentOffersService;
use negotiation_agent::grpc::api::negotiation_agent::{
    CreateOfferRequest, GetBatchOffersRequest, GetOfferByIdRequest,
    GetOfferByNegotiationMessageRequest, GetOffersByNegotiationProcessRequest, ListOffersRequest,
};
use negotiation_agent::grpc::offer::NegotiationAgentOfferGrpc;
use negotiation_agent::services::offer::MockOfferServiceTrait;
use negotiation_agent::services::offer::views::OfferView;
use serde_json::json;
use tonic::Code;

fn grpc(service: MockOfferServiceTrait) -> NegotiationAgentOfferGrpc {
    NegotiationAgentOfferGrpc::new(Arc::new(service), Arc::new(StubValidator))
}

fn view(n: u32) -> OfferView {
    OfferView::assemble(offer_row(n))
}

fn by_id(id: &str) -> GetOfferByIdRequest {
    GetOfferByIdRequest { id: id.to_string() }
}

#[tokio::test]
async fn get_without_token_is_unauthenticated() {
    let g = grpc(MockOfferServiceTrait::new());
    let err = g
        .get_offer_by_id(request(by_id(&urn(1)), None, Some(TENANT)))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::Unauthenticated);
}

#[tokio::test]
async fn get_foreign_tenant_without_admin_is_permission_denied() {
    let g = grpc(MockOfferServiceTrait::new());
    let err = g
        .get_offer_by_id(request(by_id(&urn(1)), Some("owner"), Some(OTHER_TENANT)))
        .await
        .unwrap_err();
    assert_eq!(err.code(), Code::PermissionDenied);
}

#[tokio::test]
async fn invalid_urns_name_their_field() {
    let g = grpc(MockOfferServiceTrait::new());
    let err = g.get_offer_by_id(owner(by_id("nope"))).await.unwrap_err();
    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(err.message().starts_with("id:"), "{}", err.message());

    let err = g
        .get_offer_by_negotiation_message(owner(GetOfferByNegotiationMessageRequest {
            message_id: "nope".into(),
        }))
        .await
        .unwrap_err();
    assert!(
        err.message().starts_with("message_id:"),
        "{}",
        err.message()
    );

    let err = g
        .get_batch_offers(owner(GetBatchOffersRequest {
            ids: vec!["nope".into()],
        }))
        .await
        .unwrap_err();
    assert!(err.message().starts_with("ids[0]:"), "{}", err.message());

    let err = g
        .create_offer(owner(CreateOfferRequest {
            negotiation_agent_process_id: urn(100),
            negotiation_agent_message_id: "nope".into(),
            ..Default::default()
        }))
        .await
        .unwrap_err();
    assert!(
        err.message().starts_with("negotiation_agent_message_id:"),
        "{}",
        err.message()
    );
}

#[tokio::test]
async fn create_maps_struct_content() {
    let mut svc = MockOfferServiceTrait::new();
    svc.expect_create()
        .withf(|_, dto| dto.offer_id == "offer-x" && dto.offer_content == json!({"permission": []}))
        .returning(|_, _| Ok(view(1)));
    let g = grpc(svc);
    let req = CreateOfferRequest {
        negotiation_agent_process_id: urn(100),
        negotiation_agent_message_id: urn(200),
        offer_id: "offer-x".into(),
        offer_content: Some(json!({"permission": []}).into_prost_struct()),
        ..Default::default()
    };
    assert!(g.create_offer(owner(req)).await.is_ok());
}

#[tokio::test]
async fn domain_not_found_maps_to_not_found() {
    let mut svc = MockOfferServiceTrait::new();
    svc.expect_get_one()
        .returning(|_, id| Err(ResourceError::not_found(id, "offer")));
    let g = grpc(svc);
    let err = g.get_offer_by_id(owner(by_id(&urn(1)))).await.unwrap_err();
    assert_eq!(err.code(), Code::NotFound);
}

#[tokio::test]
async fn list_propagates_cursor_total_and_parsed_filters() {
    let mut svc = MockOfferServiceTrait::new();
    svc.expect_get_all()
        .withf(|_, filter, page, _| {
            filter.offer_id.as_deref() == Some("offer-1")
                && filter.target.is_none()
                && page.limit == 2
        })
        .returning(|_, _, _, _| Ok(Paginated::new(vec![view(1)], Some("n".into()), Some(7))));
    let g = grpc(svc);
    let req = ListOffersRequest {
        offer_id: "offer-1".into(),
        limit: 2,
        ..Default::default()
    };
    let resp = g.get_all_offers(owner(req)).await.unwrap().into_inner();
    assert_eq!(resp.items[0].offer_id, "offer-1");
    assert_eq!(resp.next_cursor, "n");
    assert_eq!(resp.total, 7);
}

#[tokio::test]
async fn by_process_returns_full_set_with_total() {
    let mut svc = MockOfferServiceTrait::new();
    svc.expect_get_by_process()
        .withf(|_, id| id.to_string() == urn(100))
        .returning(|_, _| Ok(vec![view(1), view(2), view(3)]));
    let g = grpc(svc);
    let resp = g
        .get_offers_by_negotiation_process(owner(GetOffersByNegotiationProcessRequest {
            process_id: urn(100),
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(resp.total, 3);
    assert!(resp.next_cursor.is_empty());
}
