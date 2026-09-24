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

//! Subscription pattern matching event topics, with single- and multi-level wildcards.

use std::fmt::{Display, Formatter};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::entities::topic::Topic;

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
    /// POSIX regex with the same semantics as `matches`, so topic filters run in the database.
    pub fn to_sql_regex(&self) -> String {
        let segments: Vec<&str> = self.0.split(['.', ':']).collect();
        if segments.iter().all(|s| *s == "**") {
            return ".*".to_string();
        }
        let mut regex = String::from("^");
        let mut needs_separator = false;
        for (i, segment) in segments.iter().enumerate() {
            match *segment {
                "**" if i == 0 => regex.push_str("([^.:]+[.:])*"),
                "**" => regex.push_str("([.:][^.:]+)*"),
                other => {
                    if needs_separator {
                        regex.push_str("[.:]");
                    }
                    if other == "*" {
                        regex.push_str("[^.:]+");
                    } else {
                        for c in other.chars() {
                            if !c.is_ascii_alphanumeric() && c != '_' && c != '-' {
                                regex.push('\\');
                            }
                            regex.push(c);
                        }
                    }
                }
            }
            needs_separator = *segment != "**" || i > 0;
        }
        regex.push('$');
        regex
    }

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
