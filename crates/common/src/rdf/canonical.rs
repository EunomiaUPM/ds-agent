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

//! Protocol-agnostic JSON-LD expansion and URDNA2015 / RDFC-1.0 canonicalization.

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
use sophia_jsonld::parser::RdfTerm;
use sophia_jsonld::vocabulary::{ArcIri, ArcVoc};
use sophia_term::ArcTerm;
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::rdf::loader::RdfContextLoader;

/// Standard W3C XML Schema datatypes context used by default.
pub const DEFAULT_EXPAND_CONTEXT: &str =
    r#"{"@context": {"xsd": "http://www.w3.org/2001/XMLSchema#"}}"#;

type Quads = HashSet<Spog<ArcTerm>>;
type LdMeta = Location<ArcIri, Span>;
type LdOptions = JsonLdOptions<ArcIri, LdMeta, ContextValue<LdMeta>>;

/// Products of expanding a JSON-LD message: expanded document and canonical n-quads.
pub struct RdfExpansion {
    pub expanded: serde_json::Value,
    pub canonical_n_quads: String,
}

/// Canonicalizes a JSON-LD document to URDNA2015 / RDFC-1.0 quads.
pub struct RdfCanonicalizer {
    message: serde_json::Value,
    loader: Option<RdfContextLoader>,
    expand_context: Option<String>,
}

impl RdfCanonicalizer {
    /// Constructs a canonicalizer for the given parsed JSON-LD document.
    pub fn new(message: serde_json::Value) -> Self {
        Self {
            message,
            loader: None,
            expand_context: Some(DEFAULT_EXPAND_CONTEXT.to_string()),
        }
    }

    /// Constructs a canonicalizer with a dedicated context loader.
    pub fn with_loader(message: serde_json::Value, loader: RdfContextLoader) -> Self {
        Self {
            message,
            loader: Some(loader),
            expand_context: Some(DEFAULT_EXPAND_CONTEXT.to_string()),
        }
    }

    /// Overrides the default `@context` used during expansion.
    pub fn with_expand_context(mut self, context: impl Into<String>) -> Self {
        self.expand_context = Some(context.into());
        self
    }

    /// Expands the message and returns URDNA2015 / RDFC-1.0 canonical n-quads.
    pub async fn canonicalize(&self) -> Outcome<String> {
        Ok(self.expand_once().await?.canonical_n_quads)
    }

    /// Expands the message and returns both expanded JSON and canonical n-quads.
    pub async fn expand_once(&self) -> Outcome<RdfExpansion> {
        let message = serde_json::to_string(&self.message).map_err(|_| {
            Errors::format(
                BadFormat::Received,
                "message is not serializable JSON",
                None,
            )
        })?;

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
        let mut loader = self.loader.clone().unwrap_or_default();

        let mut to_rdf = document
            .to_rdf_with_using(
                &mut vocabulary,
                &mut generator,
                &mut loader,
                self.options()?,
            )
            .await
            .map_err(|e| {
                Errors::format(
                    BadFormat::Received,
                    format!("JSON-LD expansion failed: {e}"),
                    None,
                )
            })?;

        let expanded = Self::expanded_json(&to_rdf)?;
        let quads: Quads = to_rdf.cloned_quads().map(Self::to_sophia_quad).collect();
        let canonical_n_quads = Self::to_canonical_n_quads(&quads)?;

        Ok(RdfExpansion {
            expanded,
            canonical_n_quads,
        })
    }

    /// Prepares JsonLdOptions including optional expand_context.
    fn options(&self) -> Outcome<LdOptions> {
        let mut options = LdOptions::default();
        if let Some(ctx_str) = &self.expand_context {
            let expand_context = ctx_str.as_str().try_into_context_ref().map_err(|e| {
                Errors::crazy(
                    format!("default expand @context is not valid JSON-LD: {e}"),
                    None,
                )
            })?;
            options.expand_context = Some(expand_context);
        }
        Ok(options)
    }

    /// Renders the expanded document into a Serde JSON value.
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

    /// Converts an rdf_types Quad to a Sophia Spog quad representation.
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

    /// Serializes an expanded quad set into URDNA2015 / RDFC-1.0 canonical n-quads.
    fn to_canonical_n_quads(quads: &Quads) -> Outcome<String> {
        let mut buf = Vec::new();
        rdfc10::normalize(quads, &mut buf)
            .map_err(|e| Errors::crazy(format!("RDF canonicalization failed: {e}"), None))?;
        String::from_utf8(buf).map_err(|_| Errors::crazy("canonical n-quads are not UTF-8", None))
    }
}
