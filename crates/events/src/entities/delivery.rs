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

// Record representing an individual webhook delivery attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDeliveryRecord {
    pub id: String,
    pub tenant_id: String,
    pub event_id: String,
    pub subscription_id: String,
    pub status: DeliveryStatus,
    pub attempts: u32,
    pub last_attempt_at: Option<DateTime<Utc>>,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub response_status_code: Option<u16>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

// Delivery attempt lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeliveryStatus {
    Pending,
    Delivered,
    Failed,
    DeadLetter,
}

impl DeliveryStatus {
    // Static string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            DeliveryStatus::Pending => "Pending",
            DeliveryStatus::Delivered => "Delivered",
            DeliveryStatus::Failed => "Failed",
            DeliveryStatus::DeadLetter => "DeadLetter",
        }
    }
}

impl std::str::FromStr for DeliveryStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Pending" => Ok(DeliveryStatus::Pending),
            "Delivered" => Ok(DeliveryStatus::Delivered),
            "Failed" => Ok(DeliveryStatus::Failed),
            "DeadLetter" => Ok(DeliveryStatus::DeadLetter),
            other => Err(format!("unknown delivery status: {other}")),
        }
    }
}
