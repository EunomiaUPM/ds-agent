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

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::entities::dead_letter::DeadLetterRecord;
use crate::entities::delivery::EventDeliveryRecord;
use crate::entities::envelope::EventEnvelope;
use crate::entities::subscription::SubscriptionRecord;

// View DTO representing a published domain event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventView {
    pub id: String,
    pub topic: String,
    pub source_crate: String,
    pub schema_version: u32,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: Option<String>,
    pub payload: serde_json::Value,
}

impl EventView {
    // Assemble EventView from pure domain EventEnvelope.
    pub fn assemble(envelope: EventEnvelope) -> Self {
        Self {
            id: envelope.id.to_string(),
            topic: envelope.topic.to_string(),
            source_crate: envelope.source_crate,
            schema_version: envelope.schema_version,
            timestamp: envelope.timestamp,
            correlation_id: envelope.correlation_id.map(|u| u.to_string()),
            payload: envelope.payload,
        }
    }
}

// View DTO representing a webhook subscription.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionView {
    pub id: String,
    pub callback_address: String,
    pub topic_pattern: String,
    pub active: bool,
    pub retry_limit: Option<u32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub expiration_time: Option<DateTime<Utc>>,
}

impl SubscriptionView {
    // Assemble SubscriptionView from domain SubscriptionRecord.
    pub fn assemble(record: SubscriptionRecord) -> Self {
        Self {
            id: record.id,
            callback_address: record.callback_address,
            topic_pattern: record.topic_pattern.to_string(),
            active: record.active,
            retry_limit: record.retry_limit,
            created_at: record.created_at,
            updated_at: record.updated_at,
            expiration_time: record.expiration_time,
        }
    }
}

// View DTO representing a delivery attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryView {
    pub id: String,
    pub event_id: String,
    pub subscription_id: String,
    pub status: String,
    pub attempts: u32,
    pub last_attempt_at: Option<DateTime<Utc>>,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub response_status_code: Option<u16>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl DeliveryView {
    // Assemble DeliveryView from domain EventDeliveryRecord.
    pub fn assemble(delivery: EventDeliveryRecord) -> Self {
        Self {
            id: delivery.id,
            event_id: delivery.event_id,
            subscription_id: delivery.subscription_id,
            status: delivery.status.as_str().to_string(),
            attempts: delivery.attempts,
            last_attempt_at: delivery.last_attempt_at,
            next_retry_at: delivery.next_retry_at,
            error_message: delivery.error_message,
            response_status_code: delivery.response_status_code,
            delivered_at: delivery.delivered_at,
            created_at: delivery.created_at,
        }
    }
}

// View DTO representing an entry in the Dead Letter Queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadLetterView {
    pub id: String,
    pub delivery_id: Option<String>,
    pub event_id: String,
    pub subscription_id: String,
    pub topic: String,
    pub callback_address: String,
    pub payload: serde_json::Value,
    pub error_message: String,
    pub attempts: u32,
    pub status: String,
    pub failed_at: DateTime<Utc>,
    pub replayed_at: Option<DateTime<Utc>>,
}

impl DeadLetterView {
    // Assemble DeadLetterView from domain DeadLetterRecord.
    pub fn assemble(dlq: DeadLetterRecord) -> Self {
        Self {
            id: dlq.id,
            delivery_id: dlq.delivery_id,
            event_id: dlq.event_id,
            subscription_id: dlq.subscription_id,
            topic: dlq.topic,
            callback_address: dlq.callback_address,
            payload: dlq.payload,
            error_message: dlq.error_message,
            attempts: dlq.attempts,
            status: dlq.status.as_str().to_string(),
            failed_at: dlq.failed_at,
            replayed_at: dlq.replayed_at,
        }
    }
}
