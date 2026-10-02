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

//! The `TransferProcess` ACK a peer returns when a state change succeeded
//! (DSP 9.3.1). All five members are REQUIRED, so none of them is an `Option`.

use common::dsp_common::context_field::ContextField;
use serde::{Deserialize, Serialize};

use crate::protocols::dsp::entities::message_types::TransferDSPMessageType;
use crate::protocols::dsp::entities::state::TransferDSPState;

/// `TransferProcess` acknowledgement returned to the peer, with both pids and the state.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferProcessAck {
    #[serde(rename = "@context")]
    pub context: ContextField,
    #[serde(rename = "@type")]
    pub _type: TransferDSPMessageType,
    pub consumer_pid: String,
    pub provider_pid: String,
    pub state: TransferDSPState,
}

impl TransferProcessAck {
    pub fn new(consumer_pid: String, provider_pid: String, state: TransferDSPState) -> Self {
        Self {
            context: ContextField::default(),
            _type: TransferDSPMessageType::TransferProcess,
            consumer_pid,
            provider_pid,
            state,
        }
    }
}
