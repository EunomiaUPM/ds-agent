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

//! The Transfer Process Protocol fields (DSP 9.2), as extracted. This is what
//! crosses into the domain — no bytes, no graph, no HTTP.

use common::dsp_common::data_address::DataAddress;
use serde::{Deserialize, Serialize};

/// Every member is optional because extraction is total: which ones are
/// required depends on the message type, and that is the validator's call.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferProtocolFields {
    pub consumer_pid: Option<String>,
    pub provider_pid: Option<String>,
    /// The agreement the transfer runs under; the domain resolves it from here.
    pub agreement_id: Option<String>,
    pub callback_address: Option<String>,
    /// `dct:format`, the identifier of a distribution of the agreement's target.
    pub format: Option<String>,
    pub data_address: Option<DataAddress>,
    /// Only in scope for suspension and termination (DSP 9.2.3, 9.2.5).
    pub code: Option<String>,
    /// `@container: @set` in the context, so genuinely multi-valued.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason: Vec<String>,
}
