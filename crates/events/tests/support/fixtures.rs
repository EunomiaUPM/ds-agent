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

//! Envelopes, subscriptions, deliveries, a jitter-free policy and a bus over mocked repos.

use std::sync::Arc;

use chrono::Utc;
use events::data::repo::{
    MockEventDeadLetterRepo, MockEventDeliveryRepo, MockEventStoreRepo, MockEventSubscriptionRepo,
};
use events::entities::delivery::{DeliveryStatus, EventDeliveryRecord};
use events::entities::subscription::SubscriptionRecord;
use events::{EventBus, EventEnvelope, RetryPolicy, Topic, TopicPattern};
use serde_json::json;

pub const TENANT: &str = "tenant-1";

pub fn envelope(topic: &str) -> EventEnvelope {
    EventEnvelope::new(
        TENANT,
        Topic::new(topic).unwrap(),
        "tests",
        1,
        None,
        json!({ "transfer": "urn:uuid:1" }),
    )
}

/// Active subscription of [`TENANT`] to every topic.
pub fn subscription(
    id: &str,
    callback_address: &str,
    secret: Option<&str>,
    retry_limit: Option<u32>,
) -> SubscriptionRecord {
    SubscriptionRecord {
        id: id.to_string(),
        tenant_id: TENANT.to_string(),
        callback_address: callback_address.to_string(),
        topic_pattern: TopicPattern::match_all(),
        secret: secret.map(str::to_string),
        headers: None,
        retry_limit,
        active: true,
        created_at: Utc::now(),
        updated_at: None,
        expiration_time: None,
    }
}

/// Failed delivery of `event` to `subscription_id` after `attempts` attempts, due now.
pub fn delivery(
    id: &str,
    event: &EventEnvelope,
    subscription_id: &str,
    attempts: u32,
) -> EventDeliveryRecord {
    EventDeliveryRecord {
        id: id.to_string(),
        tenant_id: TENANT.to_string(),
        event_id: event.id.to_string(),
        subscription_id: subscription_id.to_string(),
        status: DeliveryStatus::Failed,
        attempts,
        last_attempt_at: Some(Utc::now()),
        next_retry_at: Some(Utc::now()),
        error_message: Some("HTTP 503".to_string()),
        response_status_code: Some(503),
        delivered_at: None,
        created_at: Utc::now(),
    }
}

/// Default policy without jitter and with a short timeout.
pub fn policy() -> RetryPolicy {
    RetryPolicy {
        jitter_factor: 0.0,
        timeout_secs: 2,
        ..RetryPolicy::default()
    }
}

/// Mocked repositories of the bus, configured by each test before building it.
#[derive(Default)]
pub struct Repos {
    pub events: MockEventStoreRepo,
    pub subscriptions: MockEventSubscriptionRepo,
    pub deliveries: MockEventDeliveryRepo,
    pub dead_letters: MockEventDeadLetterRepo,
}

impl Repos {
    pub fn bus(self) -> EventBus {
        EventBus::new(
            Arc::new(self.events),
            Arc::new(self.subscriptions),
            Arc::new(self.deliveries),
            Arc::new(self.dead_letters),
            policy(),
            16,
        )
    }
}
