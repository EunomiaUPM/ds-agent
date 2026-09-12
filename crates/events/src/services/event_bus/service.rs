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

use async_trait::async_trait;
use chrono::Utc;
use tokio::sync::broadcast;
use tracing::{error, info, warn};
use urn::Urn;
use uuid::Uuid;

use crate::data::repo::{
    EventDeadLetterRepo, EventDeliveryRepo, EventStoreRepo, EventSubscriptionRepo,
};
use crate::entities::dead_letter::DeadLetterRecord;
use crate::entities::delivery::EventDeliveryRecord;
use crate::entities::envelope::EventEnvelope;
use crate::entities::subscription::{DeadLetterStatus, DeliveryStatus};
use crate::entities::traits::Event;
use crate::errors::EventBusError;
use crate::services::event_bus::dispatcher::EventDispatcher;
use crate::services::event_bus::policy::RetryPolicy;
use crate::services::event_bus::{EventBusTrait, EventPublisherTrait};

// Central event bus orchestrating event persistence, broadcasting, and delivery.
#[derive(Clone)]
pub struct EventBus {
    event_repo: Arc<dyn EventStoreRepo>,
    subscription_repo: Arc<dyn EventSubscriptionRepo>,
    delivery_repo: Arc<dyn EventDeliveryRepo>,
    dlq_repo: Arc<dyn EventDeadLetterRepo>,
    dispatcher: Arc<EventDispatcher>,
    policy: RetryPolicy,
    broadcast_tx: broadcast::Sender<EventEnvelope>,
}

impl EventBus {
    // Initialize EventBus with repositories, policy, and broadcast channel capacity.
    pub fn new(
        event_repo: Arc<dyn EventStoreRepo>,
        subscription_repo: Arc<dyn EventSubscriptionRepo>,
        delivery_repo: Arc<dyn EventDeliveryRepo>,
        dlq_repo: Arc<dyn EventDeadLetterRepo>,
        policy: RetryPolicy,
        broadcast_capacity: usize,
    ) -> Self {
        let dispatcher = Arc::new(EventDispatcher::new(policy.timeout()));
        let (broadcast_tx, _) = broadcast::channel(broadcast_capacity.max(16));

        Self {
            event_repo,
            subscription_repo,
            delivery_repo,
            dlq_repo,
            dispatcher,
            policy,
            broadcast_tx,
        }
    }

    // Publish a strongly-typed domain event instance.
    pub async fn emit<E: Event>(&self, event: E) -> Result<EventEnvelope, EventBusError> {
        <Self as EventBusTrait>::publish(self, event.into_envelope()).await
    }

    // Publish an event envelope into the event store and broadcast channels.
    pub async fn publish(&self, envelope: EventEnvelope) -> Result<EventEnvelope, EventBusError> {
        <Self as EventBusTrait>::publish(self, envelope).await
    }

    // Publish a typed domain event into the event bus.
    pub async fn publish_event<E: Event>(&self, event: E) -> Result<EventEnvelope, EventBusError> {
        <Self as EventPublisherTrait>::publish_event(self, event).await
    }

    // Subscribe to the in-process event broadcast stream.
    pub fn subscribe(&self) -> broadcast::Receiver<EventEnvelope> {
        <Self as EventBusTrait>::subscribe(self)
    }

    // Access underlying retry policy.
    pub fn policy(&self) -> &RetryPolicy {
        &self.policy
    }

    // Access event store repository.
    pub fn event_repo(&self) -> Arc<dyn EventStoreRepo> {
        self.event_repo.clone()
    }

    // Access subscription repository.
    pub fn subscription_repo(&self) -> Arc<dyn EventSubscriptionRepo> {
        self.subscription_repo.clone()
    }

    // Access delivery repository.
    pub fn delivery_repo(&self) -> Arc<dyn EventDeliveryRepo> {
        self.delivery_repo.clone()
    }

    // Access dead letter queue repository.
    pub fn dlq_repo(&self) -> Arc<dyn EventDeadLetterRepo> {
        self.dlq_repo.clone()
    }

    // Access HTTP event dispatcher.
    pub fn dispatcher(&self) -> Arc<EventDispatcher> {
        self.dispatcher.clone()
    }

    // Replay a single dead letter record by ID.
    pub async fn replay_dead_letter(
        &self,
        dlq_id: &str,
    ) -> Result<EventDeliveryRecord, EventBusError> {
        let record = self
            .dlq_repo
            .get_dead_letter(dlq_id)
            .await
            .map_err(|e| EventBusError::Database(format!("{e:?}")))?
            .ok_or_else(|| {
                EventBusError::DeadLetterNotFound(Uuid::parse_str(dlq_id).unwrap_or_default())
            })?;

        let event_urn = Urn::from_str(&record.event_id)
            .or_else(|_| Urn::from_str(&format!("urn:uuid:{}", record.event_id)))
            .map_err(|e| EventBusError::Database(format!("invalid event URN: {e}")))?;

        let event = self
            .event_repo
            .get_event_by_id(&event_urn)
            .await
            .map_err(|e| EventBusError::Database(format!("{e:?}")))?
            .ok_or_else(|| {
                EventBusError::EventNotFound(Uuid::parse_str(&record.event_id).unwrap_or_default())
            })?;

        let sub = self
            .subscription_repo
            .get_subscription(&record.subscription_id)
            .await
            .map_err(|e| EventBusError::Database(format!("{e:?}")))?
            .ok_or_else(|| {
                EventBusError::SubscriptionNotFound(
                    Uuid::parse_str(&record.subscription_id).unwrap_or_default(),
                )
            })?;

        match self
            .dispatcher
            .dispatch(
                &sub.callback_address,
                &event,
                sub.secret.as_deref(),
                sub.headers.as_ref(),
            )
            .await
        {
            Ok(status) if status.is_success() => {
                info!(dlq_id, status = %status, "Dead letter replayed successfully");
                self.dlq_repo
                    .mark_replayed(dlq_id)
                    .await
                    .map_err(|e| EventBusError::Database(format!("{e:?}")))?;

                if let Some(delivery_id) = &record.delivery_id {
                    let _ = self
                        .delivery_repo
                        .mark_delivered(delivery_id, record.attempts + 1, status.as_u16())
                        .await;
                }

                let delivery_record = EventDeliveryRecord {
                    id: record
                        .delivery_id
                        .unwrap_or_else(|| format!("urn:uuid:{}", Uuid::new_v4())),
                    event_id: record.event_id,
                    subscription_id: record.subscription_id,
                    status: DeliveryStatus::Delivered,
                    attempts: record.attempts + 1,
                    last_attempt_at: Some(Utc::now()),
                    next_retry_at: None,
                    error_message: None,
                    response_status_code: Some(status.as_u16()),
                    delivered_at: Some(Utc::now()),
                    created_at: record.failed_at,
                };
                Ok(delivery_record)
            }
            Ok(status) => {
                let err = format!("Replay failed with HTTP {status}");
                Err(EventBusError::DispatchFailed(err))
            }
            Err(e) => Err(EventBusError::DispatchFailed(e)),
        }
    }

    // Replay all unresolved dead letter records in batches.
    pub async fn replay_all_dead_letters(&self) -> Result<usize, EventBusError> {
        let mut success_count = 0;
        let mut offset = 0;
        let limit = 50;

        loop {
            let dead_letters = self
                .dlq_repo
                .list_dead_letters(Some("Unresolved"), limit, offset)
                .await
                .map_err(|e| EventBusError::Database(format!("{e:?}")))?;

            if dead_letters.is_empty() {
                break;
            }

            let batch_len = dead_letters.len();
            for dl in dead_letters {
                if self.replay_dead_letter(&dl.id).await.is_ok() {
                    success_count += 1;
                } else {
                    offset += 1;
                }
            }

            if batch_len < limit as usize {
                break;
            }
        }

        Ok(success_count)
    }

    // Spawn an immediate delivery attempt in a background task.
    #[allow(clippy::too_many_arguments)]
    fn spawn_immediate_dispatch(
        &self,
        delivery_id: String,
        event: EventEnvelope,
        sub_id: String,
        callback_address: String,
        secret: Option<String>,
        headers: Option<std::collections::HashMap<String, String>>,
        retry_limit: u32,
    ) {
        let dispatcher = self.dispatcher.clone();
        let delivery_repo = self.delivery_repo.clone();
        let dlq_repo = self.dlq_repo.clone();
        let policy = self.policy.clone();

        tokio::spawn(async move {
            match dispatcher
                .dispatch(
                    &callback_address,
                    &event,
                    secret.as_deref(),
                    headers.as_ref(),
                )
                .await
            {
                Ok(status) if status.is_success() => {
                    info!(delivery_id = %delivery_id, status = %status, "Immediate delivery succeeded");
                    let _ = delivery_repo
                        .mark_delivered(&delivery_id, 1, status.as_u16())
                        .await;
                }
                Ok(status) => {
                    let err_msg = format!("HTTP {status}");
                    let is_retryable = RetryPolicy::is_retryable_status(status.as_u16());
                    if is_retryable && 1 < retry_limit {
                        let next_retry = policy.calculate_next_retry(1);
                        warn!(
                            delivery_id = %delivery_id,
                            next_retry = %next_retry,
                            "Initial delivery attempt failed, scheduling retry"
                        );
                        let _ = delivery_repo
                            .record_failed_attempt(
                                &delivery_id,
                                1,
                                Some(next_retry),
                                &err_msg,
                                Some(status.as_u16()),
                            )
                            .await;
                    } else {
                        error!(delivery_id = %delivery_id, "Delivery failed and non-retryable; sending to DLQ");
                        let _ = delivery_repo
                            .record_failed_attempt(
                                &delivery_id,
                                1,
                                None,
                                &err_msg,
                                Some(status.as_u16()),
                            )
                            .await;
                        let _ = delivery_repo.mark_dead_letter(&delivery_id).await;

                        let dlq_record = DeadLetterRecord {
                            id: format!("urn:uuid:{}", Uuid::new_v4()),
                            delivery_id: Some(delivery_id),
                            event_id: event.id.to_string(),
                            subscription_id: sub_id,
                            topic: event.topic.to_string(),
                            callback_address,
                            payload: event.payload,
                            error_message: err_msg,
                            attempts: 1,
                            status: DeadLetterStatus::Unresolved,
                            failed_at: Utc::now(),
                            replayed_at: None,
                        };
                        let _ = dlq_repo.create_dead_letter(&dlq_record).await;
                    }
                }
                Err(err_msg) => {
                    if 1 < retry_limit {
                        let next_retry = policy.calculate_next_retry(1);
                        warn!(
                            delivery_id = %delivery_id,
                            next_retry = %next_retry,
                            error = %err_msg,
                            "Initial dispatch error, scheduling retry"
                        );
                        let _ = delivery_repo
                            .record_failed_attempt(
                                &delivery_id,
                                1,
                                Some(next_retry),
                                &err_msg,
                                None,
                            )
                            .await;
                    } else {
                        let _ = delivery_repo
                            .record_failed_attempt(&delivery_id, 1, None, &err_msg, None)
                            .await;
                        let _ = delivery_repo.mark_dead_letter(&delivery_id).await;

                        let dlq_record = DeadLetterRecord {
                            id: format!("urn:uuid:{}", Uuid::new_v4()),
                            delivery_id: Some(delivery_id),
                            event_id: event.id.to_string(),
                            subscription_id: sub_id,
                            topic: event.topic.to_string(),
                            callback_address,
                            payload: event.payload,
                            error_message: err_msg,
                            attempts: 1,
                            status: DeadLetterStatus::Unresolved,
                            failed_at: Utc::now(),
                            replayed_at: None,
                        };
                        let _ = dlq_repo.create_dead_letter(&dlq_record).await;
                    }
                }
            }
        });
    }
}

#[async_trait]
impl EventBusTrait for EventBus {
    async fn publish(&self, envelope: EventEnvelope) -> Result<EventEnvelope, EventBusError> {
        self.event_repo
            .insert_event(&envelope)
            .await
            .map_err(|e| EventBusError::Database(format!("{e:?}")))?;

        let _ = self.broadcast_tx.send(envelope.clone());

        let matching_subs = self
            .subscription_repo
            .get_matching_subscriptions(&envelope.topic)
            .await
            .map_err(|e| EventBusError::Database(format!("{e:?}")))?;

        for sub in matching_subs {
            let delivery_id = format!("urn:uuid:{}", Uuid::new_v4());
            let delivery = EventDeliveryRecord {
                id: delivery_id.clone(),
                event_id: envelope.id.to_string(),
                subscription_id: sub.id.clone(),
                status: DeliveryStatus::Pending,
                attempts: 0,
                last_attempt_at: None,
                next_retry_at: None,
                error_message: None,
                response_status_code: None,
                delivered_at: None,
                created_at: Utc::now(),
            };

            if let Err(e) = self.delivery_repo.create_delivery(&delivery).await {
                error!(error = ?e, sub_id = %sub.id, "Failed to create delivery record");
                continue;
            }

            let retry_limit = sub.retry_limit.unwrap_or(self.policy.max_attempts);
            self.spawn_immediate_dispatch(
                delivery_id,
                envelope.clone(),
                sub.id,
                sub.callback_address,
                sub.secret,
                sub.headers,
                retry_limit,
            );
        }

        Ok(envelope)
    }

    fn subscribe(&self) -> broadcast::Receiver<EventEnvelope> {
        self.broadcast_tx.subscribe()
    }
}

#[async_trait]
impl EventPublisherTrait for EventBus {
    async fn publish_event<E: Event>(&self, event: E) -> Result<EventEnvelope, EventBusError> {
        self.publish(event.into_envelope()).await
    }
}
