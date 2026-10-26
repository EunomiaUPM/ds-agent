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

//! DspCatalog and EunomiaCatalog read bodies through RDF, whatever aliasing they use.

use catalog_agent::protocols::dsp::entities::message_types::CatalogDspMessageType;
use catalog_agent::protocols::dsp::entities::rdf_extractor_dsp::DspCatalog;
use catalog_agent::protocols::dsp::entities::rdf_extractor_rpc::EunomiaCatalog;
use common::eunomia::EUNOMIA_CONTEXT_URL;
use common::rdf::RdfMessage;
use serde_json::json;

/// A catalog request reads the same with the DSP terms or with prefixed keys, as the TCK sends.
#[tokio::test]
async fn catalog_request_reads_with_terms_or_prefixes() {
    let plain = json!({
        "@context": ["https://w3id.org/dspace/2025/1/context.jsonld"],
        "@type": "CatalogRequestMessage",
        "filter": [{"keyword": "energy"}]
    });
    let prefixed = json!({
        "@context": ["https://w3id.org/dspace/2025/1/context.jsonld"],
        "@type": "dspace:CatalogRequestMessage"
    });

    let plain = RdfMessage::<DspCatalog>::read_as(&CatalogDspMessageType::CatalogRequestMessage, plain).await.unwrap();
    let prefixed =
        RdfMessage::<DspCatalog>::read_as(&CatalogDspMessageType::CatalogRequestMessage, prefixed).await.unwrap();

    assert_eq!(plain.fields.filter.len(), 1);
    assert!(prefixed.fields.filter.is_empty());
}

/// A body of another message type is refused for this route.
#[tokio::test]
async fn other_message_types_are_refused() {
    let request = json!({
        "@context": ["https://w3id.org/dspace/2025/1/context.jsonld"],
        "@type": "DatasetRequestMessage",
        "dataset": "urn:dataset:1"
    });

    assert!(RdfMessage::<DspCatalog>::read_as(&CatalogDspMessageType::CatalogRequestMessage, request).await.is_err());
}

/// The catalog RPC reads the peer, the dataset and `noCache` from its eunomia terms.
#[tokio::test]
async fn rpc_bodies_read_their_eunomia_terms() {
    let request = json!({
        "@context": EUNOMIA_CONTEXT_URL,
        "@type": "RpcDatasetRequestMessage",
        "associatedAgentPeer": "did:web:provider.example",
        "dataset": "urn:dataset:1",
        "noCache": true
    });

    let read = RdfMessage::<EunomiaCatalog>::read_as(&CatalogDspMessageType::DatasetRequestMessage, request)
        .await
        .unwrap();

    assert_eq!(read.fields.associated_agent_peer.as_deref(), Some("did:web:provider.example"));
    assert_eq!(read.fields.dataset.as_deref(), Some("urn:dataset:1"));
    assert!(read.fields.no_cache);
}
