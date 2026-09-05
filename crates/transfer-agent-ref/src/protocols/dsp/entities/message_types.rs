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

//! The DSP transfer message types, and reading one off a message.

use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;
use ymir::errors::{BadFormat, Errors, Outcome};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum TransferDSPMessageType {
    TransferRequestMessage,
    TransferStartMessage,
    TransferCompletionMessage,
    TransferSuspensionMessage,
    TransferTerminationMessage,
    TransferProcess,
    TransferError,
}

impl Display for TransferDSPMessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            TransferDSPMessageType::TransferRequestMessage => "TransferRequestMessage".to_string(),
            TransferDSPMessageType::TransferStartMessage => "TransferStartMessage".to_string(),
            TransferDSPMessageType::TransferCompletionMessage => {
                "TransferCompletionMessage".to_string()
            }
            TransferDSPMessageType::TransferSuspensionMessage => {
                "TransferSuspensionMessage".to_string()
            }
            TransferDSPMessageType::TransferTerminationMessage => {
                "TransferTerminationMessage".to_string()
            }
            TransferDSPMessageType::TransferProcess => "TransferProcess".to_string(),
            TransferDSPMessageType::TransferError => "TransferError".to_string(),
        };
        write!(f, "{}", str)
    }
}

impl std::str::FromStr for TransferDSPMessageType {
    type Err = Errors;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Accept the prefixed form too: a peer may write `dspace:TransferStartMessage`
        // and the namespace normalizer only strips prefixes it is wired for.
        let term = s.rsplit([':', '/', '#']).next().unwrap_or(s);
        match term {
            "TransferRequestMessage" => Ok(Self::TransferRequestMessage),
            "TransferStartMessage" => Ok(Self::TransferStartMessage),
            "TransferCompletionMessage" => Ok(Self::TransferCompletionMessage),
            "TransferSuspensionMessage" => Ok(Self::TransferSuspensionMessage),
            "TransferTerminationMessage" => Ok(Self::TransferTerminationMessage),
            "TransferProcess" => Ok(Self::TransferProcess),
            "TransferError" => Ok(Self::TransferError),
            other => Err(Errors::format(
                BadFormat::Received,
                format!("unknown DSP message type: {other}"),
                None,
            )),
        }
    }
}

impl TransferDSPMessageType {
    pub fn from_json_payload(payload: &serde_json::Value) -> Outcome<Self> {
        if let Some(t) = payload.get("@type").and_then(Self::type_term) {
            return Self::from_str(&t);
        }
        if let Some(nodes) = payload.get("@graph").and_then(serde_json::Value::as_array) {
            if let Some(found) = nodes
                .iter()
                .filter_map(|n| n.get("@type").and_then(Self::type_term))
                .find_map(|t| Self::from_str(&t).ok())
            {
                return Ok(found);
            }
        }
        Err(Errors::format(
            BadFormat::Received,
            "message declares no @type",
            None,
        ))
    }

    /// `@type` is a string or an array of them; take the first usable one.
    fn type_term(value: &serde_json::Value) -> Option<String> {
        match value {
            serde_json::Value::String(s) => Some(s.clone()),
            serde_json::Value::Array(items) => items
                .iter()
                .find_map(serde_json::Value::as_str)
                .map(str::to_string),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn reads_the_top_level_type() {
        let t =
            TransferDSPMessageType::from_json_payload(&json!({"@type": "TransferRequestMessage"}));
        assert_eq!(t.unwrap(), TransferDSPMessageType::TransferRequestMessage);
    }

    #[test]
    fn accepts_the_prefixed_form() {
        let t = TransferDSPMessageType::from_json_payload(
            &json!({"@type": "dspace:TransferStartMessage"}),
        );
        assert_eq!(t.unwrap(), TransferDSPMessageType::TransferStartMessage);
    }

    /// The `@graph` form carries no top-level `@type`; the message node is inside.
    #[test]
    fn finds_the_type_inside_a_graph() {
        let payload = json!({
            "@context": "https://w3id.org/dspace/2025/1/context.jsonld",
            "@graph": [
                {"@id": "_:addr", "@type": "DataAddress"},
                {"@id": "_:msg", "@type": "TransferRequestMessage"}
            ]
        });
        let t = TransferDSPMessageType::from_json_payload(&payload);
        assert_eq!(t.unwrap(), TransferDSPMessageType::TransferRequestMessage);
    }

    #[test]
    fn a_payload_without_a_type_is_an_error() {
        assert!(
            TransferDSPMessageType::from_json_payload(&json!({"consumerPid": "urn:uuid:cc"}))
                .is_err()
        );
    }
}
