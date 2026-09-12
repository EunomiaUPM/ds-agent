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

// Pattern supporting exact match, single-segment (*), and multi-segment (**) wildcards.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TopicPattern(String);

impl TopicPattern {
    // Validate and create a new topic pattern.
    pub fn new(pattern: impl Into<String>) -> Result<Self, String> {
        let s = pattern.into();
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err("pattern cannot be empty".to_string());
        }
        Ok(Self(trimmed.to_string()))
    }

    // Return global wildcard matching any event topic.
    pub fn match_all() -> Self {
        Self("**".to_string())
    }

    // Access underlying pattern string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    // Check whether this pattern contains wildcard tokens.
    pub fn has_wildcard(&self) -> bool {
        self.0.contains('*')
    }

    // Evaluate whether the topic satisfies this subscription pattern.
    pub fn matches(&self, topic: &Topic) -> bool {
        if self.0 == "*" || self.0 == "**" || self.0 == "*.*.*" {
            return true;
        }

        let pat_segments: Vec<&str> = self.0.split(['.', ':']).collect();
        let topic_segments = topic.segments();

        Self::match_segments(&pat_segments, &topic_segments)
    }

    // Recursive segment matcher handling single- and multi-level wildcards.
    fn match_segments(pat: &[&str], topic: &[&str]) -> bool {
        if pat.is_empty() {
            return topic.is_empty();
        }

        if pat[0] == "**" {
            if pat.len() == 1 {
                return true;
            }
            for i in 0..=topic.len() {
                if Self::match_segments(&pat[1..], &topic[i..]) {
                    return true;
                }
            }
            return false;
        }

        if topic.is_empty() {
            return false;
        }

        if pat[0] == "*" || pat[0] == topic[0] {
            return Self::match_segments(&pat[1..], &topic[1..]);
        }

        false
    }
}

impl Display for TopicPattern {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for TopicPattern {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}
