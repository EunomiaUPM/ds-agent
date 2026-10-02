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

//! RetryWorker with mocked repositories and a local webhook: due deliveries are retried,
//! rescheduled or sent to the dead letter queue.

use std::sync::Arc;
use std::time::Duration;

use axum::http::StatusCode;
use events::data::repo::DeadLetterStatus;
use events::entities::envelope::EventEnvelope;
use events::{EventDispatcher, RetryWorker};

use crate::support::fixtures::{delivery, envelope, policy, subscription, Repos};
use crate::support::webhook::Webhook;

fn worker(repos: Repos) -> RetryWorker {
    RetryWorker::new(
        Arc::new(repos.events),
        Arc::new(repos.subscriptions),
        Arc::new(repos.deliveries),
        Arc::new(repos.dead_letters),
        Arc::new(EventDispatcher::new(Duration::from_secs(2))),
        policy(),
    )
}

/// Repos with one due delivery after `attempts` attempts, whose event and subscription exist.
fn due(event: &EventEnvelope, callback: &str, attempts: u32, retry_limit: Option<u32>) -> Repos {
    let mut repos = Repos::default();
    let pending = delivery("delivery-1", event, "sub-1", attempts);
    repos
        .deliveries
        .expect_get_due_retries()
        .returning(move |_, _| Ok(vec![pending.clone()]));
    let stored = event.clone();
    repos
        .events
        .expect_get_event_by_id()
        .returning(move |_, _| Ok(Some(stored.clone())));
    let sub = subscription("sub-1", callback, None, retry_limit);
    repos
        .subscriptions
        .expect_get_subscription()
        .returning(move |_, _| Ok(Some(sub.clone())));
    repos
}

/// With nothing due the worker processes nothing.
#[tokio::test]
async fn nothing_due_processes_nothing() {
    let mut repos = Repos::default();
    repos
        .deliveries
        .expect_get_due_retries()
        .returning(|_, _| Ok(vec![]));
    assert_eq!(worker(repos).process_batch().await.unwrap(), 0);
}

/// A due delivery that now succeeds is marked delivered with the next attempt number.
#[tokio::test]
async fn successful_retry_is_marked_delivered() {
    let hook = Webhook::start(StatusCode::OK).await;
    let event = envelope("transfers:started");
    let mut repos = due(&event, &hook.url, 2, None);
    repos
        .deliveries
        .expect_mark_delivered()
        .withf(|id, attempts, status| id == "delivery-1" && *attempts == 3 && *status == 200)
        .times(1)
        .returning(|_, _, _| Ok(()));

    assert_eq!(worker(repos).process_batch().await.unwrap(), 1);
    assert_eq!(hook.received().len(), 1);
}

/// A retryable failure under the limit is rescheduled.
#[tokio::test]
async fn retryable_failure_under_the_limit_is_rescheduled() {
    let hook = Webhook::start(StatusCode::TOO_MANY_REQUESTS).await;
    let event = envelope("transfers:started");
    let mut repos = due(&event, &hook.url, 1, Some(5));
    repos
        .deliveries
        .expect_record_failed_attempt()
        .withf(|_, attempts, next, _, status| {
            *attempts == 2 && next.is_some() && *status == Some(429)
        })
        .times(1)
        .returning(|_, _, _, _, _| Ok(()));

    worker(repos).process_batch().await.unwrap();
}

/// The last allowed attempt failing moves the delivery to the dead letter queue.
#[tokio::test]
async fn exhausted_delivery_goes_to_the_dead_letter_queue() {
    let hook = Webhook::start(StatusCode::SERVICE_UNAVAILABLE).await;
    let event = envelope("transfers:started");
    let mut repos = due(&event, &hook.url, 4, Some(5));
    repos
        .deliveries
        .expect_record_failed_attempt()
        .withf(|_, attempts, next, _, _| *attempts == 5 && next.is_none())
        .times(1)
        .returning(|_, _, _, _, _| Ok(()));
    repos
        .deliveries
        .expect_mark_dead_letter()
        .times(1)
        .returning(|_| Ok(()));
    repos
        .dead_letters
        .expect_create_dead_letter()
        .withf(|dl| dl.attempts == 5 && dl.status == DeadLetterStatus::Unresolved)
        .times(1)
        .returning(|dl| Ok(dl.clone()));

    worker(repos).process_batch().await.unwrap();
}

/// A delivery whose subscription is gone is dead-lettered without calling any webhook.
#[tokio::test]
async fn missing_subscription_stops_the_retries() {
    let event = envelope("transfers:started");
    let mut repos = Repos::default();
    let pending = delivery("delivery-1", &event, "sub-1", 1);
    repos
        .deliveries
        .expect_get_due_retries()
        .returning(move |_, _| Ok(vec![pending.clone()]));
    let stored = event.clone();
    repos
        .events
        .expect_get_event_by_id()
        .returning(move |_, _| Ok(Some(stored.clone())));
    repos
        .subscriptions
        .expect_get_subscription()
        .returning(|_, _| Ok(None));
    repos
        .deliveries
        .expect_mark_dead_letter()
        .withf(|id| id == "delivery-1")
        .times(1)
        .returning(|_| Ok(()));

    worker(repos).process_batch().await.unwrap();
}
