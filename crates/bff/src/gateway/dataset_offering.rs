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

use std::str::FromStr;
use std::sync::Arc;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use common::config::services::traits::GatewayConfigTrait;
use common::config::types::traits::MinKnownConfigTrait;
use events::bus::envelope::{EventEnvelope, Topic};
use events::bus::EventBusTrait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{error, info};
use uuid::Uuid;
use ymir::config::types::HostType;

use crate::setup::context::AppContext;

/// Request payload for creating a complete dataset offering in the catalog.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDatasetOfferingRequest {
    pub dataset: DatasetOfferingInput,
    pub distribution: DistributionOfferingInput,
    pub connector: Option<ConnectorOfferingInput>,
    pub policy: Option<PolicyOfferingInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasetOfferingInput {
    pub title: String,
    pub description: Option<String>,
    pub conforms_to: Option<String>,
    pub creator: Option<String>,
    pub catalog_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DistributionOfferingInput {
    pub title: String,
    pub description: Option<String>,
    pub formats: Option<String>,
    pub access_service_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectorOfferingInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub endpoint: String,
    pub protocol: Option<String>,
    pub method: Option<String>,
    pub auth: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PolicyOfferingInput {
    pub description: Option<String>,
    pub action: Option<String>,
    pub profile: Option<String>,
    pub constraints: Option<Vec<Value>>,
}

/// Orchestrates the end-to-end creation of a dataset offering in the main catalog,
/// including its dataset record, distribution, connector instance, and ODRL policy,
/// and emits a domain event to the internal event bus.
pub async fn orchestrate_dataset_offering(
    ctx: Arc<AppContext>,
    req: CreateDatasetOfferingRequest,
) -> Response {
    let client = Client::new();
    let catalog_base = ctx.config.catalog().get_host(HostType::Http);
    let catalog_base = catalog_base.trim_end_matches('/');

    // 1. Resolve catalog_id (if not provided, auto-discover main catalog)
    let catalog_id = match req.dataset.catalog_id {
        Some(ref cid) if !cid.trim().is_empty() => cid.trim().to_string(),
        _ => {
            let main_cat_url = format!("{catalog_base}/api/v1/catalog-agent/catalogs/main");
            match client.get(&main_cat_url).send().await {
                Ok(resp) if resp.status().is_success() => {
                    let cat_val = resp.json::<Value>().await.unwrap_or_default();
                    cat_val["id"].as_str().map(String::from).unwrap_or_default()
                }
                _ => {
                    let list_url = format!("{catalog_base}/api/v1/catalog-agent/catalogs");
                    match client.get(&list_url).send().await {
                        Ok(resp) if resp.status().is_success() => {
                            let list_val = resp.json::<Value>().await.unwrap_or_default();
                            list_val
                                .as_array()
                                .and_then(|arr| arr.first())
                                .and_then(|c| c["id"].as_str())
                                .map(String::from)
                                .unwrap_or_default()
                        }
                        _ => String::new(),
                    }
                }
            }
        }
    };

    if catalog_id.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Unable to resolve target catalog ID" })),
        )
            .into_response();
    }

    // 2. Resolve access_service_id (if not provided, auto-discover main dataservice)
    let access_service_id = match req.distribution.access_service_id {
        Some(ref ds_id) if !ds_id.trim().is_empty() => ds_id.trim().to_string(),
        _ => {
            let main_ds_url = format!("{catalog_base}/api/v1/catalog-agent/data-services/main");
            match client.get(&main_ds_url).send().await {
                Ok(resp) if resp.status().is_success() => {
                    let ds_val = resp.json::<Value>().await.unwrap_or_default();
                    ds_val["id"].as_str().map(String::from).unwrap_or_default()
                }
                _ => {
                    let cat_ds_url = format!(
                        "{catalog_base}/api/v1/catalog-agent/data-services/catalog/{catalog_id}"
                    );
                    match client.get(&cat_ds_url).send().await {
                        Ok(resp) if resp.status().is_success() => {
                            let list_val = resp.json::<Value>().await.unwrap_or_default();
                            list_val
                                .as_array()
                                .and_then(|arr| arr.first())
                                .and_then(|ds| ds["id"].as_str())
                                .map(String::from)
                                .unwrap_or_default()
                        }
                        _ => format!("urn:uuid:{}", Uuid::new_v4()),
                    }
                }
            }
        }
    };

    // 3. Create Dataset: POST /api/v1/catalog-agent/datasets
    let dataset_body = json!({
        "dctTitle": req.dataset.title,
        "dctDescription": req.dataset.description.unwrap_or_else(|| format!("Dataset for {}", req.dataset.title)),
        "dctConformsTo": req.dataset.conforms_to.unwrap_or_else(|| "https://w3id.org/dspace/v0.8/dcat".to_string()),
        "dctCreator": req.dataset.creator,
        "catalogId": catalog_id
    });

    let create_dataset_url = format!("{catalog_base}/api/v1/catalog-agent/datasets");
    let dataset_resp = match client
        .post(&create_dataset_url)
        .json(&dataset_body)
        .send()
        .await
    {
        Ok(res) if res.status().is_success() => res.json::<Value>().await.unwrap_or_default(),
        Ok(res) => {
            let err_text = res.text().await.unwrap_or_default();
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({ "error": format!("Failed to create dataset in catalog agent: {err_text}") })),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({ "error": format!("Failed connecting to catalog agent: {e}") })),
            )
                .into_response();
        }
    };

    let dataset_id = match dataset_resp["id"].as_str() {
        Some(id) => id.to_string(),
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Created dataset did not return an identifier" })),
            )
                .into_response();
        }
    };

    // 4. Create Distribution: POST /api/v1/catalog-agent/distributions
    let distribution_body = json!({
        "dctTitle": req.distribution.title,
        "dctDescription": req.distribution.description.unwrap_or_else(|| format!("Distribution for {}", req.distribution.title)),
        "dctFormats": req.distribution.formats.unwrap_or_else(|| "application/json".to_string()),
        "dcatAccessService": access_service_id,
        "datasetId": dataset_id
    });

    let create_dist_url = format!("{catalog_base}/api/v1/catalog-agent/distributions");
    let dist_resp = match client
        .post(&create_dist_url)
        .json(&distribution_body)
        .send()
        .await
    {
        Ok(res) if res.status().is_success() => res.json::<Value>().await.unwrap_or_default(),
        Ok(res) => {
            let err_text = res.text().await.unwrap_or_default();
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({
                    "error": format!("Dataset created but failed to create distribution: {err_text}"),
                    "dataset": dataset_resp
                })),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({
                    "error": format!("Dataset created but distribution request failed: {e}"),
                    "dataset": dataset_resp
                })),
            )
                .into_response();
        }
    };

    let distribution_id = dist_resp["id"].as_str().unwrap_or_default().to_string();

    // 5. Create Connector Instance (if connector config provided)
    let connector_resp = if let Some(conn_input) = req.connector {
        if !distribution_id.is_empty() {
            let conn_name = conn_input
                .name
                .unwrap_or_else(|| format!("{}-connector", req.dataset.title));
            let conn_protocol = conn_input.protocol.unwrap_or_else(|| "HTTP".to_string());
            let conn_auth = conn_input
                .auth
                .unwrap_or_else(|| json!({ "type": "NO_AUTH" }));

            let conn_body = json!({
                "id": format!("urn:uuid:{}", Uuid::new_v4()),
                "name": conn_name,
                "description": conn_input.description.unwrap_or_else(|| format!("Connector for {conn_name}")),
                "version": "1.0.0",
                "distributionId": distribution_id,
                "authenticationConfig": conn_auth,
                "interaction": {
                    "mode": "PULL",
                    "dataAccess": {
                        "protocol": conn_protocol,
                        "accessUrl": conn_input.endpoint
                    }
                }
            });

            let create_conn_url = format!("{catalog_base}/api/v1/connector/instances");
            match client.post(&create_conn_url).json(&conn_body).send().await {
                Ok(res) if res.status().is_success() => Some(conn_body),
                Ok(res) => {
                    let err = res.text().await.unwrap_or_default();
                    error!("Connector instance creation returned error: {err}");
                    None
                }
                Err(e) => {
                    error!("Failed to connect to connector endpoint: {e}");
                    None
                }
            }
        } else {
            None
        }
    } else {
        None
    };

    // 6. Create ODRL Policy (if policy config provided)
    let policy_resp = if let Some(pol_input) = req.policy {
        let action = pol_input
            .action
            .unwrap_or_else(|| "http://www.w3.org/ns/odrl/2/use".to_string());
        let profile = pol_input
            .profile
            .unwrap_or_else(|| "http://www.w3.org/ns/odrl/2/".to_string());
        let constraints = pol_input.constraints.unwrap_or_default();

        let policy_body = json!({
            "entityId": dataset_id,
            "entityType": "Dataset",
            "description": pol_input.description.unwrap_or_else(|| "Dataset Usage Policy".to_string()),
            "odrlOffer": {
                "profile": [profile],
                "permission": [
                    {
                        "action": action,
                        "constraint": constraints
                    }
                ]
            }
        });

        let create_policy_url = format!("{catalog_base}/api/v1/catalog-agent/odrl-policies");
        match client
            .post(&create_policy_url)
            .json(&policy_body)
            .send()
            .await
        {
            Ok(res) if res.status().is_success() => res.json::<Value>().await.ok(),
            Ok(res) => {
                let err = res.text().await.unwrap_or_default();
                error!("ODRL policy creation returned error: {err}");
                None
            }
            Err(e) => {
                error!("Failed to connect to odrl-policies endpoint: {e}");
                None
            }
        }
    } else {
        None
    };

    // 7. Publish Event to EventBus
    if let Some(bus) = &ctx.event_bus {
        if let Ok(topic) = Topic::new("catalog.dataset.created") {
            let correlation_id = urn::Urn::from_str(&dataset_id).ok();
            let envelope = EventEnvelope::new(
                topic,
                "bff.catalog-orchestrator",
                1,
                correlation_id,
                json!({
                    "dataset": dataset_resp,
                    "distribution": dist_resp,
                    "connector": connector_resp,
                    "policy": policy_resp,
                    "catalogId": catalog_id,
                }),
            );
            let _ = bus.publish(envelope).await;
            info!("Published catalog.dataset.created event for dataset {dataset_id}");
        }
    }

    // 8. Return 201 Created with full offering bundle
    (
        StatusCode::CREATED,
        Json(json!({
            "dataset": dataset_resp,
            "distribution": dist_resp,
            "connector": connector_resp,
            "policy": policy_resp
        })),
    )
        .into_response()
}
