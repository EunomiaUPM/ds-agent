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

use crate::entities::topic::Topic;
use crate::entities::topic_pattern::TopicPattern;

// Domain representation of an external webhook subscription.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionRecord {
    pub id: String,
    pub tenant_id: String,
    pub callback_address: String,
    pub topic_pattern: TopicPattern,
    pub secret: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub retry_limit: Option<u32>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub expiration_time: Option<DateTime<Utc>>,
}

impl SubscriptionRecord {
    // Check if this subscription is active and has not expired.
    pub fn is_active(&self) -> bool {
        if !self.active {
            return false;
        }
        if let Some(exp) = self.expiration_time {
            if exp <= Utc::now() {
                return false;
            }
        }
        true
    }

    // Verify if a topic matches this subscription pattern.
    pub fn matches(&self, topic: &Topic) -> bool {
        self.is_active() && self.topic_pattern.matches(topic)
    }
}
