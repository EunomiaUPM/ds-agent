/*
 *
 *  * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
 *  *
 *  * This program is free software: you can redistribute it and/or modify
 *  * it under the terms of the GNU General Public License as published by
 *  * the Free Software Foundation, either version 3 of the License, or
 *  * (at your option) any later version.
 *  *
 *  * This program is distributed in the hope that it will be useful,
 *  * but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  * GNU General Public License for more details.
 *  *
 *  * You should have received a copy of the GNU General Public License
 *  * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 */
use crate::data::entities::transfer_event::{LogLevel, NewTransferEvent};
use crate::data::factory_trait::DataplaneRepoTrait;
use crate::entities::dataplane_transfers::DataplaneTransfersEntitiesTrait;
use crate::entities::dataplane_transfers::{InteractionMode, TransferState};
use axum::body::{to_bytes, Body};
use axum::extract::{FromRef, Path, Request, State};
use axum::response::{IntoResponse, Response};
use axum::routing::any;
use axum::Router;
use hyper::Method;
use common::utils::get_urn_from_string;
use reqwest::Response as ReqwestResponse;
use reqwest::{Client, StatusCode};
use serde_json::json;
use std::sync::Arc;
use tracing::{error, info};

#[derive(Clone)]
pub struct TestingHTTPProxy {
    client: Client,
    dataplane_service: Arc<dyn DataplaneTransfersEntitiesTrait>,
    repo: Arc<dyn DataplaneRepoTrait>,
}

impl FromRef<TestingHTTPProxy> for Client {
    fn from_ref(input: &TestingHTTPProxy) -> Self {
        input.client.clone()
    }
}

impl FromRef<TestingHTTPProxy> for Arc<dyn DataplaneTransfersEntitiesTrait> {
    fn from_ref(input: &TestingHTTPProxy) -> Self {
        input.dataplane_service.clone()
    }
}

impl TestingHTTPProxy {
    pub fn new(
        dataplane_service: Arc<dyn DataplaneTransfersEntitiesTrait>,
        repo: Arc<dyn DataplaneRepoTrait>,
    ) -> Self {
        let client = reqwest::Client::new();
        Self { client, dataplane_service, repo }
    }
    pub fn router(self) -> Router {
        Router::new()
            .route("/{data_plane_id}", any(Self::forward_request_base))
            .route("/{data_plane_id}/{*path}", any(Self::forward_request_wildcard))
            .with_state(self)
    }

    async fn forward_request_base(
        State(state): State<TestingHTTPProxy>,
        Path(data_plane_id): Path<String>,
        req: Request,
    ) -> impl IntoResponse {
        Self::handle_request(state, data_plane_id, None, req).await
    }

    async fn forward_request_wildcard(
        State(state): State<TestingHTTPProxy>,
        Path((data_plane_id, path)): Path<(String, String)>,
        req: Request,
    ) -> impl IntoResponse {
        Self::handle_request(state, data_plane_id, Some(path), req).await
    }

    async fn handle_request(
        state: TestingHTTPProxy,
        data_plane_id: String,
        path: Option<String>,
        mut req: Request,
    ) -> impl IntoResponse {
        info!("* /data/{} (path: {:?})", data_plane_id, path);
        // validations
        let data_plane_urn = match get_urn_from_string(&data_plane_id) {
            Ok(urn) => urn,
            Err(_) => return (StatusCode::BAD_REQUEST, "data_plane_id not urn").into_response(),
        };

        // PDP: Fetch by Dataplane ID (urn:dataplane-transfer:...)
        let dataplane =
            match state.dataplane_service.get_dataplane_transfer_by_id(&data_plane_urn).await {
                Ok(dataplane) => match dataplane {
                    Some(dataplane) => dataplane,
                    None => {
                        return (StatusCode::NOT_FOUND, "dataplane id not found").into_response()
                    }
                },
                Err(_) => {
                    return (StatusCode::INTERNAL_SERVER_ERROR, "error fetching dataplane")
                        .into_response()
                }
            };

        // STRICT State Check: Only STARTED is allowed
        if dataplane.inner.state != TransferState::Started {
            return (
                StatusCode::FORBIDDEN,
                format!("Transfer is not STARTED (current: {:?})", dataplane.inner.state),
            )
                .into_response();
        }

        // Read egress config from the dataplane process
        use crate::entities::dataplane_manager::config_builder::EgressConfig;
        let egress: EgressConfig =
            match serde_json::from_value(dataplane.inner.egress_config.clone()) {
                Ok(e) => e,
                Err(_) => {
                    return (StatusCode::INTERNAL_SERVER_ERROR, "Invalid or missing egress_config")
                        .into_response()
                }
            };

        let mut next_hop = match &egress {
            EgressConfig::HttpProxy { url } => url.clone(),
            EgressConfig::DataAddress { endpoint, .. } => endpoint.clone(),
            EgressConfig::Connector { .. } => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Connector egress not handled by HTTP proxy",
                )
                    .into_response()
            }
        };

        // Append path if present
        if let Some(p) = &path {
            // Ensure no double slashes or missing slash
            if !next_hop.ends_with('/') {
                next_hop.push('/');
            }
            next_hop.push_str(&p);
        }

        let body = std::mem::take(req.body_mut());
        let body_bytes = match to_bytes(body, 2024 * 1024) // MAX_BUFFER 2MB?
            .await
        {
            Ok(body_bytes) => body_bytes,
            Err(_) => return (StatusCode::BAD_REQUEST, "body too big").into_response(),
        };
        let method = match Method::try_from(req.method()) {
            Ok(method) => method,
            Err(_) => return (StatusCode::BAD_REQUEST, "method not allowed").into_response(),
        };
        let res =
            state.client.request(method.clone(), next_hop.clone()).body(body_bytes).send().await;

        // Enhance Logging
        let role = dataplane.inner.role;
        let mode = dataplane.inner.interaction_mode;

        let ingress_type = match serde_json::from_value::<
            crate::entities::dataplane_manager::config_builder::IngressConfig,
        >(dataplane.inner.ingress_config.clone())
        {
            Ok(
                crate::entities::dataplane_manager::config_builder::IngressConfig::HttpListener {
                    ..
                },
            ) => "HttpListener",
            Ok(crate::entities::dataplane_manager::config_builder::IngressConfig::Connector {
                ..
            }) => "Connector",
            Err(_) => "Unknown",
        };

        let egress_type = match &egress {
            EgressConfig::HttpProxy { .. } => "HttpProxy",
            EgressConfig::DataAddress { .. } => "DataAddress",
            EgressConfig::Connector { .. } => "Connector",
        };

        // Log Transfer Event
        let event = NewTransferEvent {
            transfer_id: dataplane.inner.id.clone(),
            level: LogLevel::Info,
            component: "DataProxy".to_string(),
            message: format!(
                "Transfer [{}|{}] from {} to {} | Ingress: {} | Egress: {}",
                role, mode, data_plane_id, next_hop, ingress_type, egress_type
            ),
            data: Some(json!({
                "role": role,
                "mode": mode,
                "ingress": ingress_type,
                "egress": egress_type,
                "origin_path": path,
                "method": method.to_string(),
                "target": next_hop,
                "status": res.as_ref().map(|r| r.status().as_u16()).unwrap_or(0),
            })),
        };

        // Fire and forget logging (or await/warn)
        if let Err(e) = state.repo.get_transfer_events_repo().create_transfer_event(&event).await {
            error!("Failed to log transfer event: {:?}", e);
        }

        // forward request upstream
        match res {
            Ok(res) => Self::forward_response_helper(res),
            Err(_) => return (StatusCode::BAD_GATEWAY, "peer connection problem").into_response(),
        }
    }

    pub fn forward_response_helper(reqwest_response: ReqwestResponse) -> Response {
        let status = reqwest_response.status();
        let headers = reqwest_response.headers().clone();
        let body_stream = reqwest_response.bytes_stream();
        let body = Body::from_stream(body_stream);
        let mut response = Response::builder().status(status);
        let response_headers = response.headers_mut().unwrap();
        for (key, value) in headers.iter() {
            response_headers.insert(key, value.clone());
        }

        response.body(body).unwrap()
    }
}
