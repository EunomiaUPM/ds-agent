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

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// Command payload to register a new webhook subscription.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSubscriptionDto {
    pub callback_address: String,
    pub topic_pattern: String,
    pub secret: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub retry_limit: Option<u32>,
    pub expiration_time: Option<DateTime<Utc>>,
}

// Command payload to update an existing subscription.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateSubscriptionDto {
    pub callback_address: Option<String>,
    pub topic_pattern: Option<String>,
    pub secret: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub retry_limit: Option<u32>,
    pub active: Option<bool>,
    pub expiration_time: Option<DateTime<Utc>>,
}

// Command payload to publish a generic event via HTTP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishEventRequest {
    pub topic: String,
    pub source_crate: Option<String>,
    pub schema_version: Option<u32>,
    pub correlation_id: Option<String>,
    pub payload: serde_json::Value,
}
