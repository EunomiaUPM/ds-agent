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

//! Listing filters of the event feed, webhook subscriptions and dead letters.

use common::paginated_spec::deserialize_opt_bool_from_str_or_bool;
use common::query::QueryFilter;
use serde::Deserialize;
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::entities::topic_pattern::TopicPattern;

/// `topic` is a topic pattern: `*` matches one segment, `**` any number of them.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct EventFilter {
    pub topic: Option<String>,
}

impl EventFilter {
    pub fn topic_pattern(&self) -> Outcome<Option<TopicPattern>> {
        self.topic
            .as_deref()
            .map(|t| TopicPattern::new(t).map_err(|e| Errors::format(BadFormat::Received, e, None)))
            .transpose()
    }
}

impl QueryFilter for EventFilter {
    fn is_empty(&self) -> bool {
        self.topic.is_none()
    }

    fn validate(&self) -> Outcome<()> {
        self.topic_pattern().map(|_| ())
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SubscriptionFilter {
    #[serde(default, deserialize_with = "deserialize_opt_bool_from_str_or_bool")]
    pub active: Option<bool>,
}

impl QueryFilter for SubscriptionFilter {
    fn is_empty(&self) -> bool {
        self.active.is_none()
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct DeadLetterFilter {
    pub status: Option<String>,
}

impl QueryFilter for DeadLetterFilter {
    fn is_empty(&self) -> bool {
        self.status.is_none()
    }
}
