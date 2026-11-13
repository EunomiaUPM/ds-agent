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

//! Unified high-level RDF engine for JSON-LD expansion, canonicalization, and typed extraction.

use ymir::errors::{BadFormat, Errors, Outcome};

use crate::rdf::canonical::{RdfCanonicalizer, RdfExpansion};
use crate::rdf::loader::RdfContextLoader;
use crate::rdf::node::ExpandedDoc;
use crate::rdf::profile::{DefaultProfile, RdfProfile};
use crate::rdf::traits::FromRdf;

/// Generic facade for JSON-LD expansion, canonicalization, and typed RDF extraction.
#[derive(Clone, Debug)]
pub struct RdfEngine {
    loader: RdfContextLoader,
    expand_context: Option<String>,
}

impl Default for RdfEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl RdfEngine {
    /// Creates a generic engine instance configured with the default W3C XSD profile.
    pub fn new() -> Self {
        Self::with_profile::<DefaultProfile>()
    }

    /// Creates an engine instance pre-configured with Dataspace Protocol 2025-1 profile.
    pub fn dsp() -> Self {
        Self::with_profile::<crate::dsp_common::rdf::DspProfile>()
    }

    /// Creates an engine instance configured with a specific protocol profile.
    pub fn with_profile<P: RdfProfile>() -> Self {
        let mut loader = RdfContextLoader::new();
        P::configure_loader(&mut loader);
        Self {
            loader,
            expand_context: P::default_expand_context().map(str::to_string),
        }
    }

    /// Creates an engine instance configured with a custom context loader.
    pub fn with_loader(loader: RdfContextLoader) -> Self {
        Self {
            loader,
            expand_context: DefaultProfile::default_expand_context().map(str::to_string),
        }
    }

    /// Returns a reference to the active context loader.
    pub fn loader(&self) -> &RdfContextLoader {
        &self.loader
    }

    /// Returns a mutable reference to the active context loader.
    pub fn loader_mut(&mut self) -> &mut RdfContextLoader {
        &mut self.loader
    }

    fn canonicalizer(&self, message: &serde_json::Value) -> RdfCanonicalizer {
        let mut canonicalizer = RdfCanonicalizer::with_loader(message.clone(), self.loader.clone());
        if let Some(ctx) = &self.expand_context {
            canonicalizer = canonicalizer.with_expand_context(ctx.clone());
        }
        canonicalizer
    }

    /// Expands a JSON-LD message into an expanded document and canonical n-quads.
    pub async fn expand(&self, message: &serde_json::Value) -> Outcome<RdfExpansion> {
        self.canonicalizer(message).expand_once().await
    }

    /// Expands a JSON-LD message into expanded JSON without running RDFC-1.0 canonicalization.
    pub async fn expand_only(&self, message: &serde_json::Value) -> Outcome<serde_json::Value> {
        self.canonicalizer(message).expand_json_only().await
    }

    /// Produces the URDNA2015 / RDFC-1.0 canonical n-quads for a JSON-LD message.
    pub async fn canonicalize(&self, message: &serde_json::Value) -> Outcome<String> {
        Ok(self.expand(message).await?.canonical_n_quads)
    }

    /// Computes the SHA-256 hash over the message's canonical n-quads representation.
    pub async fn hash(&self, message: &serde_json::Value) -> Outcome<String> {
        Ok(Self::hex(&self.expand(message).await?.canonical_hash()))
    }

    fn hex(hash: &[u8; 32]) -> String {
        hash.iter().map(|b| format!("{b:02x}")).collect()
    }

    /// Internal helper: expands, parses document, and extracts the target model.
    async fn extract_parts<T: FromRdf>(
        &self,
        message: &serde_json::Value,
    ) -> Outcome<(RdfExpansion, T)> {
        let expansion = self.expand(message).await?;
        let doc = ExpandedDoc::new(&expansion.expanded).ok_or_else(|| {
            Errors::format(
                BadFormat::Received,
                "expanded JSON-LD is not a top-level array",
                None,
            )
        })?;
        let root = doc.root_node::<T>()?;
        let target = T::from_rdf(&root)?;
        Ok((expansion, target))
    }

    /// Extracts a typed domain structure implementing `FromRdf` from a JSON-LD message.
    pub async fn extract<T: FromRdf>(&self, message: &serde_json::Value) -> Outcome<T> {
        let (_, target) = self.extract_parts(message).await?;
        Ok(target)
    }

    /// Extracts a domain structure and computes the canonical content hash in a single pass.
    pub async fn extract_with_hash<T: FromRdf>(
        &self,
        message: &serde_json::Value,
    ) -> Outcome<(T, String)> {
        let (expansion, target) = self.extract_parts(message).await?;
        Ok((target, Self::hex(&expansion.canonical_hash())))
    }

    /// Helper: extracts directly from a raw JSON-LD string using the default engine.
    pub async fn extract_from_str<T: FromRdf>(raw_json_ld: &str) -> Outcome<T> {
        let value: serde_json::Value = serde_json::from_str(raw_json_ld).map_err(|e| {
            Errors::format(
                BadFormat::Received,
                format!("Failed to parse JSON string: {e}"),
                None,
            )
        })?;
        Self::new().extract(&value).await
    }

    /// Helper: extracts directly from a serde JSON value using the default engine.
    pub async fn extract_from_json<T: FromRdf>(value: &serde_json::Value) -> Outcome<T> {
        Self::new().extract(value).await
    }
}
