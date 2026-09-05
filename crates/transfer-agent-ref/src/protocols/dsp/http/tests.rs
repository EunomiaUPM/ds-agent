use crate::protocols::dsp::http::dsp::DspRouter;
use axum::Router;
use axum::extract::Request;
use axum::response::Response;
use common::facades::ssi_auth_facade::MockSSIAuthFacadeTrait;
use http::StatusCode;
use std::sync::Arc;
use tower::ServiceExt;
use ymir::data::entities::shared::participant::Model as Mates;
use ymir::errors::Errors;
use ymir::types::participants::ParticipantType;

const REQUEST_BODY: &str = r#"{
        "@context": ["https://w3id.org/dspace/2025/1/context.jsonld"],
        "@type": "TransferRequestMessage",
        "consumerPid": "urn:uuid:32541fe6-c580-409e-85a8-8a9a32fbe833",
        "agreementId": "urn:uuid:e8dc8655-44c2-46ef-b701-4cffdc2faa44",
        "format": "example:HTTP_PUSH",
        "callbackAddress": "https://example.com/callback"
    }"#;

fn mate() -> Mates {
    let t = chrono::Utc::now();
    Mates {
        participant_id: "did:example:consumer".into(),
        participant_type: ParticipantType::Agent,
        participant_nick: "Consumer".to_string(),
        base_url: "http://127.0.0.1:1100".to_string(),
        token: None,
        saved_at: t.into(),
        last_interaction: t.into(),
        extra_fields: serde_json::Value::Null,
        is_me: false,
    }
}

/// A router whose auth facade accepts every token, or rejects every token.
fn router(authorized: bool) -> Router {
    let mut ssi = MockSSIAuthFacadeTrait::new();
    if authorized {
        ssi.expect_verify_token().returning(|_| Ok(mate()));
    } else {
        ssi.expect_verify_token()
            .returning(|_| Err(Errors::unauthorized("nope", None)));
    }
    DspRouter::new(Arc::new(ssi)).router()
}

fn post_request(token: Option<&str>) -> Request {
    let mut builder = Request::builder()
        .method("POST")
        .uri("/request")
        .header("content-type", "application/json");
    if let Some(token) = token {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
    builder.body(axum::body::Body::from(REQUEST_BODY)).unwrap()
}

async fn body_json(response: Response) -> serde_json::Value {
    let bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

/// The pipeline runs green end to end: wire -> parsed -> rdf -> typed. Asserts
/// only that no stage errored; the DSP response body is still to come.
#[tokio::test]
async fn transfer_request_runs_the_pipeline() {
    let response = router(true)
        .oneshot(post_request(Some("tok")))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::ACCEPTED, "a stage errored");
}

/// DSP 10.2.2: 201 Created with a TransferProcess body (9.3.1).
#[tokio::test]
#[ignore = "needs to_domain and the DSP response projection in build_response"]
async fn transfer_request_answers_201_with_a_transfer_process() {
    let response = router(true)
        .oneshot(post_request(Some("tok")))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = body_json(response).await;
    assert_eq!(body["@type"], "TransferProcess");
    assert_eq!(body["state"], "REQUESTED");
    assert_eq!(
        body["consumerPid"],
        "urn:uuid:32541fe6-c580-409e-85a8-8a9a32fbe833"
    );
    assert!(
        body["providerPid"].as_str().is_some_and(|p| !p.is_empty()),
        "providerPid is REQUIRED: {body}"
    );
    assert!(body["@context"].is_array(), "@context is REQUIRED: {body}");
}

/// DSP 9.2.1: a repeated Transfer Request for a known consumerPid is answered
/// as the first one was — same providerPid, not a second process.
#[tokio::test]
#[ignore = "needs to_domain and the DSP response projection in build_response"]
async fn a_repeated_transfer_request_replays_the_same_ack() {
    let app = router(true);
    let first = app
        .clone()
        .oneshot(post_request(Some("tok")))
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::CREATED);
    let first = body_json(first).await;

    let second = app.oneshot(post_request(Some("tok"))).await.unwrap();
    assert_eq!(second.status(), StatusCode::CREATED);
    let second = body_json(second).await;

    assert_eq!(
        first["providerPid"], second["providerPid"],
        "a retry must not mint a second Transfer Process"
    );
    assert_eq!(first, second);
}

/// DSP 10.1.2.3: an unauthorized client gets 404, not 401 — the same answer as
/// a missing process, so probing reveals nothing.
#[tokio::test]
async fn an_unauthorized_request_answers_404() {
    let response = router(false)
        .oneshot(post_request(Some("bad")))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_request_without_a_token_answers_404() {
    let response = router(true).oneshot(post_request(None)).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
