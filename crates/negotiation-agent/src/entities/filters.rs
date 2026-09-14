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

//! Domain filters for negotiation processes, messages, agreements, and offers.

use chrono::{DateTime, Utc};
use common::query::{QueryFilter, validate_date_range};
use serde::{Deserialize, Serialize};
use ymir::errors::Outcome;

/// Filter criteria for negotiation processes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct NegotiationProcessFilter {
    pub state: Option<String>,
    pub role: Option<String>,
    pub protocol: Option<String>,
    #[serde(alias = "associated_agent_peer")]
    pub associated_agent_peer: Option<String>,
    #[serde(alias = "created_after")]
    pub created_after: Option<DateTime<Utc>>,
    #[serde(alias = "created_before")]
    pub created_before: Option<DateTime<Utc>>,
}

impl QueryFilter for NegotiationProcessFilter {
    fn is_empty(&self) -> bool {
        self.state.is_none()
            && self.role.is_none()
            && self.protocol.is_none()
            && self.associated_agent_peer.is_none()
            && self.created_after.is_none()
            && self.created_before.is_none()
    }

    fn validate(&self) -> Outcome<()> {
        validate_date_range(self.created_after, self.created_before)
    }
}

/// Filter criteria for negotiation messages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct NegotiationMessageFilter {
    #[serde(alias = "process_id")]
    pub process_id: Option<String>,
    pub protocol: Option<String>,
    #[serde(alias = "message_type")]
    pub message_type: Option<String>,
    pub direction: Option<String>,
    #[serde(alias = "created_after")]
    pub created_after: Option<DateTime<Utc>>,
    #[serde(alias = "created_before")]
    pub created_before: Option<DateTime<Utc>>,
}

impl QueryFilter for NegotiationMessageFilter {
    fn is_empty(&self) -> bool {
        self.process_id.is_none()
            && self.protocol.is_none()
            && self.message_type.is_none()
            && self.direction.is_none()
            && self.created_after.is_none()
            && self.created_before.is_none()
    }

    fn validate(&self) -> Outcome<()> {
        validate_date_range(self.created_after, self.created_before)
    }
}

/// Filter criteria for agreements.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AgreementFilter {
    #[serde(alias = "process_id")]
    pub process_id: Option<String>,
    #[serde(
        alias = "consumer_id",
        alias = "consumerParticipantId",
        alias = "consumer_participant_id"
    )]
    pub consumer_id: Option<String>,
    #[serde(
        alias = "provider_id",
        alias = "providerParticipantId",
        alias = "provider_participant_id"
    )]
    pub provider_id: Option<String>,
    pub target: Option<String>,
    pub state: Option<String>,
    #[serde(alias = "created_after")]
    pub created_after: Option<DateTime<Utc>>,
    #[serde(alias = "created_before")]
    pub created_before: Option<DateTime<Utc>>,
}

impl QueryFilter for AgreementFilter {
    fn is_empty(&self) -> bool {
        self.process_id.is_none()
            && self.consumer_id.is_none()
            && self.provider_id.is_none()
            && self.target.is_none()
            && self.state.is_none()
            && self.created_after.is_none()
            && self.created_before.is_none()
    }

    fn validate(&self) -> Outcome<()> {
        validate_date_range(self.created_after, self.created_before)
    }
}

/// Filter criteria for offers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct OfferFilter {
    #[serde(alias = "process_id")]
    pub process_id: Option<String>,
    #[serde(alias = "offer_id")]
    pub offer_id: Option<String>,
    pub target: Option<String>,
    #[serde(alias = "created_after")]
    pub created_after: Option<DateTime<Utc>>,
    #[serde(alias = "created_before")]
    pub created_before: Option<DateTime<Utc>>,
}

impl QueryFilter for OfferFilter {
    fn is_empty(&self) -> bool {
        self.process_id.is_none()
            && self.offer_id.is_none()
            && self.target.is_none()
            && self.created_after.is_none()
            && self.created_before.is_none()
    }

    fn validate(&self) -> Outcome<()> {
        validate_date_range(self.created_after, self.created_before)
    }
}
