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
use crate::entities::dataplane_transfers::DataplaneTransferDto;
use axum::body::{to_bytes, Body, Bytes};
use axum::extract::{FromRef, Path, Request, State};
use axum::http::HeaderMap;
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
use std::time::Duration;
use tracing::{error, info, warn};
use urn::Urn;
use ymir::errors::Outcome;

/// Maximum request body we buffer before forwarding upstream (2 MiB).
const MAX_BODY_BYTES: usize = 2 * 1024 * 1024;
/// Cap on establishing the upstream TCP connection.
const UPSTREAM_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// Cap on the whole upstream request/response exchange.
const UPSTREAM_TIMEOUT: Duration = Duration::from_secs(30);

/// Hop-by-hop headers (RFC 7230 §6.1) that must not be relayed end-to-end.
const HOP_BY_HOP_HEADERS: [&str; 8] = [
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailer",
    "transfer-encoding",
    "upgrade",
];

/// Early-exit conditions while proxying, each mapped to an HTTP status.
enum ProxyError {
    BadDataplaneId,
    DataplaneNotFound,
    DataplaneLookupFailed,
    NotStarted(TransferState),
    InvalidEgressConfig,
    UnsupportedEgress,
    BodyTooLarge,
}

impl IntoResponse for ProxyError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ProxyError::BadDataplaneId => {
                (StatusCode::BAD_REQUEST, "data_plane_id not urn".to_string())
            }
            ProxyError::DataplaneNotFound => {
                (StatusCode::NOT_FOUND, "dataplane id not found".to_string())
            }
            ProxyError::DataplaneLookupFailed => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "error fetching dataplane".to_string(),
            ),
            ProxyError::NotStarted(state) => (
                StatusCode::FORBIDDEN,
                format!("Transfer is not STARTED (current: {state:?})"),
            ),
            ProxyError::InvalidEgressConfig => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Invalid or missing egress_config".to_string(),
            ),
            ProxyError::UnsupportedEgress => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Connector egress not handled by HTTP proxy".to_string(),
            ),
            ProxyError::BodyTooLarge => (StatusCode::BAD_REQUEST, "body too big".to_string()),
        };
        (status, message).into_response()
    }
}

/// The parts of the incoming request we forward upstream.
struct OutboundRequest {
    method: Method,
    headers: HeaderMap,
    body: Bytes,
}

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
        // `danger_accept_invalid_certs` is intentional: this is a TESTING proxy
        // that must reach upstreams using self-signed certificates. Timeouts stop
        // a dead upstream from hanging the proxy connection indefinitely.
        let client = Client::builder()
            .danger_accept_invalid_certs(true)
            .connect_timeout(UPSTREAM_CONNECT_TIMEOUT)
            .timeout(UPSTREAM_TIMEOUT)
            .build()
            .expect("failed to build reqwest HTTP client");
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

    /// Axum entry point: runs the proxy pipeline and turns an early
    /// `ProxyError` into its HTTP response.
    async fn handle_request(
        state: TestingHTTPProxy,
        data_plane_id: String,
        path: Option<String>,
        req: Request,
    ) -> Response {
        match state.proxy(&data_plane_id, path.as_deref(), req).await {
            Ok(response) => response,
            Err(err) => err.into_response(),
        }
    }

    /// The proxy pipeline, top to bottom: validate, authorize, build the
    /// target URL, resolve credentials, then forward and log.
    async fn proxy(
        &self,
        data_plane_id: &str,
        path: Option<&str>,
        req: Request,
    ) -> Result<Response, ProxyError> {
        info!("* proxy /data/{data_plane_id} (path: {path:?})");

        let urn = Self::parse_dataplane_id(data_plane_id)?;
        let dataplane = self.load_started_dataplane(&urn).await?;
        let egress = Self::parse_egress(&dataplane)?;

        // Capture the query string before the body consumes the request.
        let query = req.uri().query().map(str::to_owned);
        let outbound = Self::extract_outbound(req).await?;

        let target = Self::build_target_url(&egress, path, query.as_deref())?;
        let credentials = self.resolve_credentials(&dataplane, &egress).await;

        Ok(self
            .forward_and_log(&dataplane, data_plane_id, path, &egress, &target, outbound, &credentials)
            .await)
    }

    // --- Validation & lookup ---------------------------------------------

    /// Parses the `{data_plane_id}` path segment into a transfer URN.
    fn parse_dataplane_id(raw: &str) -> Result<Urn, ProxyError> {
        get_urn_from_string(&raw.to_string()).map_err(|_| ProxyError::BadDataplaneId)
    }

    /// Loads the transfer and enforces that it is `Started` — the proxy only
    /// relays traffic for active transfers.
    async fn load_started_dataplane(&self, urn: &Urn) -> Result<DataplaneTransferDto, ProxyError> {
        let dataplane = self
            .dataplane_service
            .get_dataplane_transfer_by_id(urn)
            .await
            .map_err(|_| ProxyError::DataplaneLookupFailed)?
            .ok_or(ProxyError::DataplaneNotFound)?;

        if dataplane.inner.state != TransferState::Started {
            return Err(ProxyError::NotStarted(dataplane.inner.state.clone()));
        }
        Ok(dataplane)
    }

    /// Reads the egress configuration the transfer was provisioned with.
    fn parse_egress(dataplane: &DataplaneTransferDto) -> Result<DataplaneProxyEgress, ProxyError> {
        serde_json::from_value(dataplane.inner.egress_config.clone())
            .map_err(|_| ProxyError::InvalidEgressConfig)
    }

    // --- Request shaping --------------------------------------------------

    /// Extracts the method, headers and size-limited body from the request.
    async fn extract_outbound(mut req: Request) -> Result<OutboundRequest, ProxyError> {
        let method = req.method().clone();
        let headers = req.headers().clone();
        let body = std::mem::take(req.body_mut());
        let body = to_bytes(body, MAX_BODY_BYTES)
            .await
            .map_err(|_| ProxyError::BodyTooLarge)?;
        Ok(OutboundRequest { method, headers, body })
    }

    /// Builds the upstream URL: egress base path + optional wildcard path + query.
    fn build_target_url(
        egress: &DataplaneProxyEgress,
        path: Option<&str>,
        query: Option<&str>,
    ) -> Result<String, ProxyError> {
        let DataplaneProxyEgress::HttpProxy { path: base, .. } = egress else {
            return Err(ProxyError::UnsupportedEgress);
        };
        let mut url = base.clone();

        // Join the wildcard segment, guarding against missing/double slashes.
        if let Some(p) = path {
            if !url.ends_with('/') {
                url.push('/');
            }
            url.push_str(p);
        }

        // Preserve the original query string.
        if let Some(q) = query {
            url.push(if url.contains('?') { '&' } else { '?' });
            url.push_str(q);
        }
        Ok(url)
    }

    // --- Credentials ------------------------------------------------------

    /// Determines the credentials to present upstream.
    ///
    /// Provider-side: credentials live in `flow_control.auth` (resolved from the
    /// connector secret during `set_auth`); vaulted OAuth2 tokens are resolved
    /// via the keystore. Consumer-side: there is no connector, so we fall back
    /// to the bearer token carried in the egress config.
    async fn resolve_credentials(
        &self,
        dataplane: &DataplaneTransferDto,
        egress: &DataplaneProxyEgress,
    ) -> ResolvedAuthCredentials {
        let runtime = dataplane
            .inner
            .flow_control
            .as_ref()
            .and_then(|v| serde_json::from_value::<DataplaneRuntime>(v.clone()).ok());

        // Resolve vaulted placeholders when a keystore is configured.
        let runtime = match (runtime, &self.keystore) {
            (Some(rt), Some(lookup)) => {
                Some(RuntimeSecretVault::resolve_with_lookup(rt, lookup).await)
            }
            (rt, _) => rt,
        };

        let auth = runtime
            .map(|rt| rt.auth)
            .unwrap_or(ResolvedAuthCredentials::NoAuth);

        // Fall back to the egress bearer token when the runtime carries no auth.
        match auth {
            ResolvedAuthCredentials::NoAuth => match egress {
                DataplaneProxyEgress::HttpProxy { token: Some(t), .. } if !t.is_empty() => {
                    ResolvedAuthCredentials::BearerToken { token: t.clone() }
                }
                _ => ResolvedAuthCredentials::NoAuth,
            },
            other => other,
        }
    }

    // --- Forwarding & response -------------------------------------------

    /// Forwards the request upstream, records a transfer event, and maps the
    /// upstream response back to the caller.
    #[allow(clippy::too_many_arguments)]
    async fn forward_and_log(
        &self,
        dataplane: &DataplaneTransferDto,
        data_plane_id: &str,
        path: Option<&str>,
        egress: &DataplaneProxyEgress,
        target: &str,
        outbound: OutboundRequest,
        credentials: &ResolvedAuthCredentials,
    ) -> Response {
        let auth_preview = Self::preview_auth(credentials);
        // Logged at warn so it stays visible regardless of the configured level.
        warn!(
            "Proxy {} {} | keystore={} | {}",
            outbound.method,
            target,
            self.keystore.is_some(),
            auth_preview
        );

        // Drop the inbound Authorization header — the proxy injects its own
        // outbound credentials and two would produce duplicate values upstream.
        let mut headers = outbound.headers;
        headers.remove("authorization");

        let response = http_proxy::forward(
            &self.client,
            outbound.method.clone(),
            target,
            None, // the wildcard path is already baked into `target`
            &headers,
            outbound.body,
            credentials,
        )
        .await;

        self.record_event(dataplane, data_plane_id, path, egress, target, &outbound.method, &response)
            .await;

        Self::map_response(response, &outbound.method, target, &auth_preview, self.keystore.is_some())
            .await
    }

    /// Persists a best-effort transfer event describing this proxied request.
    #[allow(clippy::too_many_arguments)]
    async fn record_event(
        &self,
        dataplane: &DataplaneTransferDto,
        data_plane_id: &str,
        path: Option<&str>,
        egress: &DataplaneProxyEgress,
        target: &str,
        method: &Method,
        response: &Outcome<ReqwestResponse>,
    ) {
        let role = dataplane.inner.role.clone();
        let mode = dataplane.inner.interaction_mode.clone();
        let ingress_type = Self::ingress_label(dataplane);
        let egress_type = Self::egress_label(egress);
        let status = response.as_ref().map(|r| r.status().as_u16()).unwrap_or(0);

        let event = NewTransferEvent {
            transfer_id: dataplane.inner.id.clone(),
            level: LogLevel::Info,
            component: "DataProxy".to_string(),
            message: format!(
                "Transfer [{role}|{mode}] from {data_plane_id} to {target} | Ingress: {ingress_type} | Egress: {egress_type}"
            ),
            data: Some(json!({
                "role": role,
                "mode": mode,
                "ingress": ingress_type,
                "egress": egress_type,
                "origin_path": path,
                "method": method.to_string(),
                "target": target,
                "status": status,
            })),
        };

        // Fire-and-forget: a logging failure must not break the data path.
        if let Err(e) = self
            .repo
            .get_transfer_events_repo()
            .create_transfer_event(&event)
            .await
        {
            error!("Failed to log transfer event: {e:?}");
        }
    }

    /// Maps the upstream result to the client response, masking upstream error
    /// bodies (still logged server-side) so internal details are not leaked.
    async fn map_response(
        result: Outcome<ReqwestResponse>,
        method: &Method,
        target: &str,
        auth_preview: &str,
        keystore_present: bool,
    ) -> Response {
        match result {
            Ok(res) if res.status().is_client_error() || res.status().is_server_error() => {
                let status = res.status();
                let body_text = res.text().await.unwrap_or_else(|_| "<unreadable>".to_string());
                error!(
                    "Upstream {} {} -> {} | sent=[{}] keystore={} | body: {}",
                    method, target, status, auth_preview, keystore_present, body_text
                );
                (status, Json(json!({"error": "upstream error"}))).into_response()
            }
            Ok(res) => Self::relay_response(res),
            Err(e) => {
                error!("Proxy forward failed {} {}: {:?}", method, target, e);
                (StatusCode::BAD_GATEWAY, Json(json!({"error": "proxy error"}))).into_response()
            }
        }
    }

    /// Streams the upstream response back to the client, dropping hop-by-hop
    /// headers that must not be relayed end-to-end (RFC 7230 §6.1).
    fn relay_response(upstream: ReqwestResponse) -> Response {
        let status = upstream.status();
        let headers = upstream.headers().clone();
        let body = Body::from_stream(upstream.bytes_stream());

        let mut response = Response::builder().status(status);
        if let Some(out_headers) = response.headers_mut() {
            for (key, value) in headers.iter() {
                if HOP_BY_HOP_HEADERS.contains(&key.as_str().to_ascii_lowercase().as_str()) {
                    continue;
                }
                out_headers.insert(key, value.clone());
            }
        }
        response
            .body(body)
            .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
    }

    // --- Small labels for logging ----------------------------------------

    /// Human-readable label of the configured ingress kind.
    fn ingress_label(dataplane: &DataplaneTransferDto) -> &'static str {
        match serde_json::from_value::<DataplaneProxyIngress>(dataplane.inner.ingress_config.clone()) {
            Ok(DataplaneProxyIngress::HttpListener { .. }) => "HttpListener",
            Ok(DataplaneProxyIngress::NoOp) => "NoOp",
            Err(_) => "Unknown",
        }
    }

    /// Human-readable label of the configured egress kind.
    fn egress_label(egress: &DataplaneProxyEgress) -> &'static str {
        match egress {
            DataplaneProxyEgress::HttpProxy { .. } => "HttpProxy",
            _ => "Unknown",
        }
    }

    /// Renders a truncated, log-safe preview of the outbound Authorization header.
    fn preview_auth(credentials: &ResolvedAuthCredentials) -> String {
        match credentials {
            ResolvedAuthCredentials::NoAuth => "Authorization: <none>".to_string(),
            ResolvedAuthCredentials::BearerToken { token } => {
                format!("Authorization: Bearer {}", &token[..token.len().min(40)])
            }
            ResolvedAuthCredentials::ApiKey { key, value, .. } => {
                format!("{}: {}...", key, &value[..value.len().min(20)])
            }
            ResolvedAuthCredentials::BasicAuth { username, .. } => {
                format!("Authorization: Basic <{username}:***>")
            }
            ResolvedAuthCredentials::OAuth2 { access_token, token_type, .. } => format!(
                "Authorization: {} {}",
                token_type,
                &access_token[..access_token.len().min(40)]
            ),
        }
    }
}
