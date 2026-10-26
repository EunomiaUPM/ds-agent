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

//! Reading the DSP catalog messages (DSP 6.2) off their expansion.

use common::dsp_common::context_field::ContextField;
use common::dsp_common::rdf::DspProfile;
use common::rdf::{RdfCanonicalizer, RdfEngine, RdfNode, RdfProtocolTrait};
use serde_json::Value;
use ymir::errors::Outcome;

use crate::entities::vocab::Vocab;
use crate::protocols::dsp::entities::message_types::CatalogDspMessageType;
use crate::protocols::dsp::entities::protocol_fields::CatalogProtocolFields;

/// The catalog protocol, as an extraction target.
#[derive(Debug, Clone)]
pub struct DspCatalog;

impl RdfProtocolTrait for DspCatalog {
    type MessageType = CatalogDspMessageType;
    type Fields = CatalogProtocolFields;

    fn engine() -> &'static RdfEngine {
        DspProfile::shared()
    }

    fn context() -> Value {
        serde_json::to_value(ContextField::default()).unwrap_or_default()
    }

    fn compact_type(message: &CatalogDspMessageType) -> String {
        message.to_string()
    }

    fn type_iri(message: &CatalogDspMessageType) -> String {
        match message {
            CatalogDspMessageType::Catalog => Vocab::DCAT_CATALOG.to_string(),
            CatalogDspMessageType::Dataset => Vocab::DCAT_DATASET.to_string(),
            other => format!("{}{other}", Vocab::DSPACE),
        }
    }

    /// Only the DSP namespace counts: in expanded form every `@type` is a full IRI.
    fn message_type(node: &RdfNode<'_, '_>) -> Option<CatalogDspMessageType> {
        node.types()
            .filter_map(|iri| iri.strip_prefix(Vocab::DSPACE))
            .find_map(|term| term.parse().ok())
    }

    fn extract(node: &RdfNode<'_, '_>) -> Outcome<CatalogProtocolFields> {
        Ok(CatalogProtocolFields {
            filter: node.values(&format!("{}filter", Vocab::DSPACE)).cloned().collect(),
            dataset: node
                .iri_or_literal(&format!("{}dataset", Vocab::DSPACE))
                .map(Self::below_base),
            associated_agent_peer: None,
            no_cache: false,
        })
    }
}

impl DspCatalog {
    /// A relative id expands under the document base; it is read back as sent.
    pub fn below_base(expanded: &str) -> String {
        expanded
            .strip_prefix(RdfCanonicalizer::DOCUMENT_BASE)
            .map(|rest| rest.trim_start_matches('/'))
            .unwrap_or(expanded)
            .to_string()
    }
}
