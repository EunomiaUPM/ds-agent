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

//! Reading a catalog RPC body, typed with the eunomia context, off its expansion.

use common::eunomia::{EunomiaProfile, EUNOMIA_CONTEXT_URL, EUNOMIA_NS};
use common::rdf::{RdfEngine, RdfNode, RdfProtocolTrait};
use serde_json::{json, Value};
use ymir::errors::Outcome;

use crate::entities::vocab::Vocab;
use crate::protocols::dsp::entities::message_types::CatalogDspMessageType;
use crate::protocols::dsp::entities::protocol_fields::CatalogProtocolFields;
use crate::protocols::dsp::entities::rdf_extractor_dsp::DspCatalog;

/// The catalog RPC, as an extraction target.
#[derive(Debug, Clone)]
pub struct EunomiaCatalog;

impl EunomiaCatalog {
    /// Prefix of RPC types over the DSP message they send.
    const RPC_PREFIX: &'static str = "Rpc";
}

impl RdfProtocolTrait for EunomiaCatalog {
    type MessageType = CatalogDspMessageType;
    type Fields = CatalogProtocolFields;

    fn engine() -> &'static RdfEngine {
        EunomiaProfile::shared()
    }

    fn context() -> Value {
        json!(EUNOMIA_CONTEXT_URL)
    }

    /// RPC types are the DSP message they send, prefixed: `RpcCatalogRequestMessage`.
    fn compact_type(message: &CatalogDspMessageType) -> String {
        format!("{}{message}", Self::RPC_PREFIX)
    }

    fn type_iri(message: &CatalogDspMessageType) -> String {
        EunomiaProfile::term(&Self::compact_type(message))
    }

    fn message_type(node: &RdfNode<'_, '_>) -> Option<CatalogDspMessageType> {
        node.types()
            .filter_map(|iri| iri.strip_prefix(EUNOMIA_NS)?.strip_prefix(Self::RPC_PREFIX))
            .find_map(|term| term.parse().ok())
    }

    fn extract(node: &RdfNode<'_, '_>) -> Outcome<CatalogProtocolFields> {
        Ok(CatalogProtocolFields {
            filter: node.values(&format!("{}filter", Vocab::DSPACE)).cloned().collect(),
            dataset: node
                .iri_or_literal(&format!("{}dataset", Vocab::DSPACE))
                .map(DspCatalog::below_base),
            associated_agent_peer: node
                .iri_or_literal(&EunomiaProfile::term("associatedAgentPeer"))
                .map(str::to_string),
            no_cache: node
                .get_opt_bool(&EunomiaProfile::term("noCache"))?
                .unwrap_or(false),
        })
    }
}
