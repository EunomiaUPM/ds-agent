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

//! Create and update commands.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use common::oauth::Visibility;
use serde::{Deserialize, Serialize};

/// New webhook subscription of the caller; private unless `visibility` says otherwise.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSubscriptionDto {
    #[serde(default)]
    pub visibility: Option<Visibility>,
    pub callback_address: String,
    pub topic_pattern: String,
    pub secret: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub retry_limit: Option<u32>,
    pub expiration_time: Option<DateTime<Utc>>,
}

/// Partial subscription update; absent fields stay as they are.
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

/// Event published through the HTTP API, about a record of the caller (private unless
/// `visibility` says otherwise); the source defaults to `events`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishEventRequest {
    #[serde(default)]
    pub visibility: Option<Visibility>,
    pub topic: String,
    pub source_crate: Option<String>,
    pub schema_version: Option<u32>,
    pub correlation_id: Option<String>,
    pub payload: serde_json::Value,
}
