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

//! Expanding one DSP JSON-LD message: the expanded document fields are read
//! from, and the canonical n-quads its semantic identity is hashed over.

use std::collections::HashSet;
use std::sync::Arc;

use contextual::WithContext;
use json_syntax::{Parse, Value as JsonSyntaxValue};
use locspan::{Location, Span};
use sophia_api::quad::Spog;
use sophia_api::term::Term as _;
use sophia_c14n::rdfc10;
use sophia_iri::Iri;
use sophia_jsonld::context::TryIntoContextRef;
use sophia_jsonld::json_ld::syntax::context::Value as ContextValue;
use sophia_jsonld::json_ld::{JsonLdProcessor, Options as JsonLdOptions, RemoteDocument};
use sophia_jsonld::loader::StaticLoader;
use sophia_jsonld::parser::RdfTerm;
use sophia_jsonld::vocabulary::{ArcIri, ArcVoc};
use sophia_term::ArcTerm;
use ymir::errors::{BadFormat, Errors, Outcome};

const DSP_CONTEXT_URL: &str = "https://w3id.org/dspace/2025/1/context.jsonld";
const DSP_CONTEXT_DOC: &str = include_str!("../../assets/dspace-2025-1-context.jsonld");
const DSP_ODRL_PROFILE_URL: &str = "https://w3id.org/dspace/2025/1/odrl-profile.jsonld";
const DSP_ODRL_PROFILE_DOC: &str = include_str!("../../assets/dspace-2025-1-odrl-profile.jsonld");
/// JSON-LD has no built-in prefixes: an undeclared `xsd:dateTime` is already a
/// syntactically valid IRI, so it expands silently as itself. Declared here once.
const DEFAULT_EXPAND_CONTEXT: &str =
    r#"{"@context": {"xsd": "http://www.w3.org/2001/XMLSchema#"}}"#;

/// The quads an expansion produces, in the owned shape `rdfc10` normalizes.
type Quads = HashSet<Spog<ArcTerm>>;

/// Source location carried by every JSON-LD term. Fixed by `sophia_jsonld`'s
/// loader and vocabulary.
type LdMeta = Location<ArcIri, Span>;

/// Processor options shared by both outputs, configured in one place:
/// [`DspCanonicalizer::options`].
type LdOptions = JsonLdOptions<ArcIri, LdMeta, ContextValue<LdMeta>>;

pub struct DspExpansion {
    /// The message expanded: full IRIs, values as `@id` or `@value` objects.
    /// The form to read fields from — invariant under the peer's aliasing.
    pub expanded: serde_json::Value,
    /// URDNA2015 / RDFC-1.0 canonical n-quads of the same expansion.
    pub canonical_n_quads: String,
}

/// Canonicalizes one DSP JSON-LD message. [`expand_once`](Self::expand_once)
/// keeps the expanded document as well.
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
        Ok(self.expand_once().await?.canonical_n_quads)
    }

    /// Expand once and keep both products: the expanded document and the
    /// canonical n-quads derived from it.
    pub async fn expand_once(&self) -> Outcome<DspExpansion> {
        let message = serde_json::to_string(&self.message).map_err(|_| {
            Errors::format(
                BadFormat::Received,
                "message is not serializable JSON",
                None,
            )
        })?;

        // The base and blank-node generator `sophia_jsonld`'s own parser uses, so the
        // quads — and the hash peers correlate on — stay bit-identical to its output.
        let base: ArcIri = Iri::new_unchecked(Arc::from("x-string://"));
        let json = JsonSyntaxValue::parse_str(&message, |span| Location::new(base.clone(), span))
            .map_err(|e| {
            Errors::format(
                BadFormat::Received,
                format!("body is not valid JSON: {e}"),
                None,
            )
        })?;
        let document = RemoteDocument::new(Some(base), None, json);

        let mut vocabulary = ArcVoc {};
        let mut generator = rdf_types::generator::Blank::new().with_metadata(Location::new(
            Iri::new_unchecked(Arc::from("x-bnode-gen://")),
            Span::default(),
        ));
        let mut loader = Self::loader()?;

        let mut to_rdf = document
            .to_rdf_with_using(
                &mut vocabulary,
                &mut generator,
                &mut loader,
                Self::options()?,
            )
            .await
            .map_err(|e| {
                Errors::format(
                    BadFormat::Received,
                    format!("JSON-LD expansion failed: {e}"),
                    None,
                )
            })?;

        // Expanded document first: `cloned_quads` borrows `to_rdf` mutably for longer
        // than the vocabulary, so nothing can read it afterwards.
        let expanded = Self::expanded_json(&to_rdf)?;
        // Canonical last:
        let quads: Quads = to_rdf.cloned_quads().map(Self::to_sophia_quad).collect();
        let canonical_n_quads = Self::to_canonical_n_quads(&quads)?;

        Ok(DspExpansion {
            expanded,
            canonical_n_quads,
        })
    }

    /// Only `expand_context` deviates from the JSON-LD 1.1 defaults
    /// `sophia_jsonld` passes — see [`DEFAULT_EXPAND_CONTEXT`].
    fn options() -> Outcome<LdOptions> {
        let expand_context = DEFAULT_EXPAND_CONTEXT.try_into_context_ref().map_err(|e| {
            Errors::crazy(
                format!("default expand @context is not valid JSON-LD: {e}"),
                None,
            )
        })?;
        Ok(LdOptions {
            expand_context: Some(expand_context),
            ..LdOptions::default()
        })
    }

    /// Render the expansion as `serde_json`; the vocabulary resolves the interned
    /// IRIs back to strings.
    fn expanded_json<G>(
        to_rdf: &sophia_jsonld::json_ld::ToRdf<'_, '_, ArcVoc, LdMeta, G>,
    ) -> Outcome<serde_json::Value>
    where
        G: rdf_types::MetaGenerator<ArcVoc, LdMeta>,
    {
        use sophia_jsonld::json_ld::print::Print;

        let rendered = to_rdf
            .document()
            .value()
            .with(to_rdf.vocabulary())
            .compact_print()
            .to_string();
        serde_json::from_str(&rendered)
            .map_err(|e| Errors::crazy(format!("expanded JSON-LD is not valid JSON: {e}"), None))
    }

    /// Serves the DSP context and the ODRL profile it imports. Any other
    /// `@context` URL fails the expansion rather than reaching for the network.
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

    /// Adapt one `json_ld` quad to the shape `rdfc10` normalizes. Same conversion
    /// `sophia_jsonld`'s parser applies internally; its own helper is private.
    fn to_sophia_quad(
        quad: rdf_types::Quad<
            rdf_types::Id<ArcIri, sophia_jsonld::vocabulary::ArcBnode>,
            rdf_types::Id<ArcIri, sophia_jsonld::vocabulary::ArcBnode>,
            rdf_types::Term<
                rdf_types::Id<ArcIri, sophia_jsonld::vocabulary::ArcBnode>,
                rdf_types::Literal<
                    rdf_types::literal::Type<ArcIri, sophia_jsonld::vocabulary::ArcTag>,
                    String,
                >,
            >,
            rdf_types::Id<ArcIri, sophia_jsonld::vocabulary::ArcBnode>,
        >,
    ) -> Spog<ArcTerm> {
        (
            [
                RdfTerm::from(quad.0).into_term::<ArcTerm>(),
                RdfTerm::from(quad.1).into_term::<ArcTerm>(),
                RdfTerm::from(quad.2).into_term::<ArcTerm>(),
            ],
            quad.3.map(|g| RdfTerm::from(g).into_term::<ArcTerm>()),
        )
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

    /// A `TransferRequestMessage` exactly as a peer put it on the wire.
    fn transfer_request_message() -> serde_json::Value {
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

    /// Pins the content identity of a real message: dropping the `sophia_jsonld`
    /// parser for a direct `json_ld` pass must not move a bit of this hash.
    #[tokio::test]
    async fn canonical_hash_is_stable() {
        use sha2::{Digest, Sha256};

        let n_quads = canon(transfer_request_message()).await;
        let hash: [u8; 32] = Sha256::digest(n_quads.as_bytes()).into();
        assert_eq!(
            hex_lower(&hash),
            "2a74c50cad115b2b9e21b3d5c580d7263cc754328e5eb4b159416f71f2ef5b1a"
        );
    }

    /// Why the expanded form is worth carrying: full IRIs, and `@id` terms visibly
    /// distinct from literals. Neither is recoverable from the compact body.
    #[tokio::test]
    async fn expanded_document_exposes_iris_and_term_kinds() {
        let expansion = DspCanonicalizer::new(transfer_request_message())
            .expand_once()
            .await
            .unwrap();

        let node = &expansion.expanded.as_array().unwrap()[0];
        assert_eq!(
            node["https://w3id.org/dspace/2025/1/consumerPid"][0]["@id"],
            "urn:uuid:32541fe6-c580-409e-85a8-8a9a32fbe833",
            "consumerPid is @id-typed by the DSP context"
        );
        assert_eq!(
            node["https://w3id.org/dspace/2025/1/callbackAddress"][0]["@value"],
            "https://example.com/callback",
            "callbackAddress is a plain literal"
        );
        // `format` is `dct:format` in the DSP context, not a `dspace:` term.
        assert_eq!(
            node["http://purl.org/dc/terms/format"][0]["@id"],
            "example:HTTP_PUSH"
        );
    }

    /// Alias independence: a `dspace:`-prefixed message expands to the same
    /// predicates, where reading the compact body by key would find nothing.
    #[tokio::test]
    async fn prefixed_and_stripped_terms_expand_alike() {
        let stripped = DspCanonicalizer::new(json!({
            "@context": "https://w3id.org/dspace/2025/1/context.jsonld",
            "@type": "TransferStartMessage",
            "consumerPid": "urn:uuid:cc"
        }))
        .expand_once()
        .await
        .unwrap();

        let prefixed = DspCanonicalizer::new(json!({
            "@context": {"dspace": "https://w3id.org/dspace/2025/1/"},
            "@type": "dspace:TransferStartMessage",
            "dspace:consumerPid": {"@id": "urn:uuid:cc"}
        }))
        .expand_once()
        .await
        .unwrap();

        let pid = |v: &serde_json::Value| {
            v.as_array().unwrap()[0]["https://w3id.org/dspace/2025/1/consumerPid"][0]["@id"]
                .as_str()
                .unwrap()
                .to_string()
        };
        assert_eq!(pid(&stripped.expanded), pid(&prefixed.expanded));
    }

    fn hex_lower(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
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

    /// A peer may use `xsd:` without declaring it. Undeclared, it expands to the
    /// bogus IRI `<xsd:dateTime>` — valid, silent, and matching nobody.
    #[tokio::test]
    async fn undeclared_xsd_prefix_still_expands() {
        let out = canon(json!({
            "@context": {"ex": "http://example.org/"},
            "@id": "http://example.org/m1",
            "ex:issued": {"@value": "2026-09-02T00:00:00Z", "@type": "xsd:dateTime"}
        }))
        .await;
        assert!(
            out.contains("http://www.w3.org/2001/XMLSchema#dateTime"),
            "xsd: must resolve even when the message omits the prefix, got: {out}"
        );
        assert!(
            !out.contains("<xsd:"),
            "prefix must not survive as an IRI: {out}"
        );
    }

    /// The same gap on the wire: the ODRL profile declares no `xsd` prefix, yet
    /// constraint literals carry `xsd:` datatypes.
    #[tokio::test]
    async fn odrl_offer_constraint_datatypes_expand() {
        let out = canon(json!({
            "@context": [DSP_ODRL_PROFILE_URL],
            "@id": "urn:policy:0000-00-0",
            "@type": "Offer",
            "permission": [{
                "@type": "Permission",
                "action": "use",
                "constraint": [{
                    "leftOperand": "odrl:dateTime",
                    "operator": "odrl:lteq",
                    "rightOperand": {"@type": "xsd:dateTime", "@value": "2026-12-31T23:59:59Z"}
                }, {
                    "leftOperand": "odrl:count",
                    "operator": "odrl:lteq",
                    "rightOperand": {"@type": "xsd:integer", "@value": "2"}
                }]
            }],
            "target": {"@id": "urn:dataset:0000-00-0"}
        }))
        .await;
        assert!(
            out.contains(r#""2026-12-31T23:59:59Z"^^<http://www.w3.org/2001/XMLSchema#dateTime>"#),
            "got: {out}"
        );
        assert!(
            out.contains(r#""2"^^<http://www.w3.org/2001/XMLSchema#integer>"#),
            "got: {out}"
        );
        assert!(!out.contains("<xsd:"), "got: {out}");
    }
}
