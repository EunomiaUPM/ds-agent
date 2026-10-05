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

//! EventBus with mocked repositories and a local webhook: publishing, the first delivery
//! attempt and its outcome, and dead letter replay.

use axum::http::StatusCode;
use chrono::Utc;
use common::oauth::OwnerScope;
use events::data::repo::{DeadLetterRecord, DeadLetterStatus, DeliveryStatus};
use events::entities::envelope::EventEnvelope;
use serde_json::json;
use ymir::errors::Errors;

use crate::support::fixtures::{envelope, owner, subscription, Repos};
use crate::support::signals::Signals;
use crate::support::webhook::Webhook;

/// Repos that store any event and find `subs` for it.
fn publishing(subs: Vec<events::data::repo::SubscriptionRecord>) -> Repos {
    let mut repos = Repos::default();
    repos
        .events
        .expect_insert_event()
        .times(1)
        .returning(|_| Ok(()));
    repos
        .subscriptions
        .expect_get_matching_subscriptions()
        .withf(|record_owner, topic| *record_owner == owner() && topic.as_str() == "transfers:started")
        .returning(move |_, _| Ok(subs.clone()));
    repos
}

/// A publish stores the event, broadcasts it in-process and returns it.
#[tokio::test]
async fn publish_stores_and_broadcasts_the_event() {
    let bus = publishing(vec![]).bus();
    let mut listener = bus.subscribe();

    let event = envelope("transfers:started");
    let published = bus.publish(event.clone()).await.unwrap();

    assert_eq!(published.id, event.id);
    let heard: EventEnvelope = listener.recv().await.unwrap();
    assert_eq!(heard.id, event.id);
}

/// A failing event store fails the publish before anything is broadcast.
#[tokio::test]
async fn store_failure_fails_the_publish() {
    let mut repos = Repos::default();
    repos
        .events
        .expect_insert_event()
        .returning(|_| Err(Errors::crazy("db down", None)));
    let bus = repos.bus();
    let mut listener = bus.subscribe();

    assert!(bus.publish(envelope("transfers:started")).await.is_err());
    assert!(listener.try_recv().is_err());
}

/// Each matching subscription gets a pending delivery, and a webhook answering 2xx marks it
/// delivered on the first attempt.
#[tokio::test]
async fn each_matching_subscription_is_delivered() {
    let hook = Webhook::start(StatusCode::OK).await;
    let mut repos = publishing(vec![
        subscription("sub-1", &hook.url, Some("s3cret"), None),
        subscription("sub-2", &hook.url, None, None),
    ]);
    repos
        .deliveries
        .expect_create_delivery()
        .withf(|d| d.status == DeliveryStatus::Pending && d.attempts == 0)
        .times(2)
        .returning(|d| Ok(d.clone()));
    let mut signals = Signals::new();
    let tx = signals.tx.clone();
    repos
        .deliveries
        .expect_mark_delivered()
        .withf(|_, attempts, status| *attempts == 1 && *status == 200)
        .times(2)
        .returning(move |_, _, _| {
            tx.send("delivered").unwrap();
            Ok(())
        });

    repos
        .bus()
        .publish(envelope("transfers:started"))
        .await
        .unwrap();

    assert_eq!(signals.next(2).await, vec!["delivered", "delivered"]);
    assert_eq!(hook.received().len(), 2);
}

/// A subscription whose delivery record cannot be created is skipped; the others still go out.
#[tokio::test]
async fn a_failed_delivery_record_does_not_block_the_rest() {
    let hook = Webhook::start(StatusCode::OK).await;
    let mut repos = publishing(vec![
        subscription("broken", &hook.url, None, None),
        subscription("fine", &hook.url, None, None),
    ]);
    repos.deliveries.expect_create_delivery().returning(|d| {
        if d.subscription_id == "broken" {
            Err(Errors::crazy("insert failed", None))
        } else {
            Ok(d.clone())
        }
    });
    let mut signals = Signals::new();
    let tx = signals.tx.clone();
    repos
        .deliveries
        .expect_mark_delivered()
        .times(1)
        .returning(move |_, _, _| {
            tx.send("delivered").unwrap();
            Ok(())
        });

    repos
        .bus()
        .publish(envelope("transfers:started"))
        .await
        .unwrap();

    assert_eq!(signals.next(1).await, vec!["delivered"]);
    assert_eq!(hook.received().len(), 1);
}

/// A retryable failure on the first attempt schedules the next one.
#[tokio::test]
async fn retryable_failure_schedules_a_retry() {
    let hook = Webhook::start(StatusCode::SERVICE_UNAVAILABLE).await;
    let mut repos = publishing(vec![subscription("sub-1", &hook.url, None, Some(3))]);
    repos
        .deliveries
        .expect_create_delivery()
        .returning(|d| Ok(d.clone()));
    let mut signals = Signals::new();
    let tx = signals.tx.clone();
    repos
        .deliveries
        .expect_record_failed_attempt()
        .withf(|_, attempts, next, error, status| {
            *attempts == 1 && next.is_some() && error.contains("503") && *status == Some(503)
        })
        .times(1)
        .returning(move |_, _, _, _, _| {
            tx.send("retry scheduled").unwrap();
            Ok(())
        });

    repos
        .bus()
        .publish(envelope("transfers:started"))
        .await
        .unwrap();

    assert_eq!(signals.next(1).await, vec!["retry scheduled"]);
}

/// A non-retryable status sends the delivery straight to the dead letter queue.
#[tokio::test]
async fn permanent_failure_goes_to_the_dead_letter_queue() {
    let hook = Webhook::start(StatusCode::BAD_REQUEST).await;
    let mut repos = publishing(vec![subscription("sub-1", &hook.url, None, Some(5))]);
    repos
        .deliveries
        .expect_create_delivery()
        .returning(|d| Ok(d.clone()));
    repos
        .deliveries
        .expect_record_failed_attempt()
        .withf(|_, attempts, next, _, status| {
            *attempts == 1 && next.is_none() && *status == Some(400)
        })
        .returning(|_, _, _, _, _| Ok(()));
    repos
        .deliveries
        .expect_mark_dead_letter()
        .times(1)
        .returning(|_| Ok(()));
    let mut signals = Signals::new();
    let tx = signals.tx.clone();
    repos
        .dead_letters
        .expect_create_dead_letter()
        .withf(|dl| {
            dl.subscription_id == "sub-1"
                && dl.topic == "transfers:started"
                && dl.attempts == 1
                && dl.status == DeadLetterStatus::Unresolved
        })
        .returning(move |dl| {
            tx.send("dead letter").unwrap();
            Ok(dl.clone())
        });

    repos
        .bus()
        .publish(envelope("transfers:started"))
        .await
        .unwrap();

    assert_eq!(signals.next(1).await, vec!["dead letter"]);
}

/// With a retry limit of one, even a retryable failure is final.
#[tokio::test]
async fn retry_limit_of_one_makes_any_failure_final() {
    let hook = Webhook::start(StatusCode::SERVICE_UNAVAILABLE).await;
    let mut repos = publishing(vec![subscription("sub-1", &hook.url, None, Some(1))]);
    repos
        .deliveries
        .expect_create_delivery()
        .returning(|d| Ok(d.clone()));
    repos
        .deliveries
        .expect_record_failed_attempt()
        .withf(|_, _, next, _, _| next.is_none())
        .returning(|_, _, _, _, _| Ok(()));
    repos
        .deliveries
        .expect_mark_dead_letter()
        .returning(|_| Ok(()));
    let mut signals = Signals::new();
    let tx = signals.tx.clone();
    repos
        .dead_letters
        .expect_create_dead_letter()
        .returning(move |dl| {
            tx.send("dead letter").unwrap();
            Ok(dl.clone())
        });

    repos
        .bus()
        .publish(envelope("transfers:started"))
        .await
        .unwrap();

    assert_eq!(signals.next(1).await, vec!["dead letter"]);
}

/// An unreachable webhook schedules a retry, with no status recorded.
#[tokio::test]
async fn unreachable_webhook_schedules_a_retry() {
    let mut repos = publishing(vec![subscription(
        "sub-1",
        "http://127.0.0.1:1/hook",
        None,
        None,
    )]);
    repos
        .deliveries
        .expect_create_delivery()
        .returning(|d| Ok(d.clone()));
    let mut signals = Signals::new();
    let tx = signals.tx.clone();
    repos
        .deliveries
        .expect_record_failed_attempt()
        .withf(|_, attempts, next, _, status| *attempts == 1 && next.is_some() && status.is_none())
        .returning(move |_, _, _, _, _| {
            tx.send("retry scheduled").unwrap();
            Ok(())
        });

    repos
        .bus()
        .publish(envelope("transfers:started"))
        .await
        .unwrap();

    assert_eq!(signals.next(1).await, vec!["retry scheduled"]);
}

/// A topic that is not a valid topic is rejected before anything is stored.
#[tokio::test]
async fn emitting_under_an_invalid_topic_is_rejected() {
    let bus = Repos::default().bus();
    let result = bus
        .emit_payload_for(&owner(), "transfers:*", "tests", &json!({}))
        .await;
    assert!(result.is_err());
}

fn dead_letter(event: &EventEnvelope, callback: &str) -> DeadLetterRecord {
    DeadLetterRecord {
        id: "dl-1".to_string(),
        owner: owner(),
        delivery_id: Some("delivery-1".to_string()),
        event_id: event.id.to_string(),
        subscription_id: "sub-1".to_string(),
        topic: event.topic.to_string(),
        callback_address: callback.to_string(),
        payload: event.payload.clone(),
        error_message: "HTTP 503".to_string(),
        attempts: 5,
        status: DeadLetterStatus::Unresolved,
        failed_at: Utc::now(),
        replayed_at: None,
    }
}

/// Repos that find the dead letter, its event and its subscription pointing at `callback`.
fn replaying(event: &EventEnvelope, callback: &str) -> Repos {
    let mut repos = Repos::default();
    let dl = dead_letter(event, callback);
    repos
        .dead_letters
        .expect_get_dead_letter()
        .returning(move |_, _| Ok(Some(dl.clone())));
    let stored = event.clone();
    repos
        .events
        .expect_get_event_by_id()
        .returning(move |_, _| Ok(Some(stored.clone())));
    let sub = subscription("sub-1", callback, None, None);
    repos
        .subscriptions
        .expect_get_subscription()
        .returning(move |_, _| Ok(Some(sub.clone())));
    repos
}

/// Replaying a dead letter redelivers it, marks it replayed and counts one more attempt.
#[tokio::test]
async fn replay_redelivers_and_marks_replayed() {
    let hook = Webhook::start(StatusCode::OK).await;
    let event = envelope("transfers:started");
    let mut repos = replaying(&event, &hook.url);
    repos
        .dead_letters
        .expect_mark_replayed()
        .withf(|id| id == "dl-1")
        .times(1)
        .returning(|_| Ok(()));
    repos
        .deliveries
        .expect_mark_delivered()
        .withf(|id, attempts, status| id == "delivery-1" && *attempts == 6 && *status == 200)
        .times(1)
        .returning(|_, _, _| Ok(()));

    let delivery = repos.bus().replay_dead_letter(&OwnerScope::All, "dl-1").await.unwrap();

    assert_eq!(delivery.status, DeliveryStatus::Delivered);
    assert_eq!(delivery.attempts, 6);
    assert_eq!(hook.received().len(), 1);
}

/// A replay that fails again is an error and leaves the dead letter unresolved.
#[tokio::test]
async fn failed_replay_keeps_the_dead_letter() {
    let hook = Webhook::start(StatusCode::INTERNAL_SERVER_ERROR).await;
    let event = envelope("transfers:started");
    let repos = replaying(&event, &hook.url);

    assert!(repos.bus().replay_dead_letter(&OwnerScope::All, "dl-1").await.is_err());
}

/// Replaying an unknown dead letter is not found.
#[tokio::test]
async fn replaying_an_unknown_dead_letter_is_not_found() {
    let mut repos = Repos::default();
    repos
        .dead_letters
        .expect_get_dead_letter()
        .returning(|_, _| Ok(None));

    assert!(repos
        .bus()
        .replay_dead_letter(&OwnerScope::All, "missing")
        .await
        .is_err());
}
