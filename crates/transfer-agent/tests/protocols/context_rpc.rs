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

//! The RPC request context: the fields read from an outbound call.

use serde_json::json;
use transfer_agent::protocols::dsp::entities::context_rpc::*;

/// A TransferRequest-shaped RPC body: routing + agreement, no pids yet.
#[test]
fn serde_extraction_pulls_request_fields_and_tolerates_missing() {
    let body = json!({
        "agreementId": "urn:uuid:agr",
        "format": "HttpData",
        "providerAddress": "https://provider.example/dsp",
        "callbackAddress": "https://me.example/cb",
        "associatedAgentPeer": "urn:peer:provider",
        "somethingWeIgnore": true
    });
    let f: RpcMessageFields = serde_json::from_value(body).unwrap();
    assert_eq!(f.agreement_id.as_deref(), Some("urn:uuid:agr"));
    assert_eq!(
        f.provider_address.as_deref(),
        Some("https://provider.example/dsp")
    );
    assert!(f.consumer_pid.is_none()); // minted later, not in the request body
    assert!(f.data_address.is_none());
}
