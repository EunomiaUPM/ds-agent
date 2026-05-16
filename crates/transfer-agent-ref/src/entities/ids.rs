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

use compact_str::CompactString;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use urn::Urn;

// URN-based identifiers ─────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub(crate) struct TransferProcessId(pub(crate) Urn);

impl TransferProcessId {
    pub fn new(urn: Urn) -> Self {
        Self(urn)
    }

    pub fn generate() -> Self {
        Self(uuid_urn("transfer-process"))
    }

    pub fn as_urn(&self) -> &Urn {
        &self.0
    }
}

impl fmt::Display for TransferProcessId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub(crate) struct MessageId(pub(crate) Urn);

#[allow(dead_code)]
impl MessageId {
    pub fn new(urn: Urn) -> Self {
        Self(urn)
    }

    pub fn generate() -> Self {
        Self(uuid_urn("transfer-message"))
    }

    pub fn as_urn(&self) -> &Urn {
        &self.0
    }
}

impl fmt::Display for MessageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub(crate) struct ParticipantId(pub(crate) Urn);

#[allow(dead_code)]
impl ParticipantId {
    pub fn new(urn: Urn) -> Self {
        Self(urn)
    }
    pub fn as_urn(&self) -> &Urn {
        &self.0
    }
}

impl fmt::Display for ParticipantId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// String-based identifiers ──────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub(crate) struct TenantId(pub(crate) String);

impl TenantId {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TenantId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// X-Correlation-ID: ties a chain of related requests/responses together.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub(crate) struct CorrelationId(pub(crate) CompactString);

impl CorrelationId {
    pub fn new(s: impl Into<CompactString>) -> Self {
        Self(s.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CorrelationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// X-Request-ID: unique identifier for a single inbound/outbound request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub(crate) struct RequestId(pub(crate) CompactString);

impl RequestId {
    pub fn new(s: impl Into<CompactString>) -> Self {
        Self(s.into())
    }

    pub fn generate() -> Self {
        Self(CompactString::from(uuid::Uuid::new_v4().to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RequestId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Helpers ───────────────────────────────────────────────────────────────────

fn uuid_urn(prefix: &str) -> Urn {
    Urn::from_str(&format!("urn:{}:{}", prefix, uuid::Uuid::new_v4()))
        .expect("UUID URN is always valid")
}
