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

//! Offer router over a mocked offer service: `odrlOffer` travels as an ODRL `Offer`.

use axum::body::Body;
use axum::extract::Request;
use axum::http::StatusCode;
use catalog_agent::entities::resource_kind::ResourceKind;
use catalog_agent::http::odrl_policies::OdrlOfferEntityRouter;
use catalog_agent::services::odrl_policies::MockOdrlPolicyServiceTrait;
use common::dsp_common::odrl::ODRL_CONTEXT_URL;
use common::test_utils::scopes::TestUsers;
use serde_json::{json, Value};
use tower::ServiceExt;

use crate::support::builders::{id, offer};
use crate::support::context::TestContext;

async fn post(router: axum::Router, body: Value) -> (StatusCode, Value) {
    let mut request = Request::post("/")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    request.extensions_mut().insert(TestUsers::alone("ana"));
    let response = router.oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

fn new_offer(context: &str) -> Value {
    json!({
        "odrlOffer": {"@context": context, "@type": "Offer", "permission": [{"action": "use"}]},
        "entityId": id(ResourceKind::Dataset, 3),
        "entityType": "Dataset"
    })
}

/// The rules reach the service without their context, and come back as an ODRL `Offer`.
#[tokio::test]
async fn offer_rules_travel_as_an_odrl_offer() {
    let mut service = MockOdrlPolicyServiceTrait::new();
    service
        .expect_create()
        .withf(|_, command| serde_json::to_value(command.odrl_offer.inner()).unwrap().get("@context").is_none())
        .returning(|_, command| Ok(offer(1, &command.entity_id)));
    let ctx = TestContext::new().offers(service).build();

    let (status, answer) = post(OdrlOfferEntityRouter::new(ctx).router(), new_offer(ODRL_CONTEXT_URL)).await;

    assert_eq!(status, StatusCode::CREATED, "{answer}");
    assert_eq!(answer["odrlOffer"]["@context"], ODRL_CONTEXT_URL);
    assert_eq!(answer["odrlOffer"]["@type"], "Offer");
    assert_eq!(answer["odrlOffer"]["permission"][0]["action"], "use");
}

/// Rules in another context are refused before the service is called.
#[tokio::test]
async fn offer_rules_in_another_context_are_refused() {
    let ctx = TestContext::new().offers(MockOdrlPolicyServiceTrait::new()).build();
    let (status, _) = post(
        OdrlOfferEntityRouter::new(ctx).router(),
        new_offer("https://w3id.org/dspace/2025/1/context.jsonld"),
    )
    .await;
    assert!(status.is_client_error(), "{status}");
}
