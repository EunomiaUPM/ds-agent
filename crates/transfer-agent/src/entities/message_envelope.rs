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
use sha2::{Digest, Sha256};

// Message envelope

/// MessageEnvelope is a data structure that contains JSON-LD data
/// of the inbound or outbound messages, its URDNA2015 graph representation and a sha digest
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase", try_from = "MessageEnvelopeInput")]
pub struct MessageEnvelope {
    /// URDNA2015 canonical form (N-Quads text) for DSP JSON-LD messages; None for non-RDF
    /// protocols.
    pub canonical_form: Option<String>,
    #[serde(serialize_with = "serialize_opt_hash_hex")]
    pub canonical_hash: Option<[u8; 32]>,
    /// The protocol message stored as-is (JSON).
    pub payload: Json,
}

impl MessageEnvelope {
    /// The canonical pair is present only for RDF protocols; plain-JSON ones
    /// have no canonical form to record.
    pub fn new(payload: Json, canonical: Option<(String, [u8; 32])>) -> Self {
        let (canonical_form, canonical_hash) = match canonical {
            Some((form, hash)) => (Some(form), Some(hash)),
            None => (None, None),
        };
        Self {
            canonical_form,
            canonical_hash,
            payload,
        }
    }

    /// Builds an envelope hashing the canonical form (SHA-256) when one is given.
    pub fn from_canonical(payload: Json, canonical_form: Option<String>) -> Self {
        let canonical = canonical_form.map(|form| {
            let hash: [u8; 32] = Sha256::digest(form.as_bytes()).into();
            (form, hash)
        });
        Self::new(payload, canonical)
    }
}

/// Deserialization input: `canonical_form` as string, `canonical_hash` as hex.
/// Matches the serialized form produced by the `serialize_with` helpers.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageEnvelopeInput {
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
