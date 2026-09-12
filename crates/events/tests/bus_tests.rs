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

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::{Json, Router};
use events::bus::dispatcher::EventDispatcher;
use events::bus::envelope::{EventEnvelope, Topic, TopicPattern};
use events::bus::policy::RetryPolicy;
use events::bus::{EventBus, EventBusTrait};
use events::data::repo::{CreateSubscriptionDto, DeadLetterStatus, DeliveryStatus};
use events::setup::AppContext;
use serde_json::json;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

#[tokio::test]
async fn test_topic_and_pattern_matching() {
    let valid_topic = Topic::new("transfer.process.started").expect("valid topic");
    assert_eq!(valid_topic.as_str(), "transfer.process.started");
    assert_eq!(
        valid_topic.segments(),
        vec!["transfer", "process", "started"]
    );

    assert!(Topic::new("").is_err());
    assert!(Topic::new("transfer.*.started").is_err());
    assert!(Topic::new("transfer..started").is_err());

    let exact_pat = TopicPattern::new("transfer.process.started").unwrap();
    assert!(exact_pat.matches(&valid_topic));

    let single_wildcard = TopicPattern::new("transfer.*.started").unwrap();
    assert!(single_wildcard.matches(&valid_topic));

    let prefix_wildcard = TopicPattern::new("transfer.**").unwrap();
    assert!(prefix_wildcard.matches(&valid_topic));

    let match_all = TopicPattern::match_all();
    assert!(match_all.matches(&valid_topic));

    let non_matching = TopicPattern::new("catalog.**").unwrap();
    assert!(!non_matching.matches(&valid_topic));
}

#[tokio::test]
async fn test_retry_policy_calculation_and_classification() {
    let policy = RetryPolicy {
        max_attempts: 3,
        initial_backoff_secs: 2,
        max_backoff_secs: 60,
        multiplier: 2.0,
        jitter_factor: 0.1,
        timeout_secs: 5,
        poll_interval_secs: 1,
    };

    assert_eq!(policy.calculate_delay(0), Duration::ZERO);
    let delay_1 = policy.calculate_delay(1);
    assert!(delay_1.as_secs_f64() >= 1.5 && delay_1.as_secs_f64() <= 2.5);

    assert!(RetryPolicy::is_retryable_status(408));
    assert!(RetryPolicy::is_retryable_status(429));
    assert!(RetryPolicy::is_retryable_status(500));
    assert!(RetryPolicy::is_retryable_status(503));

    assert!(!RetryPolicy::is_retryable_status(400));
    assert!(!RetryPolicy::is_retryable_status(401));
    assert!(!RetryPolicy::is_retryable_status(404));

    assert!(!policy.is_exhausted(2));
    assert!(policy.is_exhausted(3));
}

#[tokio::test]
async fn test_hmac_sha256_signature() {
    let secret = "my-webhook-secret-key";
    let payload = b"{\"event\":\"test\"}";
    let sig = EventDispatcher::compute_signature(secret, payload);

    assert!(sig.starts_with("sha256="));
    let sig2 = EventDispatcher::compute_signature(secret, payload);
    assert_eq!(sig, sig2);

    let different_sig = EventDispatcher::compute_signature("different-secret", payload);
    assert_ne!(sig, different_sig);
}

#[tokio::test]
async fn test_in_memory_event_bus_broadcast() {
    let ctx = AppContext::in_memory(None);
    let mut rx = ctx.event_bus.subscribe();

    let topic = Topic::new("negotiation.contract.agreed").unwrap();
    let payload = json!({ "contract_id": "c-12345", "status": "Agreed" });
    let envelope = EventEnvelope::new(topic.clone(), "negotiation", 1, None, payload.clone());

    let published = ctx
        .event_bus
        .publish(envelope)
        .await
        .expect("publish succeeds");
    assert_eq!(published.topic, topic);

    let received = rx.recv().await.expect("received from broadcast");
    assert_eq!(received.id, published.id);
    assert_eq!(received.topic, topic);
    assert_eq!(received.payload, payload);
}

#[derive(Clone, Default)]
struct WebhookCapture {
    received_headers: Arc<Mutex<Vec<HashMap<String, String>>>>,
    received_payloads: Arc<Mutex<Vec<serde_json::Value>>>,
    status_to_return: Arc<AtomicU32>,
}

#[tokio::test]
async fn test_webhook_delivery_with_hmac_and_headers() {
    let capture = WebhookCapture::default();
    capture.status_to_return.store(200, Ordering::SeqCst);

    let capture_state = capture.clone();
    let app = Router::new()
        .route(
            "/webhook",
            post(
                |State(state): State<WebhookCapture>,
                 headers: HeaderMap,
                 Json(body): Json<serde_json::Value>| async move {
                    let mut map = HashMap::new();
                    for (k, v) in headers.iter() {
                        map.insert(k.to_string(), v.to_str().unwrap_or_default().to_string());
                    }
                    state.received_headers.lock().await.push(map);
                    state.received_payloads.lock().await.push(body);
                    let code = state.status_to_return.load(Ordering::SeqCst) as u16;
                    StatusCode::from_u16(code).unwrap_or(StatusCode::OK)
                },
            ),
        )
        .with_state(capture_state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let callback_url = format!("http://127.0.0.1:{port}/webhook");
    let ctx = AppContext::in_memory(None);

    let mut custom_headers = HashMap::new();
    custom_headers.insert("X-Custom-Tenant".to_string(), "tenant-alpha".to_string());

    let sub = ctx
        .subscription_repo
        .create_subscription(CreateSubscriptionDto {
            callback_address: callback_url,
            topic_pattern: "transfer.**".to_string(),
            secret: Some("test-secret-123".to_string()),
            headers: Some(custom_headers),
            retry_limit: Some(3),
            expiration_time: None,
        })
        .await
        .expect("create subscription succeeds");

    assert!(sub.active);

    let topic = Topic::new("transfer.process.completed").unwrap();
    let payload = json!({ "process_id": "tp-999", "state": "COMPLETED" });
    let envelope = EventEnvelope::new(topic.clone(), "dataplane", 1, None, payload.clone());

    ctx.event_bus
        .publish(envelope.clone())
        .await
        .expect("publish succeeds");

    // Allow immediate dispatch background task to complete
    tokio::time::sleep(Duration::from_millis(200)).await;

    let payloads = capture.received_payloads.lock().await;
    assert_eq!(payloads.len(), 1);
    assert_eq!(payloads[0], payload);

    let headers_list = capture.received_headers.lock().await;
    let headers = &headers_list[0];
    assert!(headers.contains_key("x-hub-signature-256"));
    assert!(headers
        .get("x-hub-signature-256")
        .unwrap()
        .starts_with("sha256="));
    assert_eq!(
        headers.get("x-event-topic").unwrap(),
        "transfer.process.completed"
    );
    assert_eq!(headers.get("x-custom-tenant").unwrap(), "tenant-alpha");

    let deliveries = ctx
        .delivery_repo
        .list_by_event(envelope.id.as_str())
        .await
        .unwrap();
    assert_eq!(deliveries.len(), 1);
    assert_eq!(deliveries[0].status, DeliveryStatus::Delivered);
    assert_eq!(deliveries[0].attempts, 1);
    assert!(deliveries[0].last_attempt_at.is_some());
    assert!(deliveries[0].delivered_at.is_some());
    assert_eq!(deliveries[0].response_status_code, Some(200));
}

#[tokio::test]
async fn test_retry_and_dead_letter_queue_flow() {
    let capture = WebhookCapture::default();
    capture.status_to_return.store(500, Ordering::SeqCst);

    let capture_state = capture.clone();
    let app = Router::new()
        .route(
            "/failing-webhook",
            post(|State(state): State<WebhookCapture>| async move {
                let code = state.status_to_return.load(Ordering::SeqCst) as u16;
                StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
            }),
        )
        .with_state(capture_state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let callback_url = format!("http://127.0.0.1:{port}/failing-webhook");

    let policy = RetryPolicy {
        max_attempts: 2,
        initial_backoff_secs: 0,
        max_backoff_secs: 1,
        multiplier: 1.0,
        jitter_factor: 0.0,
        timeout_secs: 2,
        poll_interval_secs: 1,
    };

    let ctx = AppContext::in_memory(Some(policy));

    let sub = ctx
        .subscription_repo
        .create_subscription(CreateSubscriptionDto {
            callback_address: callback_url,
            topic_pattern: "data.transfer.failed".to_string(),
            secret: None,
            headers: None,
            retry_limit: Some(2),
            expiration_time: None,
        })
        .await
        .unwrap();

    let topic = Topic::new("data.transfer.failed").unwrap();
    let payload = json!({ "error": "network reset" });
    let envelope = EventEnvelope::new(topic, "dataplane", 1, None, payload);

    ctx.event_bus.publish(envelope.clone()).await.unwrap();

    // Allow initial failed dispatch
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Delivery failed attempt 1, scheduled for retry
    let deliveries = ctx
        .delivery_repo
        .list_by_event(envelope.id.as_str())
        .await
        .unwrap();
    assert_eq!(deliveries.len(), 1);
    assert_eq!(deliveries[0].status, DeliveryStatus::Failed);
    assert_eq!(deliveries[0].attempts, 1);

    // Run retry worker batch for attempt 2 (which exhausts retry_limit of 2)
    let processed = ctx.retry_worker.process_batch().await.unwrap();
    assert_eq!(processed, 1);

    // Verify delivery transitioned to DeadLetter
    let updated_delivery = ctx
        .delivery_repo
        .get_delivery(&deliveries[0].id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updated_delivery.status, DeliveryStatus::DeadLetter);

    // Verify record in Dead Letter Queue
    let dead_letters = ctx
        .dlq_repo
        .list_dead_letters(Some("Unresolved"), 10, 0)
        .await
        .unwrap();
    assert_eq!(dead_letters.len(), 1);
    assert_eq!(dead_letters[0].status, DeadLetterStatus::Unresolved);
    assert_eq!(dead_letters[0].topic, "data.transfer.failed");

    // Test DLQ Replay: Webhook recovers and returns 200 OK
    capture.status_to_return.store(200, Ordering::SeqCst);
    let replayed = ctx
        .event_bus
        .replay_dead_letter(&dead_letters[0].id)
        .await
        .expect("replay succeeds");
    assert_eq!(replayed.status, DeliveryStatus::Delivered);

    // Dead letter is now marked Replayed
    let resolved_dl = ctx
        .dlq_repo
        .get_dead_letter(&dead_letters[0].id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(resolved_dl.status, DeadLetterStatus::Replayed);
}

#[tokio::test]
async fn test_http_api_endpoints() {
    let ctx = Arc::new(AppContext::in_memory(None));
    let app = events::http::EventsHttpRouter::build(ctx.event_bus.clone());

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let client = reqwest::Client::new();
    let base = format!("http://127.0.0.1:{port}");

    let create_sub_resp = client
        .post(format!("{base}/subscriptions"))
        .json(&json!({
            "callback_address": "https://example.com/webhook",
            "topic_pattern": "catalog.**",
            "secret": "s3cr3t",
            "retry_limit": 5
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(create_sub_resp.status(), StatusCode::CREATED);
    let sub: events::data::repo::SubscriptionRecord = create_sub_resp.json().await.unwrap();
    assert_eq!(sub.callback_address, "https://example.com/webhook");

    let list_subs_resp = client
        .get(format!("{base}/subscriptions"))
        .send()
        .await
        .unwrap();
    assert_eq!(list_subs_resp.status(), StatusCode::OK);
    let subs: Vec<events::data::repo::SubscriptionRecord> = list_subs_resp.json().await.unwrap();
    assert_eq!(subs.len(), 1);

    let pub_resp = client
        .post(format!("{base}/events/publish"))
        .json(&json!({
            "topic": "catalog.dataset.published",
            "source_crate": "catalog",
            "schema_version": 1,
            "payload": { "dataset_id": "ds-100" }
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(pub_resp.status(), StatusCode::CREATED);
    let event: EventEnvelope = pub_resp.json().await.unwrap();
    assert_eq!(event.topic.as_str(), "catalog.dataset.published");

    let list_events_resp = client
        .get(format!("{base}/events?topic=catalog.dataset.published"))
        .send()
        .await
        .unwrap();
    assert_eq!(list_events_resp.status(), StatusCode::OK);
    let events: Vec<EventEnvelope> = list_events_resp.json().await.unwrap();
    assert_eq!(events.len(), 1);

    let dlq_resp = client.get(format!("{base}/dlq")).send().await.unwrap();
    assert_eq!(dlq_resp.status(), StatusCode::OK);
    let dlq: Vec<events::data::repo::DeadLetterRecord> = dlq_resp.json().await.unwrap();
    assert!(dlq.is_empty());

    // Direct root /publish route
    let direct_pub = client
        .post(format!("{base}/publish"))
        .json(&json!({
            "topic": "catalog.dataset.updated",
            "payload": { "dataset_id": "ds-100" }
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(direct_pub.status(), StatusCode::CREATED);

    // Updating non-existent subscription returns 404
    let update_404_resp = client
        .put(format!(
            "{base}/subscriptions/urn:uuid:00000000-0000-0000-0000-000000000000"
        ))
        .json(&json!({
            "callback_address": "https://example.com/changed"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(update_404_resp.status(), StatusCode::NOT_FOUND);

    // Invalid URN returns 400 Bad Request
    let invalid_urn_resp = client
        .get(format!("{base}/not-valid-urn!@#"))
        .send()
        .await
        .unwrap();
    assert_eq!(invalid_urn_resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_events_module_metadata_and_routes() {
    use common::module_loader::service_module::ServiceModuleTrait;
    let ctx = Arc::new(AppContext::in_memory(None));
    let module = events::setup::EventsModule::new(ctx);

    assert_eq!(module.name(), "events");
    assert!(!module.migrations().is_empty());
    let http = module.http().expect("has http routes");
    assert_eq!(http.0, "/api/v1/events");
}
