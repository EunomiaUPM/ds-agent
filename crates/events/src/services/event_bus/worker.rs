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

use std::str::FromStr;
use std::sync::Arc;

use chrono::Utc;
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};
use urn::Urn;
use uuid::Uuid;

use crate::data::repo::{
    EventDeadLetterRepo, EventDeliveryRepo, EventStoreRepo, EventSubscriptionRepo,
};
use crate::entities::dead_letter::DeadLetterRecord;
use crate::entities::delivery::EventDeliveryRecord;
use crate::entities::subscription::DeadLetterStatus;
use crate::services::event_bus::dispatcher::EventDispatcher;
use crate::services::event_bus::policy::RetryPolicy;

// Background worker that periodically inspects and executes due webhook retries.
pub struct RetryWorker {
    event_repo: Arc<dyn EventStoreRepo>,
    subscription_repo: Arc<dyn EventSubscriptionRepo>,
    delivery_repo: Arc<dyn EventDeliveryRepo>,
    dlq_repo: Arc<dyn EventDeadLetterRepo>,
    dispatcher: Arc<EventDispatcher>,
    policy: RetryPolicy,
    concurrency_limit: usize,
}

impl RetryWorker {
    // Create a new RetryWorker with dependencies and retry policy.
    pub fn new(
        event_repo: Arc<dyn EventStoreRepo>,
        subscription_repo: Arc<dyn EventSubscriptionRepo>,
        delivery_repo: Arc<dyn EventDeliveryRepo>,
        dlq_repo: Arc<dyn EventDeadLetterRepo>,
        dispatcher: Arc<EventDispatcher>,
        policy: RetryPolicy,
    ) -> Self {
        Self {
            event_repo,
            subscription_repo,
            delivery_repo,
            dlq_repo,
            dispatcher,
            policy,
            concurrency_limit: 10,
        }
    }

    // Set maximum concurrent HTTP delivery tasks.
    pub fn with_concurrency_limit(mut self, limit: usize) -> Self {
        self.concurrency_limit = limit.max(1);
        self
    }

    // Run the poller loop until cancelled by the given CancellationToken.
    pub async fn run(self: Arc<Self>, cancel_token: CancellationToken) {
        let interval_duration = self.policy.poll_interval();
        info!(
            interval_secs = interval_duration.as_secs(),
            "Starting event bus retry worker loop"
        );

        let mut ticker = tokio::time::interval(interval_duration);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    info!("Event bus retry worker received cancellation signal, stopping");
                    break;
                }
                _ = ticker.tick() => {
                    if let Err(e) = self.process_batch().await {
                        error!(error = %e, "Error during retry worker batch execution");
                    }
                }
            }
        }
    }

    // Fetch and process one batch of due retries.
    pub async fn process_batch(&self) -> Result<usize, String> {
        let now = Utc::now();
        let due_deliveries = self
            .delivery_repo
            .get_due_retries(now, 50)
            .await
            .map_err(|e| format!("failed to query due retries: {e:?}"))?;

        if due_deliveries.is_empty() {
            return Ok(0);
        }

        let count = due_deliveries.len();
        debug!(count, "Found due deliveries for retry processing");

        let semaphore = Arc::new(Semaphore::new(self.concurrency_limit));
        let mut handles = Vec::with_capacity(count);

        for delivery in due_deliveries {
            let sem = semaphore.clone();
            let event_repo = self.event_repo.clone();
            let sub_repo = self.subscription_repo.clone();
            let delivery_repo = self.delivery_repo.clone();
            let dlq_repo = self.dlq_repo.clone();
            let dispatcher = self.dispatcher.clone();
            let policy = self.policy.clone();

            let handle = tokio::spawn(async move {
                let _permit = sem.acquire().await.ok();
                Self::retry_single_delivery(
                    delivery,
                    event_repo,
                    sub_repo,
                    delivery_repo,
                    dlq_repo,
                    dispatcher,
                    policy,
                )
                .await;
            });
            handles.push(handle);
        }

        for h in handles {
            let _ = h.await;
        }

        Ok(count)
    }

    // Execute retry attempt for a single delivery record.
    async fn retry_single_delivery(
        delivery: EventDeliveryRecord,
        event_repo: Arc<dyn EventStoreRepo>,
        sub_repo: Arc<dyn EventSubscriptionRepo>,
        delivery_repo: Arc<dyn EventDeliveryRepo>,
        dlq_repo: Arc<dyn EventDeadLetterRepo>,
        dispatcher: Arc<EventDispatcher>,
        policy: RetryPolicy,
    ) {
        let event_urn = match Urn::from_str(&delivery.event_id)
            .or_else(|_| Urn::from_str(&format!("urn:uuid:{}", delivery.event_id)))
        {
            Ok(u) => u,
            Err(e) => {
                error!(error = %e, event_id = %delivery.event_id, "Malformed event URN in delivery");
                return;
            }
        };

        let event = match event_repo.get_event_by_id(&event_urn).await {
            Ok(Some(ev)) => ev,
            Ok(None) => {
                error!(event_id = %delivery.event_id, "Referenced event not found during retry");
                return;
            }
            Err(e) => {
                error!(error = ?e, "Failed to load event for retry");
                return;
            }
        };

        let sub = match sub_repo.get_subscription(&delivery.subscription_id).await {
            Ok(Some(s)) => s,
            Ok(None) => {
                warn!(sub_id = %delivery.subscription_id, "Subscription not found; aborting retries");
                let _ = delivery_repo.mark_dead_letter(&delivery.id).await;
                return;
            }
            Err(e) => {
                error!(error = ?e, "Failed to load subscription for retry");
                return;
            }
        };

        let attempt = delivery.attempts + 1;
        let retry_limit = sub.retry_limit.unwrap_or(policy.max_attempts);

        match dispatcher
            .dispatch(
                &sub.callback_address,
                &event,
                sub.secret.as_deref(),
                sub.headers.as_ref(),
            )
            .await
        {
            Ok(status) if status.is_success() => {
                info!(delivery_id = %delivery.id, attempt, status = %status, "Retry succeeded");
                let _ = delivery_repo
                    .mark_delivered(&delivery.id, attempt, status.as_u16())
                    .await;
            }
            Ok(status) => {
                let err_msg = format!("HTTP {status}");
                let is_retryable = RetryPolicy::is_retryable_status(status.as_u16());
                if is_retryable && attempt < retry_limit {
                    let next_retry = policy.calculate_next_retry(attempt);
                    warn!(
                        delivery_id = %delivery.id,
                        attempt,
                        next_retry = %next_retry,
                        "Delivery retry failed with retryable status, scheduling next attempt"
                    );
                    let _ = delivery_repo
                        .record_failed_attempt(
                            &delivery.id,
                            attempt,
                            Some(next_retry),
                            &err_msg,
                            Some(status.as_u16()),
                        )
                        .await;
                } else {
                    error!(
                        delivery_id = %delivery.id,
                        attempt,
                        "Delivery exhausted retries or hit permanent failure; sending to DLQ"
                    );
                    let _ = delivery_repo
                        .record_failed_attempt(
                            &delivery.id,
                            attempt,
                            None,
                            &err_msg,
                            Some(status.as_u16()),
                        )
                        .await;
                    let _ = delivery_repo.mark_dead_letter(&delivery.id).await;

                    let dlq = DeadLetterRecord {
                        id: format!("urn:uuid:{}", Uuid::new_v4()),
                        delivery_id: Some(delivery.id.clone()),
                        event_id: event.id.to_string(),
                        subscription_id: sub.id.clone(),
                        topic: event.topic.to_string(),
                        callback_address: sub.callback_address.clone(),
                        payload: event.payload.clone(),
                        error_message: err_msg,
                        attempts: attempt,
                        status: DeadLetterStatus::Unresolved,
                        failed_at: Utc::now(),
                        replayed_at: None,
                    };
                    let _ = dlq_repo.create_dead_letter(&dlq).await;
                }
            }
            Err(err_msg) => {
                if attempt < retry_limit {
                    let next_retry = policy.calculate_next_retry(attempt);
                    warn!(
                        delivery_id = %delivery.id,
                        attempt,
                        next_retry = %next_retry,
                        error = %err_msg,
                        "Network error during retry, scheduling next attempt"
                    );
                    let _ = delivery_repo
                        .record_failed_attempt(
                            &delivery.id,
                            attempt,
                            Some(next_retry),
                            &err_msg,
                            None,
                        )
                        .await;
                } else {
                    error!(delivery_id = %delivery.id, attempt, "Retry attempts exhausted; routing to DLQ");
                    let _ = delivery_repo
                        .record_failed_attempt(&delivery.id, attempt, None, &err_msg, None)
                        .await;
                    let _ = delivery_repo.mark_dead_letter(&delivery.id).await;

                    let dlq = DeadLetterRecord {
                        id: format!("urn:uuid:{}", Uuid::new_v4()),
                        delivery_id: Some(delivery.id.clone()),
                        event_id: event.id.to_string(),
                        subscription_id: sub.id.clone(),
                        topic: event.topic.to_string(),
                        callback_address: sub.callback_address.clone(),
                        payload: event.payload.clone(),
                        error_message: err_msg,
                        attempts: attempt,
                        status: DeadLetterStatus::Unresolved,
                        failed_at: Utc::now(),
                        replayed_at: None,
                    };
                    let _ = dlq_repo.create_dead_letter(&dlq).await;
                }
            }
        }
    }
}
