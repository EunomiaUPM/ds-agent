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
use urn::Urn;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum TransferDirection {
    Push,
    Pull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum TransferRole {
    Provider,
    Consumer,
    Relay,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum ProtocolId {
    #[serde(rename = "dsp2024")]
    Dsp2024,
    #[serde(rename = "dsp2025_1")]
    Dsp2025_1,
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
