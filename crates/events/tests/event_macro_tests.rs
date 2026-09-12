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

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use events::bus::envelope::{Topic, TopicPattern};
use events::event;
use events::setup::AppContext;
use events::{Event, EventPublisherTrait};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::net::TcpListener;

// 1. Define event using the inline macro syntax
event! {
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct TransferStartedEvent {
        pub transfer_id: String,
        pub amount: u64,
        pub asset_id: String,
    } => "transfers:bla", "transfers"
}

// 2. Decorate an existing serializable struct with the macro
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferCompletedEvent {
    pub transfer_id: String,
    pub bytes_sent: u64,
}

event!(TransferCompletedEvent, "transfers:completed", "transfers");

#[tokio::test]
async fn test_event_macro_definitions_and_metadata() {
    let ev = TransferStartedEvent {
        transfer_id: "tx-123".to_string(),
        amount: 5000,
        asset_id: "asset-99".to_string(),
    };

    assert_eq!(TransferStartedEvent::event_name(), "transfers:bla");
    assert_eq!(TransferStartedEvent::topic().as_str(), "transfers:bla");
    assert_eq!(TransferStartedEvent::source_crate(), "transfers");
    assert_eq!(TransferStartedEvent::schema_version(), 1);

    let envelope = ev.clone().into_envelope();
    assert_eq!(envelope.topic.as_str(), "transfers:bla");
    assert_eq!(envelope.source_crate, "transfers");
    assert_eq!(envelope.schema_version, 1);

    let deserialized: TransferStartedEvent = serde_json::from_value(envelope.payload).unwrap();
    assert_eq!(deserialized, ev);
}

#[tokio::test]
async fn test_colon_topic_matching_transfers_bla_and_wildcards() {
    let topic = Topic::new("transfers:bla").expect("valid colon topic");
    assert_eq!(topic.as_str(), "transfers:bla");
    assert_eq!(topic.segments(), vec!["transfers", "bla"]);

    // Exact subscription matches
    let exact_pat = TopicPattern::new("transfers:bla").unwrap();
    assert!(exact_pat.matches(&topic));

    // Single-level wildcard matches transfers:bla
    let single_star_colon = TopicPattern::new("transfers:*").unwrap();
    assert!(single_star_colon.matches(&topic));

    let single_star_dot = TopicPattern::new("transfers.*").unwrap();
    assert!(single_star_dot.matches(&topic));

    // Multi-level wildcard matches
    let multi_star_colon = TopicPattern::new("transfers:**").unwrap();
    assert!(multi_star_colon.matches(&topic));

    let multi_star_dot = TopicPattern::new("transfers.**").unwrap();
    assert!(multi_star_dot.matches(&topic));

    // Non-matching prefix
    let catalog_pat = TopicPattern::new("catalog:*").unwrap();
    assert!(!catalog_pat.matches(&topic));

    let catalog_multi = TopicPattern::new("catalog:**").unwrap();
    assert!(!catalog_multi.matches(&topic));
}

#[tokio::test]
async fn test_cross_crate_emit_and_typed_in_memory_subscription() {
    let ctx = Arc::new(AppContext::in_memory(None));
    let mut rx = ctx.event_bus.subscribe();

    let started = TransferStartedEvent {
        transfer_id: "tx-456".to_string(),
        amount: 10000,
        asset_id: "asset-1".to_string(),
    };

    // Emit using EventPublisherTrait::publish_event
    let published = ctx.event_bus.publish_event(started.clone()).await.unwrap();
    assert_eq!(published.topic.as_str(), "transfers:bla");

    // Receive and deserialize on subscriber side
    let received_envelope = rx.recv().await.unwrap();
    assert_eq!(received_envelope.topic.as_str(), "transfers:bla");

    let payload: TransferStartedEvent = serde_json::from_value(received_envelope.payload).unwrap();
    assert_eq!(payload, started);

    // Emit second event using EventBus::emit
    let completed = TransferCompletedEvent {
        transfer_id: "tx-456".to_string(),
        bytes_sent: 2048,
    };
    ctx.event_bus.emit(completed.clone()).await.unwrap();

    let received_completed = rx.recv().await.unwrap();
    assert_eq!(received_completed.topic.as_str(), "transfers:completed");
    let payload_completed: TransferCompletedEvent =
        serde_json::from_value(received_completed.payload).unwrap();
    assert_eq!(payload_completed, completed);
}

#[tokio::test]
async fn test_webhook_subscriber_matching_transfers_star() {
    let received_count = Arc::new(AtomicU32::new(0));
    let counter_clone = received_count.clone();

    let app = Router::new()
        .route(
            "/webhook",
            post(
                move |State(counter): State<Arc<AtomicU32>>, Json(val): Json<Value>| async move {
                    if val.get("transfer_id").and_then(Value::as_str) == Some("tx-789") {
                        counter.fetch_add(1, Ordering::SeqCst);
                    }
                    StatusCode::OK
                },
            ),
        )
        .with_state(counter_clone);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let ctx = Arc::new(AppContext::in_memory(None));

    // Register subscriber with transfers:*
    ctx.subscription_repo
        .create_subscription(events::entities::commands::CreateSubscriptionDto {
            callback_address: format!("http://127.0.0.1:{port}/webhook"),
            topic_pattern: "transfers:*".to_string(),
            secret: None,
            headers: None,
            retry_limit: Some(3),
            expiration_time: None,
        })
        .await
        .unwrap();

    // Register irrelevant subscriber with catalog:*
    ctx.subscription_repo
        .create_subscription(events::entities::commands::CreateSubscriptionDto {
            callback_address: format!("http://127.0.0.1:{port}/should-not-be-called"),
            topic_pattern: "catalog:*".to_string(),
            secret: None,
            headers: None,
            retry_limit: Some(3),
            expiration_time: None,
        })
        .await
        .unwrap();

    // Publish event with topic "transfers:bla"
    let started = TransferStartedEvent {
        transfer_id: "tx-789".to_string(),
        amount: 42,
        asset_id: "sensor-x".to_string(),
    };
    ctx.event_bus.publish_event(started).await.unwrap();

    // Allow background dispatcher to hit webhook
    tokio::time::sleep(Duration::from_millis(150)).await;

    assert_eq!(received_count.load(Ordering::SeqCst), 1);
}
