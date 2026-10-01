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

use std::fmt::{Display, Formatter};

use crate::dsp_common::well_known_types::DSPProtocolVersions;
use serde::{Deserialize, Serialize};
use ymir::errors::{BadFormat, Errors, Outcome};

pub static CONTEXT: &str = "https://w3id.org/dspace/2025/1/context.jsonld";

/// All context URLs that this implementation will accept on incoming messages.
const ACCEPTED_CONTEXTS: &[&str] = &[
    "https://w3id.org/dspace/2024/1/context.json",
    "https://w3id.org/dspace/2025/1/context.jsonld",
];

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ContextField {
    Single(String),
    Multiple(Vec<String>),
}

impl ContextField {
    pub fn validate(&self) -> Outcome<()> {
        let is_valid = match self {
            ContextField::Single(s) => ACCEPTED_CONTEXTS.contains(&s.as_str()),
            ContextField::Multiple(v) => v.iter().any(|s| ACCEPTED_CONTEXTS.contains(&s.as_str())),
        };
        if is_valid {
            Ok(())
        } else {
            Err(Errors::format(
                BadFormat::Received,
                "Invalid @context value",
                None,
            ))
        }
    }
}

impl Display for ContextField {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(CONTEXT)
    }
}

/// The protocol version a `@context` URL implies (DSP 4.3).
///
/// The endpoint version and the vocabulary version are formally different things
/// — the first comes from the mount path, the second from `@context` — but a
/// conformant peer uses the context of the version it is speaking, so this is a
/// sound derivation and a great deal better than a constant.
pub fn version_from_context_url(url: &str) -> Option<DSPProtocolVersions> {
    match url {
        "https://w3id.org/dspace/2024/1/context.json" => Some(DSPProtocolVersions::V2024_1),
        "https://w3id.org/dspace/2025/1/context.jsonld" => Some(DSPProtocolVersions::V2025_1),
        _ => None,
    }
}

/// The version implied by a message's `@context`, whatever shape it takes: a bare
/// string, or an array where the DSP context sits among profile contexts.
pub fn version_from_payload(payload: &serde_json::Value) -> Option<DSPProtocolVersions> {
    match payload.get("@context")? {
        serde_json::Value::String(s) => version_from_context_url(s),
        serde_json::Value::Array(items) => items
            .iter()
            .filter_map(serde_json::Value::as_str)
            .find_map(version_from_context_url),
        _ => None,
    }
}

impl Default for ContextField {
    fn default() -> Self {
        ContextField::Multiple(vec![CONTEXT.to_string()])
    }
}
