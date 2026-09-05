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

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AuthProtocolTypes {
    #[serde(rename = "OAuth")]
    OAuth,
    #[serde(rename = "Token")]
    OpaqueToken,
    #[serde(rename = "GNAP")]
    Gnap,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum DSPProtocolVersions {
    #[serde(rename = "2024-1")]
    V2024_1,
    #[serde(rename = "2025-1")]
    V2025_1,
}

impl std::fmt::Display for DSPProtocolVersions {
    /// The version tag as it appears on the wire (DSP 4.3), not the Rust variant
    /// name — this string is protocol-stable, so it is safe to build identifiers
    /// out of.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::V2024_1 => "2024-1",
            Self::V2025_1 => "2025-1",
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DSPIdentifierTypes {
    #[serde(rename = "did:web")]
    DidWeb,
    #[serde(rename = "did:jwk")]
    DidJWK,
    #[serde(rename = "url")]
    Url,
    #[serde(rename = "D-U-N-S")]
    DUNS,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DSPBindings {
    #[serde(rename = "HTTPS")]
    HTTPS,
    #[serde(rename = "HTTP")]
    HTTP,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VersionResponse {
    pub protocol_versions: Vec<Version>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    pub binding: DSPBindings,
    pub path: String,
    pub version: DSPProtocolVersions,
    pub auth: Option<Auth>,
    pub identifier_type: Option<DSPIdentifierTypes>,
    pub service_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Auth {
    pub protocol: AuthProtocolTypes,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VersionPath {
    pub path: String,
}
