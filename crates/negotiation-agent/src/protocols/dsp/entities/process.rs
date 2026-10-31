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

//! A stored negotiation as the DSP flows read it: its row with the pids, role and state parsed.

use std::collections::HashMap;
use std::str::FromStr;

use urn::Urn;
use ymir::errors::{BadFormat, Errors, Outcome};
use ymir::oauth::{OwnedTrait, Owner};

use crate::data::sea_orm::orm::negotiation_process::Model;
use crate::entities::negotiation_process::NegotiationCorrelation;
use crate::protocols::dsp::entities::role::NegotiationRole;
use crate::protocols::dsp::entities::state::NegotiationState;

/// A negotiation process hydrated for the protocol.
#[derive(Debug, Clone)]
pub struct NegotiationProcess {
    pub model: Model,
    pub consumer_pid: Option<String>,
    pub provider_pid: Option<String>,
    pub role: NegotiationRole,
    pub state: NegotiationState,
}

impl NegotiationProcess {
    /// Identifier keys the pids are stored under.
    pub const CONSUMER_PID: &'static str = NegotiationCorrelation::CONSUMER_PID;
    pub const PROVIDER_PID: &'static str = NegotiationCorrelation::PROVIDER_PID;

    /// The row with its identifiers; a role or state outside DSP is a broken record.
    pub fn hydrate(
        model: Model,
        identifiers: impl IntoIterator<Item = (String, Option<String>)>,
    ) -> Outcome<Self> {
        let (mut consumer_pid, mut provider_pid) = (None, None);
        for (key, value) in identifiers {
            match key.as_str() {
                Self::CONSUMER_PID => consumer_pid = value,
                Self::PROVIDER_PID => provider_pid = value,
                _ => {}
            }
        }
        Ok(Self {
            role: NegotiationRole::from_str(&model.role)?,
            state: NegotiationState::from_str(&model.state)?,
            model,
            consumer_pid,
            provider_pid,
        })
    }

    pub fn id(&self) -> Outcome<Urn> {
        Urn::from_str(&self.model.id).map_err(|e| {
            Errors::format(
                BadFormat::Received,
                format!("stored negotiation id is not a URN: {e}"),
                None,
            )
        })
    }

    pub fn owner(&self) -> Owner {
        self.model.owner()
    }

    /// The participant on the other side.
    pub fn peer_id(&self) -> &str {
        &self.model.associated_agent_peer
    }

    /// The peer's DSP base, where our messages go.
    pub fn callback_address(&self) -> Option<&str> {
        self.model.callback_address.as_deref()
    }

    /// The pid this connector minted.
    pub fn own_pid(&self) -> Option<&str> {
        match self.role {
            NegotiationRole::Provider => self.provider_pid.as_deref(),
            NegotiationRole::Consumer => self.consumer_pid.as_deref(),
        }
    }

    /// The pid the peer minted, the one our messages address.
    pub fn peer_pid(&self) -> Option<&str> {
        match self.role {
            NegotiationRole::Provider => self.consumer_pid.as_deref(),
            NegotiationRole::Consumer => self.provider_pid.as_deref(),
        }
    }

    /// The pids as stored identifiers.
    pub fn identifiers(&self) -> HashMap<String, String> {
        [
            (Self::CONSUMER_PID, &self.consumer_pid),
            (Self::PROVIDER_PID, &self.provider_pid),
        ]
        .into_iter()
        .filter_map(|(key, pid)| pid.clone().map(|pid| (key.to_string(), pid)))
        .collect()
    }

    /// Microseconds of the last write: the table has no counter, and every write stamps it.
    pub fn version(&self) -> u64 {
        let at = self.model.updated_at.unwrap_or(self.model.created_at);
        u64::try_from(at.timestamp_micros()).unwrap_or_default()
    }
}
