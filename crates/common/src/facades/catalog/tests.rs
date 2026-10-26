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

//! The remote catalog facade against a stub catalog API serving documents as the agent answers.

use std::str::FromStr;

use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{json, Value};
use tokio::net::TcpListener;
use urn::Urn;
use ymir::errors::Errors;
use ymir::types::oauth::UserInfo;

use crate::config::types::min_known_config::MinKnownConfig;
use crate::facades::catalog::remote::CatalogRemoteFacade;
use crate::facades::catalog::CatalogFacadeTrait;

const CATALOG: &str = "urn:catalog:49570cc3-600c-4e1c-9d60-06896f2293df";
const DATASET: &str = "urn:dataset:eba5144b-b684-4c63-b7e0-45f13e3fbc3e";
const DISTRIBUTION: &str = "urn:distribution:cc62a869-b5aa-4c02-900a-5cbcde4b0589";
const SERVICE: &str = "urn:data-service:3714c38d-16ed-42fe-a2f0-b5e3e622434c";
const OFFER: &str = "urn:odrl-policy:563de2b1-bbfe-4ead-843b-d734d457c351";
const CONTEXT: &str = "https://w3id.org/eunomia/v1/catalog-context.jsonld";

fn dataset() -> Value {
    json!({
        "@context": CONTEXT, "@id": DATASET, "@type": "Dataset", "identifier": DATASET,
        "inCatalog": CATALOG, "issued": "2026-10-26T18:49:40.480Z", "title": {"en": "Weather"}
    })
}

fn distribution() -> Value {
    json!({
        "@context": CONTEXT, "@id": DISTRIBUTION, "@type": "Distribution",
        "accessService": [SERVICE], "format": "http+pull", "identifier": DISTRIBUTION,
        "issued": "2026-10-26T18:49:40.505Z", "ofDataset": DATASET, "title": {"en": "Weather API"}
    })
}

fn offer() -> Value {
    json!({
        "id": OFFER, "userId": "system", "userRole": "/admin", "visibility": "Private",
        "odrlOffer": {"permission": [{"action": "use"}]}, "entity": DATASET, "entityType": "Dataset",
        "createdAt": "2026-10-26T18:49:40.544939Z", "sourceTemplateId": null,
        "sourceTemplateVersion": null, "instantiationParameters": null, "description": null
    })
}

/// A facade over a stub API that knows one dataset, its distribution and an offer.
async fn facade() -> CatalogRemoteFacade {
    let base = "/api/v1/catalog-agent";
    let missing = || async {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"message": "Missing Resource Error"})),
        )
    };
    let app = Router::new()
        .route(
            &format!("{base}/datasets/{DATASET}"),
            get(|| async { Json(dataset()) }),
        )
        .route(
            &format!("{base}/distributions/{DISTRIBUTION}"),
            get(|| async { Json(distribution()) }),
        )
        .route(
            &format!("{base}/distributions/dataset/{DATASET}/format/http+pull"),
            get(|| async { Json(distribution()) }),
        )
        .route(
            &format!("{base}/odrl-policies/{OFFER}"),
            get(|| async { Json(offer()) }),
        )
        .fallback(missing);
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port().to_string();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let config: MinKnownConfig = serde_json::from_value(json!({
        "hosts": {
            "http": {"protocol": "http", "url": "127.0.0.1", "port": port, "internal_port": port},
            "grpc": null, "graphql": null
        },
        "api_version": "v1"
    }))
    .unwrap();
    CatalogRemoteFacade::new(&config)
}

fn urn(id: &str) -> Urn {
    Urn::from_str(id).unwrap()
}

/// A dataset is read through RDF: its catalog comes from the reverse `inCatalog` link.
#[tokio::test]
async fn dataset_names_its_catalog_and_title() {
    let found = facade().await.get_dataset(&urn(DATASET)).await.unwrap();
    assert_eq!(found.id, DATASET);
    assert_eq!(found.catalog_id, CATALOG);
    assert_eq!(found.dct_title.as_deref(), Some("Weather"));
}

/// A distribution carries its dataset, `dct:format` and data service.
#[tokio::test]
async fn distribution_carries_dataset_format_and_service() {
    let remote = facade().await;
    let by_id = remote
        .get_distribution(&UserInfo::system(), &urn(DISTRIBUTION))
        .await
        .unwrap();
    let by_format = remote
        .get_distribution_by_format(&urn(DATASET), "http+pull")
        .await
        .unwrap();
    assert_eq!(by_id, by_format);
    assert_eq!(by_id.dataset_id, DATASET);
    assert_eq!(by_id.dct_format.as_deref(), Some("http+pull"));
    assert_eq!(by_id.dcat_access_service, SERVICE);
}

/// An offer keeps its entity and its ODRL content.
#[tokio::test]
async fn offer_keeps_entity_and_odrl() {
    let found = facade().await.get_offer(&urn(OFFER)).await.unwrap();
    assert_eq!(found.entity, DATASET);
    assert_eq!(found.entity_type, "Dataset");
    assert_eq!(found.odrl_offer["permission"][0]["action"], "use");
}

/// A 404 of the catalog API becomes the missing-resource error the local facade returns.
#[tokio::test]
async fn missing_records_are_missing_resources() {
    let remote = facade().await;
    let other = urn("urn:dataset:00000000-0000-0000-0000-000000000000");
    let dataset = remote.get_dataset(&other).await;
    let distribution = remote
        .get_distribution_by_format(&urn(DATASET), "http+push")
        .await;
    assert!(
        matches!(dataset, Err(Errors::MissingResourceError { .. })),
        "{dataset:?}"
    );
    assert!(
        matches!(distribution, Err(Errors::MissingResourceError { .. })),
        "{distribution:?}"
    );
}
