/*
 *
 *  * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
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
use crate::entities::dataplane_drivers::proxy::http as http_proxy;
use crate::entities::dataplane_manager::dataplane_proxy::{
    DataplaneProxyEgress, DataplaneProxyIngress,
};
use crate::entities::dataplane_manager::dataplane_runtime::{
    DataplaneRuntime, ResolvedAuthCredentials, RuntimeSecretVault,
};
use crate::entities::dataplane_transfers::DataplaneTransfersEntitiesTrait;
use crate::entities::dataplane_transfers::{InteractionMode, TransferState};
use axum::body::{to_bytes, Body};
use axum::extract::{FromRef, Path, Request, State};
use axum::response::{IntoResponse, Response};
use axum::routing::any;
use axum::{Json, Router};
use common::utils::get_urn_from_string;
use connector::KeystoreLookup;
use hyper::Method;
use reqwest::Response as ReqwestResponse;
use reqwest::{Client, StatusCode};
use serde_json::json;
use std::sync::Arc;
use tracing::{error, info, warn};

#[derive(Clone)]
pub struct TestingHTTPProxy {
    client: Client,
    dataplane_service: Arc<dyn DataplaneTransfersEntitiesTrait>,
    repo: Arc<dyn DataplaneRepoTrait>,
    keystore: Option<Arc<dyn KeystoreLookup>>,
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
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .expect("Fallo al construir el cliente HTTP de reqwest");
        Self {
            client,
            dataplane_service,
            repo,
            keystore: None,
        }
    }

    /// Attaches the keystore used to resolve credentials when proxying.
    pub fn with_keystore(mut self, keystore: Arc<dyn KeystoreLookup>) -> Self {
        self.keystore = Some(keystore);
        self
    }
    pub fn router(self) -> Router {
        Router::new()
            .route("/{data_plane_id}", any(Self::forward_request_base))
            .route(
                "/{data_plane_id}/{*path}",
                any(Self::forward_request_wildcard),
            )
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
        let dataplane = match state
            .dataplane_service
            .get_dataplane_transfer_by_id(&data_plane_urn)
            .await
        {
            Ok(dataplane) => match dataplane {
                Some(dataplane) => dataplane,
                None => return (StatusCode::NOT_FOUND, "dataplane id not found").into_response(),
            },
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "error fetching dataplane",
                )
                    .into_response()
            }
        };

        // STRICT State Check: Only STARTED is allowed
        if dataplane.inner.state != TransferState::Started {
            return (
                StatusCode::FORBIDDEN,
                format!(
                    "Transfer is not STARTED (current: {:?})",
                    dataplane.inner.state
                ),
            )
                .into_response();
        }

        // Read egress config from the dataplane process
        let egress: DataplaneProxyEgress =
            match serde_json::from_value(dataplane.inner.egress_config.clone()) {
                Ok(e) => e,
                Err(_) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Invalid or missing egress_config",
                    )
                        .into_response()
                }
            };

        let mut next_hop = match &egress {
            DataplaneProxyEgress::HttpProxy {
                path,
                token,
                token_type,
            } => path.clone(),
            _ => {
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

        // Append query
        if let Some(query) = req.uri().query() {
            let sep = if next_hop.contains('?') { '&' } else { '?' };
            next_hop.push(sep);
            next_hop.push_str(query);
        }

        // Resolve auth credentials for the outbound (egress) request.
        //
        // Provider-side proxy: credentials come from flow_control.auth (resolved from the
        // connector's SecretString during set_auth). OAuth2 tokens are vaulted — resolve
        // placeholders via the secret store before use.
        // Consumer-side proxy: the runtime has NoAuth (no connector); outbound credentials
        // come from the egress token set from the provider's DataplaneAddress.
        let flow_control_runtime = dataplane
            .inner
            .flow_control
            .as_ref()
            .and_then(|v| serde_json::from_value::<DataplaneRuntime>(v.clone()).ok());

        let flow_control_runtime = match (flow_control_runtime, &state.keystore) {
            (Some(rt), Some(lookup)) => {
                Some(RuntimeSecretVault::resolve_with_lookup(rt, lookup).await)
            }
            (rt, _) => rt,
        };

        let flow_control_auth = flow_control_runtime
            .map(|rt| rt.auth)
            .unwrap_or(ResolvedAuthCredentials::NoAuth);

        let credentials = match flow_control_auth {
            ResolvedAuthCredentials::NoAuth => match &egress {
                DataplaneProxyEgress::HttpProxy {
                    token: Some(t), ..
                } if !t.is_empty() => ResolvedAuthCredentials::BearerToken { token: t.clone() },
                _ => ResolvedAuthCredentials::NoAuth,
            },
            other => other,
        };

        let incoming_headers = req.headers().clone();
        let body = std::mem::take(req.body_mut());
        let body_bytes = match to_bytes(body, 2024 * 1024) // MAX_BUFFER 2MB
            .await
        {
            Ok(body_bytes) => body_bytes,
            Err(_) => return (StatusCode::BAD_REQUEST, "body too big").into_response(),
        };
        let method = match Method::try_from(req.method()) {
            Ok(method) => method,
            Err(_) => return (StatusCode::BAD_REQUEST, "method not allowed").into_response(),
        };

        // Log the auth header that will be sent upstream.
        let auth_header_preview = match &credentials {
            ResolvedAuthCredentials::NoAuth => "Authorization: <none>".to_string(),
            ResolvedAuthCredentials::BearerToken { token } =>
                format!("Authorization: Bearer {}", &token[..token.len().min(40)]),
            ResolvedAuthCredentials::ApiKey { key, value, .. } =>
                format!("{}: {}...", key, &value[..value.len().min(20)]),
            ResolvedAuthCredentials::BasicAuth { username, .. } =>
                format!("Authorization: Basic <{}:***>", username),
            ResolvedAuthCredentials::OAuth2 { access_token, token_type, .. } =>
                format!("Authorization: {} {}", token_type, &access_token[..access_token.len().min(40)]),
        };
        // logged at warn so it's visible regardless of log level (debug for prod)
        warn!("Proxy {} {} | keystore={} | {}", method, next_hop,
            state.keystore.is_some(), auth_header_preview);

        // Delegate to the proxy driver which injects auth headers.
        // Drop the incoming Authorization header — the proxy uses its own outbound credentials
        // and adding both would produce duplicate Authorization values on the upstream request.
        let mut filtered_headers = incoming_headers.clone();
        filtered_headers.remove("authorization");
        let base_url = next_hop.clone();
        let res = http_proxy::forward(
            &state.client,
            method.clone(),
            &base_url,
            None, // path already appended to next_hop above
            &filtered_headers,
            body_bytes,
            &credentials,
        )
        .await;

        // Enhance Logging
        let role = dataplane.inner.role;
        let mode = dataplane.inner.interaction_mode;

        let ingress_type = match serde_json::from_value::<DataplaneProxyIngress>(
            dataplane.inner.ingress_config.clone(),
        ) {
            Ok(DataplaneProxyIngress::HttpListener { .. }) => "HttpListener",
            Ok(DataplaneProxyIngress::NoOp { .. }) => "NoOp",
            Err(_) => "Unknown",
        };

        let egress_type = match &egress {
            DataplaneProxyEgress::HttpProxy { .. } => "HttpProxy",
            _ => "Unknown",
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
        if let Err(e) = state
            .repo
            .get_transfer_events_repo()
            .create_transfer_event(&event)
            .await
        {
            error!("Failed to log transfer event: {:?}", e);
        }

        // Forward response — hide upstream error details from the consumer but log them.
        match res {
            Ok(res) if res.status().is_client_error() || res.status().is_server_error() => {
                let status = res.status();
                let body_text = res
                    .text()
                    .await
                    .unwrap_or_else(|_| "<unreadable>".to_string());
                error!(
                    "Upstream {} {} -> {} | sent=[{}] keystore={} | body: {}",
                    method, next_hop, status, auth_header_preview,
                    state.keystore.is_some(), body_text
                );
                (status, Json(json!({"error": "upstream error"}))).into_response()
            }
            Ok(res) => Self::forward_response_helper(res),
            Err(e) => {
                error!("Proxy forward failed {} {}: {:?}", method, next_hop, e);
                (
                    StatusCode::BAD_GATEWAY,
                    Json(json!({"error": "proxy error"})),
                )
                    .into_response()
            }
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
