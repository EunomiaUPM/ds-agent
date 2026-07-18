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

use common::serde_utils::serialize_opt_hash_hex;
use sea_orm::prelude::Json;
use serde::{Deserialize, Serialize};

// Message envelope

/// MessageEnvelope is a data structure that contains JSON-LD data
/// of the inbound or outbound messages, its URDNA2015 graph representation and a sha digest
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase", try_from = "MessageEnvelopeInput")]
pub(crate) struct MessageEnvelope {
    /// URDNA2015 canonical form (N-Quads text) for DSP JSON-LD messages; None for non-RDF
    /// protocols.
    pub canonical_form: Option<String>,
    #[serde(serialize_with = "serialize_opt_hash_hex")]
    pub canonical_hash: Option<[u8; 32]>,
    /// The protocol message stored as-is (JSON).
    pub payload: Json,
}

/// Deserialization input: `canonical_form` as string, `canonical_hash` as hex.
/// Matches the serialized form produced by the `serialize_with` helpers.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MessageEnvelopeInput {
    #[serde(default)]
    canonical_form: Option<String>,
    #[serde(default)]
    canonical_hash: Option<String>,
    #[serde(default)]
    payload: Json,
}

impl TryFrom<MessageEnvelopeInput> for MessageEnvelope {
    type Error = hex::FromHexError;

    fn try_from(v: MessageEnvelopeInput) -> Result<Self, Self::Error> {
        let canonical_hash = v
            .canonical_hash
            .map(|s| -> Result<[u8; 32], hex::FromHexError> {
                let mut hash = [0u8; 32];
                hex::decode_to_slice(s, &mut hash)?;
                Ok(hash)
            })
            .transpose()?;
        Ok(Self {
            canonical_form: v.canonical_form,
            canonical_hash,
            payload: v.payload,
        })
    }
}

#[allow(dead_code)]
impl MessageEnvelope {
    pub fn is_canonicalized(&self) -> bool {
        self.canonical_form.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Envelopes round-trip through serde_json (the DB storage path): canonical_form
    // as plain N-Quads text, canonical_hash as hex, payload verbatim.
    #[test]
    fn envelope_roundtrips_through_json() {
        let env = MessageEnvelope {
            canonical_form: Some("_:b0 <p> _:b1 .\n".to_string()),
            canonical_hash: Some([0xABu8; 32]),
            payload: serde_json::json!({"@type": "TransferRequestMessage"}),
        };
        let json = serde_json::to_string(&env).unwrap();
        let back: MessageEnvelope = serde_json::from_str(&json).unwrap();
        assert_eq!(back.canonical_form, env.canonical_form);
        assert_eq!(back.canonical_hash, env.canonical_hash);
        assert_eq!(back.payload, env.payload);
    }

    #[test]
    fn envelope_roundtrips_when_non_rdf() {
        let env = MessageEnvelope {
            canonical_form: None,
            canonical_hash: None,
            payload: serde_json::Value::Null,
        };
        let json = serde_json::to_string(&env).unwrap();
        let back: MessageEnvelope = serde_json::from_str(&json).unwrap();
        assert!(back.canonical_form.is_none());
        assert!(back.canonical_hash.is_none());
    }
}
