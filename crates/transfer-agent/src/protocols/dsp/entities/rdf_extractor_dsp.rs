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

//! Reading the Transfer Process Protocol fields (DSP 9.2) off an expanded
//! message. Everything protocol-agnostic lives in `common::rdf`.

use common::dsp_common::data_address::{DataAddress, EndpointProperty};
use common::rdf::{ExpandedDoc, RdfNode};
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::protocols::dsp::entities::message_types::TransferDSPMessageType;
use crate::protocols::dsp::entities::protocol_fields::TransferProtocolFields;

/// Protocol-specific field extractor interface.
pub trait ExtractProtocolFields {
    type MessageType: std::fmt::Display;
    type Fields;

    fn type_iri(message: &Self::MessageType) -> String;
    fn extract(node: &RdfNode<'_, '_>) -> Outcome<Self::Fields>;
    fn message_type(node: &RdfNode<'_, '_>) -> Option<Self::MessageType>;

    fn root_message<'d, 'a>(
        doc: &'d ExpandedDoc<'a>,
    ) -> Outcome<(Self::MessageType, RdfNode<'d, 'a>)> {
        let mut matching = doc
            .nodes()
            .filter_map(|node| Self::message_type(&node).map(|kind| (kind, node)));
        let found = matching.next().ok_or_else(|| {
            Errors::format(
                BadFormat::Received,
                "expanded body declares no message type",
                None,
            )
        })?;
        if matching.next().is_some() {
            return Err(Errors::format(
                BadFormat::Received,
                "expanded body declares more than one message node",
                None,
            ));
        }
        Ok(found)
    }
}

/// Namespace every DSP term expands into.
const DSPACE: &str = "https://w3id.org/dspace/2025/1/";
/// `format` is Dublin Core in the DSP context, not a `dspace:` term.
const DCT_FORMAT: &str = "http://purl.org/dc/terms/format";

/// The transfer protocol, as an extraction target.
pub struct DspTransfer;

impl ExtractProtocolFields for DspTransfer {
    type MessageType = TransferDSPMessageType;
    type Fields = TransferProtocolFields;

    fn type_iri(message: &TransferDSPMessageType) -> String {
        Self::dspace(&message.to_string())
    }

    /// Only the DSP namespace counts: in expanded form every `@type` is a full
    /// IRI, so a same-named term from elsewhere is a different message.
    fn message_type(node: &RdfNode<'_, '_>) -> Option<TransferDSPMessageType> {
        node.types()
            .filter_map(|iri| iri.strip_prefix(DSPACE))
            .find_map(|term| term.parse().ok())
    }

    fn extract(node: &RdfNode<'_, '_>) -> Outcome<TransferProtocolFields> {
        Ok(TransferProtocolFields {
            consumer_pid: Self::owned(node, &Self::dspace("consumerPid")),
            provider_pid: Self::owned(node, &Self::dspace("providerPid")),
            agreement_id: Self::owned(node, &Self::dspace("agreementId")),
            callback_address: Self::owned(node, &Self::dspace("callbackAddress")),
            format: node.iri_or_literal(DCT_FORMAT).map(str::to_string),
            data_address: Self::data_address(node)?,
            code: Self::owned(node, &Self::dspace("code")),
            reason: node
                .values(&Self::dspace("reason"))
                .filter_map(|v| v.get("@value").and_then(serde_json::Value::as_str))
                .map(str::to_string)
                .collect(),
        })
    }
}

impl DspTransfer {
    /// A `dataAddress` is optional, but a present one missing a required member is
    /// malformed rather than absent.
    fn data_address(node: &RdfNode<'_, '_>) -> Outcome<Option<DataAddress>> {
        let Some(address) = node.object(&Self::dspace("dataAddress")) else {
            return Ok(None);
        };
        let endpoint_type = Self::owned(&address, &Self::dspace("endpointType"))
            .ok_or_else(|| Self::missing("dataAddress.endpointType"))?;

        // RDF is unordered: this array is not the sender's order. Look properties up
        // by name, never by index.
        let endpoint_properties = address
            .objects(&Self::dspace("endpointProperties"))
            .map(|p| Self::endpoint_property(&p))
            .collect::<Outcome<Vec<_>>>()?;

        Ok(Some(DataAddress {
            _type: Self::term_of(&address).unwrap_or_else(|| "DataAddress".to_string()),
            endpoint_type,
            // OPTIONAL per DSP Appendix A; whether a message needs one is a domain
            // rule, since it depends on the connector behind the `format`.
            endpoint: Self::owned(&address, &Self::dspace("endpoint")),
            endpoint_properties,
        }))
    }

    fn endpoint_property(node: &RdfNode<'_, '_>) -> Outcome<EndpointProperty> {
        Ok(EndpointProperty {
            _type: Self::term_of(node).unwrap_or_else(|| "EndpointProperty".to_string()),
            name: Self::owned(node, &Self::dspace("name"))
                .ok_or_else(|| Self::missing("dataAddress.endpointProperties[].name"))?,
            value: Self::owned(node, &Self::dspace("value"))
                .ok_or_else(|| Self::missing("dataAddress.endpointProperties[].value"))?,
        })
    }

    /// The full IRI a DSP term expands to.
    fn dspace(term: &str) -> String {
        format!("{DSPACE}{term}")
    }

    fn owned(node: &RdfNode<'_, '_>, predicate: &str) -> Option<String> {
        node.iri_or_literal(predicate).map(str::to_string)
    }

    /// The node's `@type` back as the compact DSP term, the shape entities store.
    fn term_of(node: &RdfNode<'_, '_>) -> Option<String> {
        let iri = node.types().next()?;
        Some(iri.strip_prefix(DSPACE).unwrap_or(iri).to_string())
    }

    fn missing(field: &str) -> Errors {
        Errors::format(
            BadFormat::Received,
            format!("expanded message is missing {field}"),
            None,
        )
    }
}
