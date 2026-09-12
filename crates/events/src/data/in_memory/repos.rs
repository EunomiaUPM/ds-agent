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
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use urn::Urn;
use uuid::Uuid;
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::data::repo::{
    EventDeadLetterRepo, EventDeliveryRepo, EventStoreRepo, EventSubscriptionRepo,
};
use crate::entities::commands::{CreateSubscriptionDto, UpdateSubscriptionDto};
use crate::entities::dead_letter::DeadLetterRecord;
use crate::entities::delivery::EventDeliveryRecord;
use crate::entities::envelope::EventEnvelope;
use crate::entities::subscription::{DeadLetterStatus, DeliveryStatus, SubscriptionRecord};
use crate::entities::topic::{Topic, TopicPattern};

// In-memory repository implementing all event bus storage traits for lightweight tests and mocks.
#[derive(Clone, Default)]
pub struct InMemoryEventBusRepo {
    events: Arc<Mutex<HashMap<String, EventEnvelope>>>,
    subscriptions: Arc<Mutex<HashMap<String, SubscriptionRecord>>>,
    deliveries: Arc<Mutex<HashMap<String, EventDeliveryRecord>>>,
    dead_letters: Arc<Mutex<HashMap<String, DeadLetterRecord>>>,
}

impl InMemoryEventBusRepo {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl EventStoreRepo for InMemoryEventBusRepo {
    async fn insert_event(&self, event: &EventEnvelope) -> Outcome<()> {
        let mut map = self.events.lock().unwrap();
        map.insert(event.id.to_string(), event.clone());
        Ok(())
    }

    async fn get_event_by_id(&self, id: &Urn) -> Outcome<Option<EventEnvelope>> {
        let map = self.events.lock().unwrap();
        Ok(map.get(id.as_str()).cloned())
    }

    async fn list_events(
        &self,
        topic: Option<&str>,
        limit: u64,
        offset: u64,
    ) -> Outcome<Vec<EventEnvelope>> {
        let pattern = topic.and_then(|t| TopicPattern::new(t).ok());
        let map = self.events.lock().unwrap();
        let mut list: Vec<EventEnvelope> = map
            .values()
            .filter(|e| {
                if let Some(ref pat) = pattern {
                    pat.matches(&e.topic)
                } else {
                    true
                }
            })
            .cloned()
            .collect();
        list.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        let paged = list
            .into_iter()
            .skip(offset as usize)
            .take(limit as usize)
            .collect();
        Ok(paged)
    }
}

#[async_trait]
impl EventSubscriptionRepo for InMemoryEventBusRepo {
    async fn create_subscription(&self, dto: CreateSubscriptionDto) -> Outcome<SubscriptionRecord> {
        let id = format!("urn:uuid:{}", Uuid::new_v4());
        let pattern = TopicPattern::new(&dto.topic_pattern)
            .map_err(|e| Errors::format(BadFormat::Received, e, None))?;

        let rec = SubscriptionRecord {
            id: id.clone(),
            callback_address: dto.callback_address,
            topic_pattern: pattern,
            secret: dto.secret,
            headers: dto.headers,
            retry_limit: dto.retry_limit,
            active: true,
            created_at: Utc::now(),
            updated_at: None,
            expiration_time: dto.expiration_time,
        };

        let mut map = self.subscriptions.lock().unwrap();
        map.insert(id, rec.clone());
        Ok(rec)
    }

    async fn get_subscription(&self, id: &str) -> Outcome<Option<SubscriptionRecord>> {
        let map = self.subscriptions.lock().unwrap();
        Ok(map.get(id).cloned())
    }

    async fn list_subscriptions(&self) -> Outcome<Vec<SubscriptionRecord>> {
        let map = self.subscriptions.lock().unwrap();
        Ok(map.values().cloned().collect())
    }

    async fn update_subscription(
        &self,
        id: &str,
        dto: UpdateSubscriptionDto,
    ) -> Outcome<SubscriptionRecord> {
        let mut map = self.subscriptions.lock().unwrap();
        let rec = map
            .get_mut(id)
            .ok_or_else(|| Errors::missing_resource(id, "subscription not found", None))?;

        if let Some(addr) = dto.callback_address {
            rec.callback_address = addr;
        }
        if let Some(pat) = dto.topic_pattern {
            rec.topic_pattern = TopicPattern::new(&pat)
                .map_err(|e| Errors::format(BadFormat::Received, e, None))?;
        }
        if dto.secret.is_some() {
            rec.secret = dto.secret;
        }
        if dto.headers.is_some() {
            rec.headers = dto.headers;
        }
        if dto.retry_limit.is_some() {
            rec.retry_limit = dto.retry_limit;
        }
        if let Some(act) = dto.active {
            rec.active = act;
        }
        if dto.expiration_time.is_some() {
            rec.expiration_time = dto.expiration_time;
        }
        rec.updated_at = Some(Utc::now());

        Ok(rec.clone())
    }

    async fn delete_subscription(&self, id: &str) -> Outcome<()> {
        let mut map = self.subscriptions.lock().unwrap();
        map.remove(id);
        Ok(())
    }

    async fn get_matching_subscriptions(&self, topic: &Topic) -> Outcome<Vec<SubscriptionRecord>> {
        let map = self.subscriptions.lock().unwrap();
        let matching = map
            .values()
            .filter(|sub| sub.matches(topic))
            .cloned()
            .collect();
        Ok(matching)
    }
}

#[async_trait]
impl EventDeliveryRepo for InMemoryEventBusRepo {
    async fn create_delivery(
        &self,
        delivery: &EventDeliveryRecord,
    ) -> Outcome<EventDeliveryRecord> {
        let mut map = self.deliveries.lock().unwrap();
        map.insert(delivery.id.clone(), delivery.clone());
        Ok(delivery.clone())
    }

    async fn get_delivery(&self, id: &str) -> Outcome<Option<EventDeliveryRecord>> {
        let map = self.deliveries.lock().unwrap();
        Ok(map.get(id).cloned())
    }

    async fn get_due_retries(
        &self,
        now: DateTime<Utc>,
        limit: u64,
    ) -> Outcome<Vec<EventDeliveryRecord>> {
        let map = self.deliveries.lock().unwrap();
        let retries = map
            .values()
            .filter(|d| {
                d.status == DeliveryStatus::Failed
                    && d.next_retry_at.map(|t| t <= now).unwrap_or(false)
            })
            .take(limit as usize)
            .cloned()
            .collect();
        Ok(retries)
    }

    async fn mark_delivered(&self, id: &str, attempts: u32, status_code: u16) -> Outcome<()> {
        let mut map = self.deliveries.lock().unwrap();
        if let Some(d) = map.get_mut(id) {
            d.status = DeliveryStatus::Delivered;
            d.attempts = attempts;
            d.last_attempt_at = Some(Utc::now());
            d.delivered_at = Some(Utc::now());
            d.response_status_code = Some(status_code);
            d.error_message = None;
            d.next_retry_at = None;
        }
        Ok(())
    }

    async fn record_failed_attempt(
        &self,
        id: &str,
        attempts: u32,
        next_retry_at: Option<DateTime<Utc>>,
        error: &str,
        status_code: Option<u16>,
    ) -> Outcome<()> {
        let mut map = self.deliveries.lock().unwrap();
        if let Some(d) = map.get_mut(id) {
            d.status = DeliveryStatus::Failed;
            d.attempts = attempts;
            d.last_attempt_at = Some(Utc::now());
            d.next_retry_at = next_retry_at;
            d.error_message = Some(error.to_string());
            d.response_status_code = status_code;
        }
        Ok(())
    }

    async fn mark_dead_letter(&self, id: &str) -> Outcome<()> {
        let mut map = self.deliveries.lock().unwrap();
        if let Some(d) = map.get_mut(id) {
            d.status = DeliveryStatus::DeadLetter;
            d.next_retry_at = None;
        }
        Ok(())
    }

    async fn list_by_event(&self, event_id: &str) -> Outcome<Vec<EventDeliveryRecord>> {
        let map = self.deliveries.lock().unwrap();
        let list = map
            .values()
            .filter(|d| d.event_id == event_id)
            .cloned()
            .collect();
        Ok(list)
    }
}

#[async_trait]
impl EventDeadLetterRepo for InMemoryEventBusRepo {
    async fn create_dead_letter(&self, record: &DeadLetterRecord) -> Outcome<DeadLetterRecord> {
        let mut map = self.dead_letters.lock().unwrap();
        map.insert(record.id.clone(), record.clone());
        Ok(record.clone())
    }

    async fn get_dead_letter(&self, id: &str) -> Outcome<Option<DeadLetterRecord>> {
        let map = self.dead_letters.lock().unwrap();
        Ok(map.get(id).cloned())
    }

    async fn list_dead_letters(
        &self,
        status: Option<&str>,
        limit: u64,
        offset: u64,
    ) -> Outcome<Vec<DeadLetterRecord>> {
        let map = self.dead_letters.lock().unwrap();
        let mut list: Vec<DeadLetterRecord> = map
            .values()
            .filter(|d| status.map(|s| d.status.as_str() == s).unwrap_or(true))
            .cloned()
            .collect();
        list.sort_by(|a, b| b.failed_at.cmp(&a.failed_at));
        let paged = list
            .into_iter()
            .skip(offset as usize)
            .take(limit as usize)
            .collect();
        Ok(paged)
    }

    async fn mark_replayed(&self, id: &str) -> Outcome<()> {
        let mut map = self.dead_letters.lock().unwrap();
        if let Some(d) = map.get_mut(id) {
            d.status = DeadLetterStatus::Replayed;
            d.replayed_at = Some(Utc::now());
        }
        Ok(())
    }

    async fn delete_dead_letter(&self, id: &str) -> Outcome<()> {
        let mut map = self.dead_letters.lock().unwrap();
        map.remove(id);
        Ok(())
    }
}
