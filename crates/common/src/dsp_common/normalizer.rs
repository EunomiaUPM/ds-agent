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

//! Middleware that normalises JSON-LD namespace prefixes in incoming DSP
//! messages.
//!
//! The Eclipse DSP TCK (and other DSP implementations) send compact JSON-LD
//! where every field name and `@type` value carries an explicit namespace
//! prefix, e.g.
//!
//! ```json
//! {
//!   "@context":  "https://w3id.org/dspace/2024/1/context.json",
//!   "@type":     "dspace:ContractRequestMessage",
//!   "dspace:consumerPid":    "urn:uuid:…",
//!   "dspace:offer": {
//!     "@type":        "odrl:Offer",
//!     "odrl:target":  "urn:uuid:…",
//!     "odrl:action":  "use"
//!   },
//!   "dspace:callbackAddress": "https://…"
//! }
//! ```
//!
//! Our serde structs expect the short camelCase names (`consumerPid`, `offer`,
//! `target`, …) so every message fails to deserialise when it arrives with
//! prefixes.  This middleware:
//!
//! 1. Reads the raw request body (only when `Content-Type` is JSON/JSON-LD).
//! 2. Parses it as `serde_json::Value`.
//! 3. Recursively strips `dspace:`, `odrl:`, `dct:` and `xsd:` prefixes from object keys (leaving
//!    JSON-LD keywords such as `@context`, `@type`, `@id` intact).
//! 4. Strips the same prefixes from the *values* of any `@type` key.
//! 5. Puts the normalised JSON back as the request body for downstream handlers.

use axum::{body::Body, extract::Request, middleware::Next, response::Response};

const DSP_PREFIXES: &[&str] = &["dspace:", "odrl:", "dct:", "xsd:"];

pub async fn dsp_namespace_normalizer(request: Request, next: Next) -> Response {
    let (parts, body) = request.into_parts();

    let is_json = parts
        .headers
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|ct| ct.contains("application/json") || ct.contains("application/ld+json"))
        .unwrap_or(false);

    if !is_json {
        return next.run(Request::from_parts(parts, body)).await;
    }

    let bytes = match axum::body::to_bytes(body, 4 * 1024 * 1024).await {
        Ok(b) => b,
        Err(_) => {
            return next.run(Request::from_parts(parts, Body::empty())).await;
        }
    };

    let new_body = match serde_json::from_slice::<serde_json::Value>(&bytes) {
        Ok(value) => {
            let normalised = normalise_value(value);
            match serde_json::to_vec(&normalised) {
                Ok(v) => Body::from(v),
                Err(_) => Body::from(bytes),
            }
        }
        Err(_) => Body::from(bytes),
    };

    next.run(Request::from_parts(parts, new_body)).await
}

fn normalise_value(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let new_map = map
                .into_iter()
                .map(|(k, v)| {
                    let new_val = if k == "@type" {
                        normalise_type_value(v)
                    } else {
                        normalise_value(v)
                    };
                    (normalise_key(k), new_val)
                })
                .collect();
            serde_json::Value::Object(new_map)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(normalise_value).collect())
        }
        other => other,
    }
}

/// Strip a known DSP/ODRL prefix from an object key.  Keys that start with
/// `@` (JSON-LD keywords) are left unchanged.
fn normalise_key(key: String) -> String {
    if key.starts_with('@') {
        return key;
    }
    for prefix in DSP_PREFIXES {
        if let Some(rest) = key.strip_prefix(prefix) {
            return rest.to_string();
        }
    }
    key
}

/// Strip a known DSP/ODRL prefix from a `@type` value (string or array of
/// strings).
fn normalise_type_value(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::String(s) => {
            for prefix in DSP_PREFIXES {
                if let Some(rest) = s.strip_prefix(prefix) {
                    return serde_json::Value::String(rest.to_string());
                }
            }
            serde_json::Value::String(s)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(normalise_type_value).collect())
        }
        other => other,
    }
}
