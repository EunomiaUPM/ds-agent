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

use crate::entities::ids::ParticipantId;
use compact_str::CompactString;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use urn::Urn;
// Common Transfer process related protocol fields
// Such as direction, role, protocolId, loose protocolState, loose protocolMessageType
// And Protocol correlation which is a identifiers DSP-loosely-related correlation for convenience

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransferDirection {
    Push,
    Pull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransferRole {
    #[serde(alias = "Provider")]
    Provider,
    #[serde(alias = "Consumer")]
    Consumer,
    #[serde(alias = "Relay")]
    Relay,
}

impl Display for TransferRole {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TransferRole::Provider => f.write_str("provider"),
            TransferRole::Consumer => f.write_str("consumer"),
            TransferRole::Relay => f.write_str("relay"),
        }
    }
}

impl FromStr for TransferRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "provider" => Ok(TransferRole::Provider),
            "consumer" => Ok(TransferRole::Consumer),
            "relay" => Ok(TransferRole::Relay),
            other => Err(format!("unknown role: {other}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum ProtocolId {
    #[serde(rename = "dsp2024")]
    Dsp2024,
    #[serde(rename = "dsp2025_1")]
    Dsp2025_1,
}

impl Display for ProtocolId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolId::Dsp2024 => f.write_str("dsp2024"),
            ProtocolId::Dsp2025_1 => f.write_str("dsp2025_1"),
        }
    }
}

impl FromStr for ProtocolId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dsp2024" => Ok(ProtocolId::Dsp2024),
            "dsp2025_1" => Ok(ProtocolId::Dsp2025_1),
            other => Err(format!("unknown protocol: {other}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub(crate) struct ProtocolState(pub CompactString);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub(crate) struct ProtocolMessageType(pub CompactString);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct StateMetadata {
    pub attribute: Option<String>,
    pub reason: Option<Vec<String>>,
    pub code: Option<String>,
}

impl StateMetadata {
    pub fn empty() -> Self {
        Self {
            attribute: None,
            reason: None,
            code: None,
        }
    }
}

pub(crate) const CONSUMER_PID_KEY: &str = "consumerPid";
pub(crate) const PROVIDER_PID_KEY: &str = "providerPid";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct TransferCorrelation {
    pub identifiers: HashMap<String, String>,
    pub consumer_pid: Option<String>,
    pub provider_pid: Option<String>,
    pub agreement_id: Option<Urn>,
    pub callback_address: Option<url::Url>,
    pub peer_participant_id: Option<ParticipantId>,
}

impl TransferCorrelation {
    pub fn empty() -> Self {
        Self {
            identifiers: HashMap::new(),
            consumer_pid: None,
            provider_pid: None,
            agreement_id: None,
            callback_address: None,
            peer_participant_id: None,
        }
    }
}
