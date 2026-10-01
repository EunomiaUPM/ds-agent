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

//! Protocol-agnostic RDF toolkit: JSON-LD expansion, canonicalization, and typed model extraction.
//!
//! # RDF API Tutorial & Quick Start
//!
//! This module provides a high-level, protocol-agnostic RDF engine with:
//! 1. **JSON-LD Expansion**: Invariant under peer aliasing and compacted keys.
//! 2. **Canonicalization (URDNA2015 / RDFC-1.0)**: Deterministic, key-order independent n-quads.
//! 3. **Content Hashing**: SHA-256 digest over normalized quads.
//! 4. **Typed Extraction**: Deserialization into domain structs via [`FromRdf`].
//! 5. **Context Management**: In-memory caching, filesystem preloading, and HTTP fallback.
//! 6. **Protocol Profiles**: Pluggable configurations via [`RdfProfile`].
//!
//! ---
//!
//! ## 1. Defining a Domain Entity with `FromRdf`
//!
//! Implement [`FromRdf`] on your target domain structs. Declare `TYPE_IRI` so the engine
//! can automatically discover the root node, even if the payload is wrapped in an `@graph`:
//!
//! ```rust,ignore
//! use common::rdf::{FromRdf, RdfNode};
//! use ymir::errors::Outcome;
//!
//! pub struct EndpointProperty {
//!     pub name: String,
//!     pub value: String,
//! }
//!
//! impl FromRdf for EndpointProperty {
//!     const TYPE_IRI: Option<&'static str> = Some("https://w3id.org/dspace/2025/1/EndpointProperty");
//!
//!     fn from_rdf(node: &RdfNode<'_, '_>) -> Outcome<Self> {
//!         Ok(Self {
//!             name: node.get_string("https://w3id.org/dspace/2025/1/name")?,
//!             value: node.get_string("https://w3id.org/dspace/2025/1/value")?,
//!         })
//!     }
//! }
//!
//! pub struct TransferRequest {
//!     pub consumer_pid: String,
//!     pub agreement_id: String,
//!     pub callback_address: Option<String>,
//!     pub properties: Vec<EndpointProperty>,
//! }
//!
//! impl FromRdf for TransferRequest {
//!     const TYPE_IRI: Option<&'static str> = Some("https://w3id.org/dspace/2025/1/TransferRequestMessage");
//!
//!     fn from_rdf(node: &RdfNode<'_, '_>) -> Outcome<Self> {
//!         Ok(Self {
//!             consumer_pid: node.get_string("https://w3id.org/dspace/2025/1/consumerPid")?,
//!             agreement_id: node.get_string("https://w3id.org/dspace/2025/1/agreementId")?,
//!             callback_address: node.get_opt_string("https://w3id.org/dspace/2025/1/callbackAddress")?,
//!             properties: node.get_list("https://w3id.org/dspace/2025/1/properties")?,
//!         })
//!     }
//! }
//! ```
//!
//! ---
//!
//! ## 2. Typed Extraction using `RdfEngine`
//!
//! To extract the typed struct directly from incoming JSON-LD:
//!
//! ```rust,ignore
//! use common::rdf::RdfEngine;
//!
//! async fn process_message(json_payload: &serde_json::Value) -> ymir::errors::Outcome<()> {
//!     // Use RdfEngine::dsp() for Dataspace Protocol (or RdfEngine::new() for generic RDF)
//!     let engine = RdfEngine::dsp();
//!
//!     // 1. Direct typed extraction
//!     let request: TransferRequest = engine.extract(json_payload).await?;
//!
//!     // 2. Or single-pass extraction + canonical SHA-256 content hash:
//!     let (request, hash_hex) = engine.extract_with_hash::<TransferRequest>(json_payload).await?;
//!     println!("Extracted request with canonical hash: {hash_hex}");
//!
//!     Ok(())
//! }
//! ```
//!
//! ---
//!
//! ## 3. Canonicalization and Content Hashing
//!
//! Compute URDNA2015 / RDFC-1.0 canonical n-quads and SHA-256 hashes independently:
//!
//! ```rust,ignore
//! use common::rdf::RdfEngine;
//!
//! async fn compute_identity(json_payload: &serde_json::Value) -> ymir::errors::Outcome<()> {
//!     let engine = RdfEngine::dsp();
//!
//!     // Deterministic canonical n-quads:
//!     let n_quads: String = engine.canonicalize(json_payload).await?;
//!
//!     // Canonical SHA-256 hex digest:
//!     let hash: String = engine.hash(json_payload).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ---
//!
//! ## 4. Low-Level Document Navigation with `ExpandedDoc` and `RdfNode`
//!
//! When manual navigation is needed, inspect the expanded document directly:
//!
//! ```rust,ignore
//! use common::rdf::{ExpandedDoc, RdfEngine};
//!
//! async fn inspect_expanded(json_payload: &serde_json::Value) -> ymir::errors::Outcome<()> {
//!     let expansion = RdfEngine::dsp().expand(json_payload).await?;
//!     let doc = ExpandedDoc::new(&expansion.expanded).expect("valid expanded array");
//!
//!     // Iterate over top-level nodes:
//!     for node in doc.nodes() {
//!         println!("Node ID: {:?}", node.id());
//!         for type_iri in node.types() {
//!             println!("  Type: {type_iri}");
//!         }
//!     }
//!
//!     // Find root node by type:
//!     let root = doc.root_node::<TransferRequest>()?;
//!     let pid = root.get_str("https://w3id.org/dspace/2025/1/consumerPid")?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ---
//!
//! ## 5. Defining Custom Protocol Profiles with `RdfProfile`
//!
//! If you need to support another specification (e.g. Gaia-X, DCAT-AP, Solid), implement [`RdfProfile`]:
//!
//! ```rust,ignore
//! use common::rdf::{RdfContextLoader, RdfEngine, RdfProfile};
//!
//! pub struct MyCustomProfile;
//!
//! impl RdfProfile for MyCustomProfile {
//!     fn configure_loader(loader: &mut RdfContextLoader) {
//!         let _ = loader.register_asset(
//!             "https://example.org/context.jsonld",
//!             include_str!("../path/to/context.jsonld"),
//!         );
//!     }
//!
//!     fn default_expand_context() -> Option<&'static str> {
//!         Some(r#"{"@context": {"xsd": "http://www.w3.org/2001/XMLSchema#"}}"#)
//!     }
//! }
//!
//! // Instantiate the engine configured with your custom profile:
//! let custom_engine = RdfEngine::with_profile::<MyCustomProfile>();
//! ```

pub mod canonical;
pub mod engine;
pub mod loader;
pub mod node;
pub mod profile;
pub mod traits;

pub use canonical::{RdfCanonicalizer, RdfExpansion};
pub use engine::RdfEngine;
pub use loader::{RdfContextLoader, RdfLoaderError};
pub use node::{ExpandedDoc, RdfNode};
pub use profile::{DefaultProfile, RdfProfile};
pub use traits::FromRdf;
