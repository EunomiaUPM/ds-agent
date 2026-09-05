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
use common::rdf::expanded::Node;
use common::rdf::extract::ExtractProtocolFields;
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::protocols::dsp::entities::message_types::TransferDSPMessageType;
use crate::protocols::dsp::entities::protocol_fields::TransferProtocolFields;

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
    fn message_type(node: &Node<'_, '_>) -> Option<TransferDSPMessageType> {
        node.types()
            .filter_map(|iri| iri.strip_prefix(DSPACE))
            .find_map(|term| term.parse().ok())
    }

    fn extract(node: &Node<'_, '_>) -> Outcome<TransferProtocolFields> {
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
    fn data_address(node: &Node<'_, '_>) -> Outcome<Option<DataAddress>> {
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

    fn endpoint_property(node: &Node<'_, '_>) -> Outcome<EndpointProperty> {
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

    fn owned(node: &Node<'_, '_>, predicate: &str) -> Option<String> {
        node.iri_or_literal(predicate).map(str::to_string)
    }

    /// The node's `@type` back as the compact DSP term, the shape entities store.
    fn term_of(node: &Node<'_, '_>) -> Option<String> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use common::rdf::dsp::DspCanonicalizer;
    use common::rdf::expanded::ExpandedDoc;
    use serde_json::json;

    async fn fields_of(
        message: serde_json::Value,
        kind: TransferDSPMessageType,
    ) -> TransferProtocolFields {
        let expansion = DspCanonicalizer::new(message)
            .expand_once()
            .await
            .expect("expansion");
        let doc = ExpandedDoc::new(&expansion.expanded).expect("array");
        let type_iri = DspTransfer::type_iri(&kind);
        let node = doc.root_message_node(&type_iri, &kind).expect("node");
        DspTransfer::extract(&node).expect("fields")
    }

    fn request(consumer_pid: serde_json::Value) -> serde_json::Value {
        json!({
            "@context": "https://w3id.org/dspace/2025/1/context.jsonld",
            "@type": "TransferRequestMessage",
            "consumerPid": consumer_pid,
            "agreementId": "urn:uuid:ag",
            "callbackAddress": "https://example.com/callback",
            "format": "example:HTTP_PUSH",
            "dataAddress": {
                "@type": "DataAddress",
                "endpointType": "https://w3id.org/idsa/v4.1/HTTP",
                "endpoint": "http://example.com",
                "endpointProperties": [
                    {"@type": "EndpointProperty", "name": "authorization", "value": "TOKEN-ABCDEFG"},
                    {"@type": "EndpointProperty", "name": "authType", "value": "bearer"}
                ]
            }
        })
    }

    #[tokio::test]
    async fn reads_every_request_field() {
        let f = fields_of(
            request(json!("urn:uuid:cc")),
            TransferDSPMessageType::TransferRequestMessage,
        )
        .await;
        assert_eq!(f.consumer_pid.as_deref(), Some("urn:uuid:cc"));
        assert_eq!(f.agreement_id.as_deref(), Some("urn:uuid:ag"));
        assert_eq!(
            f.callback_address.as_deref(),
            Some("https://example.com/callback")
        );
        assert_eq!(
            f.format.as_deref(),
            Some("example:HTTP_PUSH"),
            "format is dct:, not dspace:"
        );
        assert!(f.provider_pid.is_none(), "a request carries no providerPid");

        let a = f.data_address.unwrap();
        assert_eq!(a.endpoint.as_deref(), Some("http://example.com"));
        let mut props: Vec<_> = a
            .endpoint_properties
            .iter()
            .map(|p| (p.name.clone(), p.value.clone()))
            .collect();
        props.sort();
        assert_eq!(props[0].0, "authType");
    }

    /// `{"@id": …}` is the same message as the bare string and hashes alike, so it
    /// must read alike.
    #[tokio::test]
    async fn a_pid_reads_the_same_as_an_id_or_a_string() {
        let as_id = fields_of(
            request(json!({"@id": "urn:uuid:cc"})),
            TransferDSPMessageType::TransferRequestMessage,
        )
        .await;
        assert_eq!(as_id.consumer_pid.as_deref(), Some("urn:uuid:cc"));
    }

    /// `code` and `reason` are only in scope for suspension and termination, and
    /// `reason` is `@container: @set`.
    #[tokio::test]
    async fn termination_reads_code_and_every_reason() {
        let f = fields_of(
            json!({
                "@context": "https://w3id.org/dspace/2025/1/context.jsonld",
                "@type": "TransferTerminationMessage",
                "consumerPid": "urn:uuid:cc",
                "providerPid": "urn:uuid:pp",
                "code": "99",
                "reason": ["Policy violation", "Agreement expired"]
            }),
            TransferDSPMessageType::TransferTerminationMessage,
        )
        .await;
        assert_eq!(f.code.as_deref(), Some("99"));
        let mut reasons = f.reason.clone();
        reasons.sort();
        assert_eq!(reasons, vec!["Agreement expired", "Policy violation"]);
    }

    #[tokio::test]
    async fn a_data_address_without_an_endpoint_type_is_malformed() {
        let expansion = DspCanonicalizer::new(json!({
            "@context": "https://w3id.org/dspace/2025/1/context.jsonld",
            "@type": "TransferRequestMessage",
            "dataAddress": {"@type": "DataAddress", "endpoint": "http://example.com"}
        }))
        .expand_once()
        .await
        .unwrap();
        let doc = ExpandedDoc::new(&expansion.expanded).unwrap();
        let kind = TransferDSPMessageType::TransferRequestMessage;
        let type_iri = DspTransfer::type_iri(&kind);
        let node = doc.root_message_node(&type_iri, &kind).unwrap();
        assert!(DspTransfer::extract(&node).is_err());
    }
}
