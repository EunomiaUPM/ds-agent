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

//! Data address of a transfer.

use serde::{Deserialize, Serialize};

/// Where and how to reach the data of a transfer (DSP Appendix A).
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Eq)]
pub struct DataAddress {
    #[serde(rename = "@type")]
    pub _type: String,
    #[serde(rename = "endpointType")] // TODO define this
    pub endpoint_type: String,
    /// Optional (DSP Appendix A): a pull request may carry none and get it in the start message.
    /// Whether a message needs one depends on the connector behind `format`, not on its shape.
    #[serde(rename = "endpoint", skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    #[serde(rename = "endpointProperties")]
    pub endpoint_properties: Vec<EndpointProperty>,
}

/// Name and value pair inside a data address, such as an auth token.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Eq)]
pub struct EndpointProperty {
    #[serde(rename = "@type")]
    pub _type: String,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "value")]
    pub value: String,
}
