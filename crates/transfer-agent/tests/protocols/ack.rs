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

//! The DSP transfer ack: its wire shape (DSP 9.3.1).

use transfer_agent::protocols::dsp::entities::state::TransferDSPState;

use transfer_agent::protocols::dsp::entities::ack::*;

/// The wire shape is normative (DSP 9.3.1), so it is pinned here rather than
/// left to whatever the derives happen to produce.
#[test]
fn serializes_to_the_specified_shape() {
    let ack = TransferProcessAck::new(
        "urn:uuid:cc".to_string(),
        "urn:uuid:pp".to_string(),
        TransferDSPState::REQUESTED,
    );
    let json = serde_json::to_value(&ack).unwrap();
    assert_eq!(
        json,
        serde_json::json!({
            "@context": ["https://w3id.org/dspace/2025/1/context.jsonld"],
            "@type": "TransferProcess",
            "consumerPid": "urn:uuid:cc",
            "providerPid": "urn:uuid:pp",
            "state": "REQUESTED"
        })
    );
}
