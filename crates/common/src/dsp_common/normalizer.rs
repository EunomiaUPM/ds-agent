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
//! 3. Recursively strips `dspace:`, `odrl:` and `dct:` prefixes from object keys (leaving
//!    JSON-LD keywords such as `@context`, `@type`, `@id` intact).
//! 4. Strips the same prefixes from the *values* of any `@type` key.
//! 5. Puts the normalised JSON back as the request body for downstream handlers.

use axum::{body::Body, body::Bytes, extract::Request, middleware::Next, response::Response};

/// Cap on the inbound body this middleware will buffer.
const MAX_BODY_BYTES: usize = 4 * 1024 * 1024;

/// The request body **exactly as it arrived**, stashed in the request extensions
/// before [`dsp_namespace_normalizer`] rewrites it.
///
/// Everything downstream sees the normalised body, which is the right input for
/// deserialization but the wrong one for anything that must speak about what the
/// peer actually sent: signature verification and audit. Those read this.
///
/// It is captured here, rather than by a separate layer, so it cannot be ordered
/// after the rewrite that would make it a lie.
#[derive(Clone, Debug)]
pub struct WireBody(pub Bytes);

/// Prefixes stripped from object keys and `@type` values.
///
/// `xsd:` is deliberately **not** here: it is a datatype vocabulary, never a DSP
/// field name, so `"@type": "xsd:dateTime"` on a literal is correct compact
/// JSON-LD. Stripping it leaves a relative IRI that expansion resolves against
/// the document base (`x-string:///dateTime`), silently corrupting the datatype
/// and therefore the canonical hash.
const DSP_PREFIXES: &[&str] = &["dspace:", "odrl:", "dct:"];

pub async fn dsp_namespace_normalizer(request: Request, next: Next) -> Response {
    let (mut parts, body) = request.into_parts();

    let is_json = parts
        .headers
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|ct| ct.contains("application/json") || ct.contains("application/ld+json"))
        .unwrap_or(false);

    if !is_json {
        return next.run(Request::from_parts(parts, body)).await;
    }

    let bytes = match axum::body::to_bytes(body, MAX_BODY_BYTES).await {
        Ok(b) => b,
        Err(_) => {
            return next.run(Request::from_parts(parts, Body::empty())).await;
        }
    };

    // Before any rewriting: what the peer actually put on the wire.
    parts.extensions.insert(WireBody(bytes.clone()));

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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{routing::post, Router};
    use serde_json::json;
    use tower::ServiceExt;

    /// A literal's `xsd:` datatype must survive normalisation: stripping it
    /// would leave a relative IRI that expands to `x-string:///dateTime`.
    #[test]
    fn keeps_xsd_datatypes_on_literals() {
        let out = normalise_value(json!({
            "odrl:constraint": [{
                "odrl:leftOperand": "odrl:dateTime",
                "odrl:rightOperand": {
                    "@type": "xsd:dateTime",
                    "@value": "2026-12-31T23:59:59Z"
                }
            }]
        }));
        assert_eq!(
            out["constraint"][0]["rightOperand"]["@type"],
            json!("xsd:dateTime")
        );
        // The DSP/ODRL prefixes are still stripped from keys as before.
        assert_eq!(out["constraint"][0]["leftOperand"], json!("odrl:dateTime"));
    }

    #[test]
    fn still_strips_dsp_prefixes_from_keys_and_types() {
        let out = normalise_value(json!({
            "@type": "dspace:TransferRequestMessage",
            "dspace:consumerPid": "urn:uuid:cc",
            "dct:format": "HttpData-PULL"
        }));
        assert_eq!(out["@type"], json!("TransferRequestMessage"));
        assert_eq!(out["consumerPid"], json!("urn:uuid:cc"));
        assert_eq!(out["format"], json!("HttpData-PULL"));
    }
    /// The normalizer rewrites the body downstream handlers read, so it must also
    /// preserve the bytes the peer actually sent — otherwise nothing downstream
    /// can verify a signature or audit what arrived.
    #[tokio::test]
    async fn stashes_the_pre_rewrite_body() {
        const SENT: &str =
            r#"{"@type":"dspace:TransferRequestMessage","dspace:consumerPid":"urn:uuid:cc"}"#;

        async fn handler(request: axum::extract::Request) -> String {
            let wire = request
                .extensions()
                .get::<WireBody>()
                .expect("normalizer must stash the wire body")
                .0
                .clone();
            let seen = axum::body::to_bytes(request.into_body(), MAX_BODY_BYTES)
                .await
                .unwrap();
            format!(
                "{}|{}",
                String::from_utf8(wire.to_vec()).unwrap(),
                String::from_utf8(seen.to_vec()).unwrap()
            )
        }

        let app = Router::new()
            .route("/request", post(handler))
            .layer(axum::middleware::from_fn(dsp_namespace_normalizer));

        let response = app
            .oneshot(
                axum::extract::Request::builder()
                    .method("POST")
                    .uri("/request")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(SENT))
                    .unwrap(),
            )
            .await
            .unwrap();
        let out = axum::body::to_bytes(response.into_body(), MAX_BODY_BYTES)
            .await
            .unwrap();
        let out = String::from_utf8(out.to_vec()).unwrap();
        let (wire, seen) = out.split_once('|').unwrap();

        assert_eq!(wire, SENT, "wire body must be untouched");
        assert_ne!(seen, SENT, "the handler's body really was rewritten");
        assert!(
            seen.contains(r#""consumerPid""#) && !seen.contains("dspace:consumerPid"),
            "got: {seen}"
        );
    }
}
