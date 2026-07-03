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

//! Canonicalization of DSP JSON-LD messages.
//!
//! Expands a Dataspace-Protocol message to RDF against the embedded DSP
//! `@context` and emits URDNA2015 / RDFC-1.0 canonical n-quads. The result is a
//! stable content identity two peers compute *identically* for the same message
//! regardless of JSON key order — the basis for idempotency and message signing.
//!
//! Feed it the message as received on the wire: expansion coerces `@id`-typed
//! terms (e.g. `providerPid`) only for the exact term forms the `@context`
//! defines, so a prefixed (`dspace:providerPid`) and a stripped (`providerPid`)
//! payload are *not* guaranteed to canonicalize alike. To interoperate, both
//! peers must canonicalize the same representation.
//!
//! Expansion is fully offline: the `@context` (and the ODRL profile it
//! `@import`s) are embedded at compile time, so the hash never depends on a
//! network fetch.

use std::collections::HashSet;
use std::sync::Arc;

use json_syntax::{Parse, Value as JsonSyntaxValue};
use locspan::{Location, Span};
use sophia_api::quad::Spog;
use sophia_api::source::QuadSource;
use sophia_c14n::rdfc10;
use sophia_iri::Iri;
use sophia_jsonld::loader::StaticLoader;
use sophia_jsonld::vocabulary::ArcIri;
use sophia_jsonld::{JsonLdOptions, JsonLdParser};
use sophia_term::ArcTerm;
use ymir::errors::{BadFormat, Errors, Outcome};

const DSP_CONTEXT_URL: &str = "https://w3id.org/dspace/2025/1/context.jsonld";
const DSP_CONTEXT_DOC: &str = include_str!("../../assets/dspace-2025-1-context.jsonld");
/// The DSP context `@import`s the ODRL profile inside several term-scoped
/// contexts; the processor validates those eagerly when the context loads, so
/// this doc must be available even to canonicalize a message that carries no ODRL.
const DSP_ODRL_PROFILE_URL: &str = "https://w3id.org/dspace/2025/1/odrl-profile.jsonld";
const DSP_ODRL_PROFILE_DOC: &str = include_str!("../../assets/dspace-2025-1-odrl-profile.jsonld");

/// The set of RDF quads produced by expanding a message. Terms are owned
/// ([`ArcTerm`]) so the set is hashable — the shape [`rdfc10::normalize`] needs.
type Quads = HashSet<Spog<ArcTerm>>;

/// Canonicalizes one DSP JSON-LD message. Construct it with the parsed message,
/// then call [`canonicalize`](Self::canonicalize).
pub struct DspCanonicalizer {
    message: serde_json::Value,
}

impl DspCanonicalizer {
    /// Wrap a parsed DSP message for canonicalization.
    pub fn new(message: serde_json::Value) -> Self {
        Self { message }
    }

    /// Expand to RDF and return the URDNA2015 / RDFC-1.0 canonical n-quads.
    pub async fn canonicalize(&self) -> Outcome<String> {
        let quads = self.expand().await?;
        Self::to_canonical_n_quads(&quads)
    }

    /// Build the in-memory document loader: it serves the DSP context and the
    /// ODRL profile it imports, and rejects any other `@context` URL (NotFound →
    /// expansion error) rather than reaching for the network.
    fn loader() -> Outcome<StaticLoader<ArcIri, Span>> {
        let embed = |url: &str, doc: &'static str| -> Outcome<(ArcIri, _)> {
            let iri: ArcIri = Iri::new_unchecked(Arc::from(url));
            let parsed = JsonSyntaxValue::parse_str(doc, |span| Location::new(iri.clone(), span))
                .map_err(|_| {
                Errors::crazy("embedded JSON-LD context is not valid JSON", None)
            })?;
            Ok((iri, parsed))
        };
        let (ctx_url, ctx_doc) = embed(DSP_CONTEXT_URL, DSP_CONTEXT_DOC)?;
        let (odrl_url, odrl_doc) = embed(DSP_ODRL_PROFILE_URL, DSP_ODRL_PROFILE_DOC)?;
        Ok(StaticLoader::new()
            .with(ctx_url, ctx_doc)
            .with(odrl_url, odrl_doc))
    }

    /// Expand the message to a set of RDF quads (async: JSON-LD expansion drives
    /// the in-memory loader as a future).
    async fn expand(&self) -> Outcome<Quads> {
        let message = serde_json::to_string(&self.message).map_err(|_| {
            Errors::format(
                BadFormat::Received,
                "message is not serializable JSON",
                None,
            )
        })?;

        let options = JsonLdOptions::new().with_document_loader(Self::loader()?);
        let parser = JsonLdParser::new_with_options(options);

        parser
            .async_parse_str(&message)
            .await
            .collect_quads()
            .map_err(|e| {
                Errors::format(
                    BadFormat::Received,
                    format!("JSON-LD expansion failed: {e}"),
                    None,
                )
            })
    }

    /// Serialize an expanded quad set to URDNA2015 / RDFC-1.0 canonical n-quads.
    fn to_canonical_n_quads(quads: &Quads) -> Outcome<String> {
        let mut buf = Vec::new();
        rdfc10::normalize(quads, &mut buf)
            .map_err(|e| Errors::crazy(format!("RDF canonicalization failed: {e}"), None))?;
        String::from_utf8(buf).map_err(|_| Errors::crazy("canonical n-quads are not UTF-8", None))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    async fn canon(v: serde_json::Value) -> String {
        DspCanonicalizer::new(v).canonicalize().await.unwrap()
    }

    #[tokio::test]
    async fn deterministic_and_key_order_independent() {
        let a = canon(json!({
            "@context": DSP_CONTEXT_URL,
            "@type": "TransferStartMessage",
            "providerPid": "urn:uuid:pp",
            "consumerPid": "urn:uuid:cc"
        }))
        .await;
        // Same message, fields in a different source order.
        let b = canon(json!({
            "@context": DSP_CONTEXT_URL,
            "consumerPid": "urn:uuid:cc",
            "@type": "TransferStartMessage",
            "providerPid": "urn:uuid:pp"
        }))
        .await;
        assert!(!a.is_empty(), "expansion must yield quads");
        assert_eq!(a, b, "canonicalization must be key-order independent");

        // Different content → different canonical form.
        let c = canon(json!({
            "@context": DSP_CONTEXT_URL,
            "@type": "TransferStartMessage",
            "providerPid": "urn:uuid:XX",
            "consumerPid": "urn:uuid:cc"
        }))
        .await;
        assert_ne!(a, c, "different content must canonicalize differently");
    }
}
