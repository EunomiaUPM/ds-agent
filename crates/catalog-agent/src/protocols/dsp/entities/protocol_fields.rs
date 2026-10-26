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

//! The catalog protocol fields, as extracted from a DSP message or a catalog RPC body.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A `CatalogRequestMessage` or `DatasetRequestMessage` (DSP 6.2.1, 6.2.2), or its RPC.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogProtocolFields {
    /// Implementation-specific filter expressions; `@container: @set` in the DSP context.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filter: Vec<Value>,
    /// The dataset a `DatasetRequestMessage` asks for.
    pub dataset: Option<String>,
    /// `eunomia:associatedAgentPeer`: the peer an RPC asks.
    pub associated_agent_peer: Option<String>,
    /// `eunomia:noCache`: the RPC asks the peer even when its catalog is cached.
    #[serde(default)]
    pub no_cache: bool,
}
