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

//! Catalog over DSP: a consumer asks the provider's catalog through its own RPC endpoint.

use common::eunomia::EUNOMIA_CATALOG_CONTEXT_URL;
use serde_json::json;

use crate::support::dataspace::{Dataspace, PROVIDER_DID};

/// A fresh provider has a main catalog and data service, and the consumer reads them over DSP.
#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn consumer_reads_the_provider_catalog() {
    let ds = Dataspace::start().await;

    let main = ds.provider.get("/api/v1/catalog-agent/catalogs/main").await;
    assert!(main["@id"].is_string(), "{main}");
    assert_eq!(main["@type"], "Catalog", "{main}");

    let found = ds
        .consumer
        .post(
            "/dsp/current/catalog/rpc/setup-catalog-request",
            json!({
                "@context": "https://w3id.org/eunomia/v1/context.jsonld",
                "@type": "RpcCatalogRequestMessage",
                "associatedAgentPeer": PROVIDER_DID,
                "noCache": true
            }),
        )
        .await;
    assert!(found["response"]["service"][0]["endpointURL"].is_string(), "{found}");
    assert_eq!(found["@type"], "RpcCatalogResult", "{found}");
}

/// A published dataset reaches the consumer over DSP, and the provider's doctor and public
/// DCAT-AP route see the same resource.
#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn published_dataset_is_seen_by_every_profile() {
    let ds = Dataspace::start().await;
    let published = crate::negotiation::publish_offer(&ds.provider).await;
    let distribution = ds
        .provider
        .get(&format!("/api/v1/catalog-agent/distributions/{}", published.distribution_id.as_str().unwrap()))
        .await;
    let dataset = distribution["ofDataset"].as_str().unwrap().to_string();

    let report = ds
        .provider
        .get(&format!("/api/v1/catalog-agent/datasets/{dataset}/doctor"))
        .await;
    let dsp = report["profiles"].as_array().unwrap().iter().find(|p| p["profile"] == "dsp").unwrap();
    assert_eq!(dsp["compliant"], true, "{report}");

    let catalog = ds.provider.get("/api/v1/catalog-agent/catalogs/main").await;
    let open = ds
        .provider
        .post_with(
            "/api/v1/catalog-agent/datasets",
            json!({
                "@context": EUNOMIA_CATALOG_CONTEXT_URL,
                "@type": "Dataset",
                "title": {"en": "Open data"},
                "inCatalog": catalog["@id"]
            }),
            &[("eunomia-visibility", "Public")],
        )
        .await;
    let open_id = open["@id"].as_str().unwrap();
    let public = ds
        .provider
        .get_unauthenticated(&format!("/api/v1/catalog-agent/public/dcat-ap/resources/{open_id}"))
        .await;
    assert_eq!(public["@graph"][0]["@id"], open_id, "{public}");

    let found = ds
        .consumer
        .post(
            "/dsp/current/catalog/rpc/setup-dataset-request",
            json!({
                "@context": "https://w3id.org/eunomia/v1/context.jsonld",
                "@type": "RpcDatasetRequestMessage",
                "associatedAgentPeer": PROVIDER_DID,
                "dataset": dataset
            }),
        )
        .await;
    assert_eq!(found["response"]["distribution"][0]["format"], "http+pull", "{found}");
}
