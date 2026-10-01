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

//! The RDF engine, split by behaviour: typed extraction, expansion and canonicalization, and
//! navigation of an expanded document. Shares a sample DSP transfer request.

mod expanded_doc;
mod expansion;
mod extraction;

use serde_json::json;

fn sample_transfer_request() -> serde_json::Value {
    json!({
        "@context": ["https://w3id.org/dspace/2025/1/context.jsonld"],
        "@type": "TransferRequestMessage",
        "agreementId": "urn:uuid:e8dc8655-44c2-46ef-b701-4cffdc2faa44",
        "callbackAddress": "https://example.com/callback",
        "consumerPid": "urn:uuid:32541fe6-c580-409e-85a8-8a9a32fbe833",
        "dataAddress": {
            "@type": "DataAddress",
            "endpoint": "http://example.com",
            "endpointProperties": [
                {"@type": "EndpointProperty", "name": "authorization", "value": "TOKEN-ABCDEFG"},
                {"@type": "EndpointProperty", "name": "authType", "value": "bearer"}
            ],
            "endpointType": "https://w3id.org/idsa/v4.1/HTTP"
        },
        "format": "example:HTTP_PUSH"
    })
}
