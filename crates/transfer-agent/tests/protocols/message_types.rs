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

//! Reading the DSP message type of a payload.

use serde_json::json;
use transfer_agent::protocols::dsp::entities::message_types::*;

/// The type is read from the top-level `@type`.
#[test]
fn reads_the_top_level_type() {
    let t = TransferDSPMessageType::from_json_payload(&json!({"@type": "TransferRequestMessage"}));
    assert_eq!(t.unwrap(), TransferDSPMessageType::TransferRequestMessage);
}

/// A `dspace:`-prefixed type is accepted.
#[test]
fn accepts_the_prefixed_form() {
    let t =
        TransferDSPMessageType::from_json_payload(&json!({"@type": "dspace:TransferStartMessage"}));
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

/// A payload without type is an error.
#[test]
fn a_payload_without_a_type_is_an_error() {
    assert!(
        TransferDSPMessageType::from_json_payload(&json!({"consumerPid": "urn:uuid:cc"})).is_err()
    );
}
