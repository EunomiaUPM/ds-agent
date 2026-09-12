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

use crate::entities::subscription::DeliveryStatus;

// Record representing an individual webhook delivery attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDeliveryRecord {
    pub id: String,
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
