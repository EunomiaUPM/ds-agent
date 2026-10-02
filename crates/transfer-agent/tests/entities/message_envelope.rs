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

//! MessageEnvelope serialization, with and without RDF canonical form.

use transfer_agent::entities::message_envelope::*;

/// Envelopes round-trip through serde_json (the DB storage path): canonical_form
/// as plain N-Quads text, canonical_hash as hex, payload verbatim.
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

/// An envelope without canonical form or hash round-trips through JSON.
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
