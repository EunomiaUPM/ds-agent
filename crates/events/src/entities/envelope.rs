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

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use urn::Urn;
use uuid::Uuid;

use crate::entities::topic::Topic;

// Immutable envelope packaging domain events for storage and delivery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub id: Urn,
    pub topic: Topic,
    pub source_crate: String,
    pub schema_version: u32,
    pub timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<Urn>,
    pub payload: serde_json::Value,
}

impl EventEnvelope {
    // Construct a new event envelope with generated URN id and current timestamp.
    pub fn new(
        topic: Topic,
        source_crate: impl Into<String>,
        schema_version: u32,
        correlation_id: Option<Urn>,
        payload: serde_json::Value,
    ) -> Self {
        let id_str = format!("urn:uuid:{}", Uuid::new_v4());
        let id = Urn::from_str(&id_str).expect("valid URN format");
        Self {
            id,
            topic,
            source_crate: source_crate.into(),
            schema_version,
            timestamp: Utc::now(),
            correlation_id,
            payload,
        }
    }

    // Construct an envelope with explicit metadata for persistence rehydration.
    pub fn with_metadata(
        id: Urn,
        topic: Topic,
        source_crate: impl Into<String>,
        schema_version: u32,
        timestamp: DateTime<Utc>,
        correlation_id: Option<Urn>,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            id,
            topic,
            source_crate: source_crate.into(),
            schema_version,
            timestamp,
            correlation_id,
            payload,
        }
    }
}
