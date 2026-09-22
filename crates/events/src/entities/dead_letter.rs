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

// Record representing an exhausted or permanent failure stored in the Dead Letter Queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadLetterRecord {
    pub id: String,
    pub tenant_id: String,
    pub delivery_id: Option<String>,
    pub event_id: String,
    pub subscription_id: String,
    pub topic: String,
    pub callback_address: String,
    pub payload: serde_json::Value,
    pub error_message: String,
    pub attempts: u32,
    pub status: DeadLetterStatus,
    pub failed_at: DateTime<Utc>,
    pub replayed_at: Option<DateTime<Utc>>,
}

// Dead letter resolution status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeadLetterStatus {
    Unresolved,
    Replayed,
    Purged,
}

impl DeadLetterStatus {
    // Static string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            DeadLetterStatus::Unresolved => "Unresolved",
            DeadLetterStatus::Replayed => "Replayed",
            DeadLetterStatus::Purged => "Purged",
        }
    }
}

impl std::str::FromStr for DeadLetterStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Unresolved" => Ok(DeadLetterStatus::Unresolved),
            "Replayed" => Ok(DeadLetterStatus::Replayed),
            "Purged" => Ok(DeadLetterStatus::Purged),
            other => Err(format!("unknown dead letter status: {other}")),
        }
    }
}
