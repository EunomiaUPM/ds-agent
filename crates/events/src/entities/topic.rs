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

use std::fmt::{Display, Formatter};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

// Validated topic path identifying an event category (supports . and : delimiters).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Topic(String);

impl Topic {
    // Validate and create a topic supporting dot and colon delimiters.
    pub fn new(topic: impl Into<String>) -> Result<Self, String> {
        let s = topic.into();
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err("topic cannot be empty".to_string());
        }
        if trimmed.contains('*') || trimmed.contains('?') {
            return Err("topic cannot contain wildcard characters".to_string());
        }
        let segments: Vec<&str> = trimmed.split(['.', ':']).collect();
        if segments.iter().any(|seg| seg.trim().is_empty()) {
            return Err("topic segments cannot be empty".to_string());
        }
        Ok(Self(trimmed.to_string()))
    }

    // Access underlying raw topic string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    // Return individual segments split by dot or colon.
    pub fn segments(&self) -> Vec<&str> {
        self.0.split(['.', ':']).collect()
    }
}

impl Display for Topic {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Topic {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}
