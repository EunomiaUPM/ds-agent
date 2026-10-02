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

//! EventDispatcher against a local webhook: HMAC signature, event headers and status passing.

use std::collections::HashMap;
use std::time::Duration;

use axum::http::StatusCode;
use events::EventDispatcher;

use crate::support::fixtures::envelope;
use crate::support::webhook::Webhook;

fn dispatcher() -> EventDispatcher {
    EventDispatcher::new(Duration::from_secs(2))
}

/// The signature is the standard HMAC-SHA256 hex digest with a `sha256=` prefix.
#[test]
fn signature_is_hmac_sha256_hex() {
    assert_eq!(
        EventDispatcher::compute_signature("key", b"The quick brown fox jumps over the lazy dog"),
        "sha256=f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8"
    );
}

/// The webhook gets the payload, the event headers, custom headers and a signature over the
/// exact body sent.
#[tokio::test]
async fn sends_payload_headers_and_a_signature_over_the_body() {
    let hook = Webhook::start(StatusCode::OK).await;
    let event = envelope("transfers:started");
    let custom = HashMap::from([("X-Tenant".to_string(), "tenant-1".to_string())]);

    let status = dispatcher()
        .dispatch(&hook.url, &event, Some("s3cret"), Some(&custom))
        .await
        .unwrap();

    assert_eq!(status, StatusCode::OK);
    let received = hook.received();
    assert_eq!(received.len(), 1);
    let req = &received[0];
    let body: serde_json::Value = serde_json::from_slice(&req.body).unwrap();
    assert_eq!(body, event.payload);
    assert_eq!(req.headers["x-event-id"], event.id.to_string().as_str());
    assert_eq!(req.headers["x-event-topic"], "transfers:started");
    assert_eq!(req.headers["content-type"], "application/json");
    assert_eq!(req.headers["x-tenant"], "tenant-1");
    assert_eq!(
        req.headers["x-hub-signature-256"],
        EventDispatcher::compute_signature("s3cret", &req.body).as_str()
    );
}

/// Without a secret no signature header is sent.
#[tokio::test]
async fn no_secret_means_no_signature() {
    let hook = Webhook::start(StatusCode::OK).await;
    dispatcher()
        .dispatch(&hook.url, &envelope("transfers:started"), None, None)
        .await
        .unwrap();
    assert!(!hook.received()[0]
        .headers
        .contains_key("x-hub-signature-256"));
}

/// A failing status is returned as is, after a single request: retrying is not its job.
#[tokio::test]
async fn returns_the_webhook_status_without_retrying() {
    let hook = Webhook::start(StatusCode::SERVICE_UNAVAILABLE).await;
    let status = dispatcher()
        .dispatch(&hook.url, &envelope("transfers:started"), None, None)
        .await
        .unwrap();
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(hook.received().len(), 1);
}

/// An unreachable webhook is an error, not a status.
#[tokio::test]
async fn unreachable_webhook_is_an_error() {
    let result = dispatcher()
        .dispatch(
            "http://127.0.0.1:1/hook",
            &envelope("transfers:started"),
            None,
            None,
        )
        .await;
    assert!(result.is_err());
}
