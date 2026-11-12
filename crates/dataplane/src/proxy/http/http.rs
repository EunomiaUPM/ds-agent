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

//! Production HTTP data proxy (`HttpDataProxy`) and its Axum routes.

use crate::data::factory_trait::DataplaneRepoTrait;
use crate::data::sea_orm::orm::transfer_event::{LogLevel, NewTransferEvent};
use crate::engine::dataplane_drivers::proxy::http::HttpProxyDriver;
use crate::engine::dataplane_manager::dataplane_proxy::{
    DataplaneProxyEgress, DataplaneProxyIngress,
};
use crate::engine::dataplane_manager::dataplane_runtime::{
    DataplaneRuntime, DeliveryPolicy, ResolvedAuthCredentials, RuntimeSecretVault,
};
use crate::entities::dataplane_transfers::{
    DataplaneTransferDto, InteractionMode, TransferRole, TransferState,
};
use crate::proxy::tracker::{DeliveryTracker, ReserveOutcome};
use crate::services::dataplane_transfers::DataplaneTransferServiceTrait;
use crate::signals::{DataplaneSignal, DataplaneSignalSender};
use axum::body::{to_bytes, Body, Bytes};
use axum::extract::{FromRef, Path, Request, State};
use axum::http::header::AUTHORIZATION;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::any;
use axum::{Json, Router};
use common::routes::dataplane as routes;
use connector::{KeystoreLookup, TransferLifetime};
use futures_util::Stream;
use hyper::Method;
use serde_json::json;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;
use tracing::{debug, error, info};
use url::Host;
use urn::Urn;
use ymir::errors::{Errors, Outcome};
use ymir::oauth::OwnedTrait;
use ymir::services::client::ClientService;
use ymir::types::http::Response as UpstreamResponse;
use ymir::types::oauth::UserInfo;
use ymir::utils::get_urn_from_string;

/// Maximum request body we buffer before forwarding upstream (2 MiB).
const MAX_BODY_BYTES: usize = 2 * 1024 * 1024;
/// Cap on establishing the upstream TCP connection.
const UPSTREAM_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// Cap on the whole upstream request/response exchange.
const UPSTREAM_TIMEOUT: Duration = Duration::from_secs(30);
/// Query parameter carrying per-transfer ingress tokens on webhook callbacks.
const INGRESS_TOKEN_QUERY_PARAM: &str = "ingress_token";

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
    MissingIngressAuth,
    InvalidIngressAuth,
    InvalidEgressConfig,
    UnsupportedEgress,
    InvalidTargetUrl,
    ForbiddenTarget(&'static str),
    TargetResolutionFailed,
    BodyTooLarge,
    TransferCompleted,
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
            ProxyError::MissingIngressAuth => (
                StatusCode::UNAUTHORIZED,
                "missing ingress authorization".to_string(),
            ),
            ProxyError::InvalidIngressAuth => (
                StatusCode::FORBIDDEN,
                "invalid ingress authorization".to_string(),
            ),
            ProxyError::InvalidEgressConfig => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Invalid or missing egress_config".to_string(),
            ),
            ProxyError::UnsupportedEgress => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Connector egress not handled by HTTP proxy".to_string(),
            ),
            ProxyError::InvalidTargetUrl => (
                StatusCode::BAD_REQUEST,
                "invalid upstream target URL".to_string(),
            ),
            ProxyError::ForbiddenTarget(reason) => (StatusCode::FORBIDDEN, reason.to_string()),
            ProxyError::TargetResolutionFailed => (
                StatusCode::BAD_GATEWAY,
                "unable to resolve upstream target host".to_string(),
            ),
            ProxyError::BodyTooLarge => (StatusCode::BAD_REQUEST, "body too big".to_string()),
            ProxyError::TransferCompleted => (
                StatusCode::GONE,
                "finite transfer already completed".to_string(),
            ),
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

/// Wraps an upstream response body stream for `(Provider, Pull)` `FINITE` transfers so
/// `DataplaneSignal::Completed` fires once the full payload has been yielded to the HTTP server.
struct SignalOnEofStream<E> {
    inner: Pin<Box<dyn Stream<Item = Result<Bytes, E>> + Send>>,
    dataplane_id: String,
    transfer_process_id: Option<Urn>,
    max_retries: u32,
    expected_bytes: Option<u64>,
    delivered_bytes: u64,
    tracker: Arc<DeliveryTracker>,
    signals: DataplaneSignalSender,
    finished: bool,
}

impl<E> SignalOnEofStream<E> {
    fn complete_once(&mut self) {
        if self.finished {
            return;
        }
        self.finished = true;
        let dp_id = self.dataplane_id.clone();
        let tp_id = self.transfer_process_id.clone();
        if let Some(first) = self.tracker.try_complete_finite_slot_sync(&dp_id) {
            if first {
                if let Some(transfer_process_id) = tp_id {
                    self.signals.emit(DataplaneSignal::Completed {
                        transfer_process_id,
                    });
                }
            }
        } else if let Ok(handle) = tokio::runtime::Handle::try_current() {
            let tracker = self.tracker.clone();
            let signals = self.signals.clone();
            handle.spawn(async move {
                if tracker.complete_finite_slot(&dp_id).await {
                    if let Some(transfer_process_id) = tp_id {
                        signals.emit(DataplaneSignal::Completed {
                            transfer_process_id,
                        });
                    }
                }
            });
        }
    }
}

impl<E> Stream for SignalOnEofStream<E> {
    type Item = Result<Bytes, E>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.inner.as_mut().poll_next(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Some(Ok(chunk))) => {
                self.delivered_bytes = self.delivered_bytes.saturating_add(chunk.len() as u64);
                // Hyper's HTTP/1.1 `Content-Length` encoder stops polling once `Content-Length`
                // bytes have been yielded (without polling an extra time for `None`).
                if self
                    .expected_bytes
                    .is_some_and(|expected| self.delivered_bytes >= expected)
                {
                    self.complete_once();
                }
                Poll::Ready(Some(Ok(chunk)))
            }
            Poll::Ready(Some(Err(err))) => {
                if !self.finished {
                    self.finished = true;
                    let dp_id = self.dataplane_id.clone();
                    let tp_id = self.transfer_process_id.clone();
                    let max_retries = self.max_retries;
                    if let Some(exceeded) =
                        self.tracker.try_record_transient_failure_sync(&dp_id, max_retries)
                    {
                        if exceeded {
                            if let Some(transfer_process_id) = tp_id {
                                self.signals.emit(DataplaneSignal::Failed {
                                    transfer_process_id,
                                    code: "DATAPLANE_STREAM_FAILED".to_string(),
                                    reason: "upstream response stream aborted before EOF"
                                        .to_string(),
                                });
                            }
                        } else {
                            let _ = self.tracker.try_release_finite_slot_sync(&dp_id);
                        }
                    } else if let Ok(handle) = tokio::runtime::Handle::try_current() {
                        let tracker = self.tracker.clone();
                        let signals = self.signals.clone();
                        handle.spawn(async move {
                            if tracker.record_transient_failure(&dp_id, max_retries).await {
                                if let Some(transfer_process_id) = tp_id {
                                    signals.emit(DataplaneSignal::Failed {
                                        transfer_process_id,
                                        code: "DATAPLANE_STREAM_FAILED".to_string(),
                                        reason: "upstream response stream aborted before EOF"
                                            .to_string(),
                                    });
                                }
                            } else {
                                tracker.release_finite_slot(&dp_id).await;
                            }
                        });
                    }
                }
                Poll::Ready(Some(Err(err)))
            }
            Poll::Ready(None) => {
                self.complete_once();
                Poll::Ready(None)
            }
        }
    }
}

impl<E> Drop for SignalOnEofStream<E> {
    fn drop(&mut self) {
        if !self.finished {
            self.finished = true;
            let dp_id = self.dataplane_id.clone();
            if !self.tracker.try_release_finite_slot_sync(&dp_id) {
                if let Ok(handle) = tokio::runtime::Handle::try_current() {
                    let tracker = self.tracker.clone();
                    handle.spawn(async move {
                        tracker.release_finite_slot(&dp_id).await;
                    });
                }
            }
        }
    }
}

/// HTTP data proxy: validates per-transfer ingress capabilities, enforces anti-SSRF
/// egress rules, resolves outbound credentials, and forwards `/{data_plane_id}/...`.
#[derive(Clone)]
pub struct HttpDataProxy {
    client: Arc<ClientService>,
    dataplane_service: Arc<dyn DataplaneTransferServiceTrait>,
    repo: Arc<dyn DataplaneRepoTrait>,
    keystore: Option<Arc<dyn KeystoreLookup>>,
    allow_private_egress: bool,
    signals: DataplaneSignalSender,
    tracker: Arc<DeliveryTracker>,
}

impl FromRef<HttpDataProxy> for Arc<dyn DataplaneTransferServiceTrait> {
    fn from_ref(input: &HttpDataProxy) -> Self {
        input.dataplane_service.clone()
    }
}

impl HttpDataProxy {
    /// Builds an HTTP data proxy with strict TLS certificate verification (`false`
    /// invalid-cert acceptance) and private/loopback egress blocked (`false`) by default.
    pub fn new(
        dataplane_service: Arc<dyn DataplaneTransferServiceTrait>,
        repo: Arc<dyn DataplaneRepoTrait>,
    ) -> Self {
        Self {
            client: Arc::new(Self::build_client(false)),
            dataplane_service,
            repo,
            keystore: None,
            allow_private_egress: false,
            signals: DataplaneSignalSender::noop(),
            tracker: Arc::new(DeliveryTracker::in_memory()),
        }
    }

    fn build_client(accept_invalid_certs: bool) -> ClientService {
        ClientService::builder()
            .accept_invalid_certs(accept_invalid_certs)
            .connect_timeout(UPSTREAM_CONNECT_TIMEOUT)
            .timeout(Some(UPSTREAM_TIMEOUT))
            .build()
    }

    /// Attaches the keystore used to resolve vaulted credentials when proxying.
    pub fn with_keystore(mut self, keystore: Arc<dyn KeystoreLookup>) -> Self {
        self.keystore = Some(keystore);
        self
    }

    /// Configures whether self-signed/invalid TLS certificates are accepted upstream
    /// (`false` by default; enable only in controlled local/dev environments).
    pub fn with_accept_invalid_certs(mut self, accept: bool) -> Self {
        self.client = Arc::new(Self::build_client(accept));
        self
    }

    /// Configures whether private, loopback, or link-local egress targets are allowed
    /// (`false` by default; enable only for local/Docker-internal test topologies).
    pub fn with_allow_private_egress(mut self, allow: bool) -> Self {
        self.allow_private_egress = allow;
        self
    }

    /// Attaches the control-plane signal sender (`Completed` / `Failed`).
    pub fn with_signals(mut self, signals: DataplaneSignalSender) -> Self {
        self.signals = signals;
        self
    }

    /// Attaches a shared `DeliveryTracker` (e.g. Redis-backed across replicas).
    pub fn with_tracker(mut self, tracker: Arc<DeliveryTracker>) -> Self {
        self.tracker = tracker;
        self
    }

    pub fn router(self) -> Router {
        Router::new()
            .route(routes::proxy::BY_DATAPLANE, any(Self::forward_request_base))
            .route(routes::proxy::PATH, any(Self::forward_request_wildcard))
            .with_state(self)
    }

    async fn forward_request_base(
        State(state): State<HttpDataProxy>,
        Path(data_plane_id): Path<String>,
        req: Request,
    ) -> impl IntoResponse {
        Self::handle_request(state, data_plane_id, None, req).await
    }

    async fn forward_request_wildcard(
        State(state): State<HttpDataProxy>,
        Path((data_plane_id, path)): Path<(String, String)>,
        req: Request,
    ) -> impl IntoResponse {
        Self::handle_request(state, data_plane_id, Some(path), req).await
    }

    /// Axum entry point: runs the proxy pipeline and turns an early
    /// `ProxyError` into its HTTP response.
    async fn handle_request(
        state: HttpDataProxy,
        data_plane_id: String,
        path: Option<String>,
        req: Request,
    ) -> Response {
        match state.proxy(&data_plane_id, path.as_deref(), req).await {
            Ok(response) => response,
            Err(err) => err.into_response(),
        }
    }

    /// The proxy pipeline, top to bottom: validate transfer state, enforce per-transfer
    /// ingress auth, reserve `FINITE` single-shot quota when applicable, build and validate
    /// target URL (anti-SSRF), resolve credentials, then forward and log.
    async fn proxy(
        &self,
        data_plane_id: &str,
        path: Option<&str>,
        req: Request,
    ) -> Result<Response, ProxyError> {
        info!("* proxy /data/{data_plane_id} (path: {path:?})");

        let urn = Self::parse_dataplane_id(data_plane_id)?;
        let dataplane = self.load_started_dataplane(&urn).await?;

        // Capture the query string before the body consumes the request.
        let raw_query = req.uri().query().map(str::to_owned);
        Self::verify_ingress_auth(&dataplane, req.headers(), raw_query.as_deref())?;
        let clean_query = Self::strip_ingress_token_query(raw_query.as_deref());

        let policy = Self::delivery_policy(&dataplane);
        let is_provider_finite = dataplane.inner.role == TransferRole::Provider
            && policy.lifetime == TransferLifetime::Finite;

        if is_provider_finite
            && self.tracker.reserve_finite_slot(&dataplane.inner.id).await
                == ReserveOutcome::Exhausted
        {
            return Err(ProxyError::TransferCompleted);
        }

        let egress = match Self::parse_egress(&dataplane) {
            Ok(egress) => egress,
            Err(err) => {
                self.emit_failed(
                    &dataplane,
                    "DATAPLANE_CONFIG_ERROR",
                    "invalid or missing egress_config",
                );
                return Err(err);
            }
        };
        let outbound = match Self::extract_outbound(req).await {
            Ok(outbound) => outbound,
            Err(err) => {
                // Caller payload error: never terminate the transfer, and release any finite slot.
                if is_provider_finite {
                    self.tracker.release_finite_slot(&dataplane.inner.id).await;
                }
                return Err(err);
            }
        };

        let target = match Self::build_target_url(&egress, path, clean_query.as_deref()) {
            Ok(target) => target,
            Err(err) => {
                self.emit_failed(
                    &dataplane,
                    "DATAPLANE_CONFIG_ERROR",
                    "connector egress not handled by HTTP proxy",
                );
                return Err(err);
            }
        };
        if let Err(err) = self.validate_target_url(&target).await {
            match &err {
                ProxyError::InvalidTargetUrl => {
                    self.emit_failed(
                        &dataplane,
                        "DATAPLANE_INVALID_TARGET_URL",
                        "invalid upstream target URL",
                    );
                }
                ProxyError::ForbiddenTarget(reason) => {
                    self.emit_failed(&dataplane, "DATAPLANE_FORBIDDEN_TARGET", *reason);
                }
                ProxyError::TargetResolutionFailed => {
                    if self
                        .tracker
                        .record_transient_failure(&dataplane.inner.id, policy.max_retries)
                        .await
                    {
                        self.emit_failed(
                            &dataplane,
                            "DATAPLANE_MAX_RETRIES_EXCEEDED",
                            "unable to resolve upstream target host",
                        );
                    } else if is_provider_finite {
                        self.tracker.release_finite_slot(&dataplane.inner.id).await;
                    }
                }
                _ => {}
            }
            return Err(err);
        }
        let credentials = self.resolve_credentials(&dataplane, &egress).await;

        Ok(self
            .forward_and_log(
                &dataplane,
                data_plane_id,
                path,
                &egress,
                &target,
                outbound,
                &credentials,
                policy,
                is_provider_finite,
            )
            .await)
    }

    fn delivery_policy(dataplane: &DataplaneTransferDto) -> DeliveryPolicy {
        dataplane
            .inner
            .flow_control
            .as_ref()
            .and_then(|v| serde_json::from_value::<DataplaneRuntime>(v.clone()).ok())
            .map(|rt| rt.delivery_policy)
            .unwrap_or_default()
    }

    fn emit_completed(&self, dataplane: &DataplaneTransferDto) {
        if let Ok(transfer_process_id) = Urn::from_str(&dataplane.inner.transfer_process_id) {
            self.signals.emit(DataplaneSignal::Completed {
                transfer_process_id,
            });
        }
    }

    fn emit_failed(&self, dataplane: &DataplaneTransferDto, code: &str, reason: impl Into<String>) {
        if let Ok(transfer_process_id) = Urn::from_str(&dataplane.inner.transfer_process_id) {
            self.signals.emit(DataplaneSignal::Failed {
                transfer_process_id,
                code: code.to_string(),
                reason: reason.into(),
            });
        }
    }

    /// Parses the `{data_plane_id}` path segment into a transfer URN.
    fn parse_dataplane_id(raw: &str) -> Result<Urn, ProxyError> {
        get_urn_from_string(raw).map_err(|_| ProxyError::BadDataplaneId)
    }

    /// Loads the transfer and enforces that it is `Started` — the proxy only
    /// relays traffic for active transfers.
    async fn load_started_dataplane(&self, urn: &Urn) -> Result<DataplaneTransferDto, ProxyError> {
        let dataplane = self
            .dataplane_service
            .get_one(&UserInfo::system(), urn)
            .await
            .map_err(|e| match e {
                Errors::MissingResourceError { .. } => ProxyError::DataplaneNotFound,
                _ => ProxyError::DataplaneLookupFailed,
            })?;

        if dataplane.inner.state != TransferState::Started {
            return Err(ProxyError::NotStarted(dataplane.inner.state.clone()));
        }
        Ok(dataplane)
    }

    /// Enforces per-transfer ingress capability authentication when the transfer's
    /// `HttpListener` defines a non-empty token. Accepts either an `Authorization`
    /// header (`<token_type> <token>`, defaulting to `Bearer`) or an `ingress_token`
    /// query parameter (used by backend webhook callbacks).
    fn verify_ingress_auth(
        dataplane: &DataplaneTransferDto,
        headers: &HeaderMap,
        raw_query: Option<&str>,
    ) -> Result<(), ProxyError> {
        let Ok(DataplaneProxyIngress::HttpListener {
            token: Some(expected),
            token_type,
            ..
        }) = serde_json::from_value::<DataplaneProxyIngress>(
            dataplane.inner.ingress_config.clone(),
        )
        else {
            return Ok(());
        };

        if expected.is_empty() {
            return Ok(());
        }

        let expected_scheme = token_type
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or("Bearer");

        if let Some(raw_header) = headers.get(AUTHORIZATION) {
            let header_str = raw_header
                .to_str()
                .map_err(|_| ProxyError::InvalidIngressAuth)?
                .trim();
            let Some((scheme, rest)) = header_str.split_once(' ') else {
                return Err(ProxyError::InvalidIngressAuth);
            };
            if !scheme.eq_ignore_ascii_case(expected_scheme) {
                return Err(ProxyError::InvalidIngressAuth);
            }
            let candidate = rest.trim();
            if constant_time_eq(candidate.as_bytes(), expected.as_bytes()) {
                return Ok(());
            }
            return Err(ProxyError::InvalidIngressAuth);
        }

        if let Some(qs) = raw_query {
            for (k, v) in url::form_urlencoded::parse(qs.as_bytes()) {
                if k == INGRESS_TOKEN_QUERY_PARAM {
                    if constant_time_eq(v.as_bytes(), expected.as_bytes()) {
                        return Ok(());
                    }
                    return Err(ProxyError::InvalidIngressAuth);
                }
            }
        }

        Err(ProxyError::MissingIngressAuth)
    }

    /// Strips `ingress_token` from the query string so internal proxy tokens are
    /// never leaked to upstream services.
    fn strip_ingress_token_query(raw_query: Option<&str>) -> Option<String> {
        let qs = raw_query?;
        if !qs.contains(INGRESS_TOKEN_QUERY_PARAM) {
            return Some(qs.to_string());
        }
        let filtered: Vec<(String, String)> = url::form_urlencoded::parse(qs.as_bytes())
            .filter(|(k, _)| k != INGRESS_TOKEN_QUERY_PARAM)
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect();
        if filtered.is_empty() {
            None
        } else {
            let mut serializer = url::form_urlencoded::Serializer::new(String::new());
            serializer.extend_pairs(filtered);
            Some(serializer.finish())
        }
    }

    /// Reads the egress configuration the transfer was provisioned with.
    fn parse_egress(dataplane: &DataplaneTransferDto) -> Result<DataplaneProxyEgress, ProxyError> {
        serde_json::from_value(dataplane.inner.egress_config.clone())
            .map_err(|_| ProxyError::InvalidEgressConfig)
    }

    /// Extracts the method, headers and size-limited body from the request.
    async fn extract_outbound(mut req: Request) -> Result<OutboundRequest, ProxyError> {
        let method = req.method().clone();
        let headers = req.headers().clone();
        let body = std::mem::take(req.body_mut());
        let body = to_bytes(body, MAX_BODY_BYTES)
            .await
            .map_err(|_| ProxyError::BodyTooLarge)?;
        Ok(OutboundRequest {
            method,
            headers,
            body,
        })
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

        // Preserve the cleaned query string.
        if let Some(q) = query {
            if !q.is_empty() {
                url.push(if url.contains('?') { '&' } else { '?' });
                url.push_str(q);
            }
        }
        Ok(url)
    }

    /// Validates the upstream target URL scheme and, unless `allow_private_egress`
    /// is explicitly enabled, blocks SSRF against loopback, RFC1918 private networks,
    /// link-local/cloud metadata (`169.254.169.254`), and `.localhost`/`.internal` hosts.
    async fn validate_target_url(&self, target: &str) -> Result<(), ProxyError> {
        let parsed = url::Url::parse(target).map_err(|_| ProxyError::InvalidTargetUrl)?;
        if !matches!(parsed.scheme(), "http" | "https") {
            return Err(ProxyError::ForbiddenTarget("unsupported target URL scheme"));
        }
        let Some(host) = parsed.host() else {
            return Err(ProxyError::InvalidTargetUrl);
        };

        if self.allow_private_egress {
            return Ok(());
        }

        match host {
            Host::Ipv4(ip) => {
                if !is_global_ipv4(ip) {
                    return Err(ProxyError::ForbiddenTarget(
                        "private, loopback, or reserved target IP blocked",
                    ));
                }
            }
            Host::Ipv6(ip) => {
                if !is_global_ipv6(ip) {
                    return Err(ProxyError::ForbiddenTarget(
                        "private, loopback, or reserved target IP blocked",
                    ));
                }
            }
            Host::Domain(domain) => {
                let lower = domain.trim_end_matches('.').to_ascii_lowercase();
                if lower == "localhost"
                    || lower.ends_with(".localhost")
                    || lower.ends_with(".internal")
                {
                    return Err(ProxyError::ForbiddenTarget(
                        "local or internal target host blocked",
                    ));
                }
                let port = parsed.port_or_known_default().unwrap_or(80);
                let addrs: Vec<_> = tokio::net::lookup_host((domain, port))
                    .await
                    .map_err(|_| ProxyError::TargetResolutionFailed)?
                    .collect();
                if addrs.is_empty() {
                    return Err(ProxyError::TargetResolutionFailed);
                }
                if addrs.iter().any(|addr| !is_global_ip(addr.ip())) {
                    return Err(ProxyError::ForbiddenTarget(
                        "target host resolves to private, loopback, or reserved IP",
                    ));
                }
            }
        }
        Ok(())
    }

    /// Determines the credentials to present upstream.
    ///
    /// Only `(Provider, Pull)` forwards to the provider's own backend data source and
    /// therefore uses `flow_control.auth` (resolved via `RuntimeSecretVault`).
    /// All peer-facing or consumer-sink hops (`Consumer + Pull`, `Provider + Push`,
    /// `Consumer + Push`) use the token configured on `egress` (`DataAddress`),
    /// preventing provider backend credentials from ever leaking to a consumer sink.
    async fn resolve_credentials(
        &self,
        dataplane: &DataplaneTransferDto,
        egress: &DataplaneProxyEgress,
    ) -> ResolvedAuthCredentials {
        let uses_connector_auth = dataplane.inner.role == TransferRole::Provider
            && dataplane.inner.interaction_mode == InteractionMode::Pull;

        if !uses_connector_auth {
            return Self::egress_credentials(egress);
        }

        let runtime = dataplane
            .inner
            .flow_control
            .as_ref()
            .and_then(|v| serde_json::from_value::<DataplaneRuntime>(v.clone()).ok());

        // Resolve vaulted placeholders when a keystore is configured.
        let runtime = match (runtime, &self.keystore) {
            (Some(rt), Some(lookup)) => Some(
                RuntimeSecretVault::resolve_with_lookup(rt, lookup, &dataplane.inner.user_id)
                    .await,
            ),
            (rt, _) => rt,
        };

        let auth = runtime
            .map(|rt| rt.auth)
            .unwrap_or(ResolvedAuthCredentials::NoAuth);

        match auth {
            ResolvedAuthCredentials::NoAuth => Self::egress_credentials(egress),
            other => other,
        }
    }

    fn egress_credentials(egress: &DataplaneProxyEgress) -> ResolvedAuthCredentials {
        match egress {
            DataplaneProxyEgress::HttpProxy {
                token: Some(t),
                token_type,
                ..
            } if !t.is_empty() => match token_type.as_deref() {
                None => ResolvedAuthCredentials::BearerToken { token: t.clone() },
                Some(kind) if kind.eq_ignore_ascii_case("Bearer") => {
                    ResolvedAuthCredentials::BearerToken { token: t.clone() }
                }
                Some(kind) => ResolvedAuthCredentials::OAuth2 {
                    access_token: t.clone(),
                    token_type: kind.to_string(),
                    expires_at: None,
                    refresh_token: None,
                },
            },
            _ => ResolvedAuthCredentials::NoAuth,
        }
    }

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
        policy: DeliveryPolicy,
        is_provider_finite: bool,
    ) -> Response {
        let auth_preview = Self::preview_auth(credentials);
        let safe_target = Self::redact_url(target);
        debug!(
            "Proxy {} {} | keystore={} | {}",
            outbound.method,
            safe_target,
            self.keystore.is_some(),
            auth_preview
        );

        // Drop the inbound Authorization header — the proxy injects its own
        // outbound credentials and must never relay its ingress capability token.
        let mut headers = outbound.headers;
        headers.remove("authorization");

        let response = HttpProxyDriver::forward(
            &self.client,
            outbound.method.clone(),
            target,
            None, // the wildcard path is already baked into `target`
            &headers,
            outbound.body,
            credentials,
        )
        .await;

        self.record_event(
            dataplane,
            data_plane_id,
            path,
            egress,
            &safe_target,
            &outbound.method,
            &response,
        )
        .await;

        self.evaluate_and_map_response(
            dataplane,
            policy,
            is_provider_finite,
            response,
            &outbound.method,
            &safe_target,
            &auth_preview,
        )
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
        safe_target: &str,
        method: &Method,
        response: &Outcome<UpstreamResponse>,
    ) {
        let role = dataplane.inner.role.clone();
        let mode = dataplane.inner.interaction_mode.clone();
        let ingress_type = Self::ingress_label(dataplane);
        let egress_type = Self::egress_label(egress);
        let status = response.as_ref().map(|r| r.status().as_u16()).unwrap_or(0);

        let event = NewTransferEvent {
            owner: dataplane.inner.owner(),
            transfer_id: dataplane.inner.id.clone(),
            level: LogLevel::Info,
            component: "DataProxy".to_string(),
            message: format!(
                "Transfer [{role}|{mode}] from {data_plane_id} to {safe_target} | Ingress: {ingress_type} | Egress: {egress_type}"
            ),
            data: Some(json!({
                "role": role,
                "mode": mode,
                "ingress": ingress_type,
                "egress": egress_type,
                "origin_path": path,
                "method": method.to_string(),
                "target": safe_target,
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

    /// Evaluates control-plane signaling (`Completed` / `Failed` with `maxRetries`) and maps
    /// the upstream result to the client HTTP response, masking upstream error bodies.
    #[allow(clippy::too_many_arguments)]
    async fn evaluate_and_map_response(
        &self,
        dataplane: &DataplaneTransferDto,
        policy: DeliveryPolicy,
        is_provider_finite: bool,
        result: Outcome<UpstreamResponse>,
        method: &Method,
        safe_target: &str,
        auth_preview: &str,
    ) -> Response {
        let dp_id = &dataplane.inner.id;
        let role = &dataplane.inner.role;
        let mode = &dataplane.inner.interaction_mode;
        let keystore_present = self.keystore.is_some();

        match result {
            Ok(res) if res.status().is_client_error() => {
                let status = res.status();
                let body_text = res
                    .text()
                    .await
                    .unwrap_or_else(|_| "<unreadable>".to_string());
                error!(
                    "Upstream {} {} -> {} | sent=[{}] keystore={} | body: {}",
                    method, safe_target, status, auth_preview, keystore_present, body_text
                );
                self.classify_upstream_client_error(
                    dataplane,
                    policy,
                    is_provider_finite,
                    status,
                )
                .await;
                (status, Json(json!({"error": "upstream error"}))).into_response()
            }
            Ok(res) if res.status().is_server_error() => {
                let status = res.status();
                let body_text = res
                    .text()
                    .await
                    .unwrap_or_else(|_| "<unreadable>".to_string());
                error!(
                    "Upstream {} {} -> {} | sent=[{}] keystore={} | body: {}",
                    method, safe_target, status, auth_preview, keystore_present, body_text
                );
                if self
                    .tracker
                    .record_transient_failure(dp_id, policy.max_retries)
                    .await
                {
                    self.emit_failed(
                        dataplane,
                        "DATAPLANE_MAX_RETRIES_EXCEEDED",
                        format!("upstream returned {status}"),
                    );
                } else if is_provider_finite {
                    self.tracker.release_finite_slot(dp_id).await;
                }
                (status, Json(json!({"error": "upstream error"}))).into_response()
            }
            Ok(res) => {
                self.tracker.record_success(dp_id).await;
                if is_provider_finite {
                    match (role, mode) {
                        (TransferRole::Provider, InteractionMode::Pull) => {
                            return self.relay_finite_pull_response(dataplane, policy, res);
                        }
                        (TransferRole::Provider, InteractionMode::Push) => {
                            if self.tracker.complete_finite_slot(dp_id).await {
                                self.emit_completed(dataplane);
                            }
                        }
                        _ => {}
                    }
                }
                Self::relay_response(res)
            }
            Err(e) => {
                error!("Proxy forward failed {} {}: {:?}", method, safe_target, e);
                if self
                    .tracker
                    .record_transient_failure(dp_id, policy.max_retries)
                    .await
                {
                    self.emit_failed(
                        dataplane,
                        "DATAPLANE_MAX_RETRIES_EXCEEDED",
                        format!("upstream connection failed: {e}"),
                    );
                } else if is_provider_finite {
                    self.tracker.release_finite_slot(dp_id).await;
                }
                (
                    StatusCode::BAD_GATEWAY,
                    Json(json!({"error": "proxy error"})),
                )
                    .into_response()
            }
        }
    }

    async fn classify_upstream_client_error(
        &self,
        dataplane: &DataplaneTransferDto,
        policy: DeliveryPolicy,
        is_provider_finite: bool,
        status: StatusCode,
    ) {
        let dp_id = &dataplane.inner.id;
        let role = &dataplane.inner.role;
        let mode = &dataplane.inner.interaction_mode;

        match (role, mode) {
            (TransferRole::Provider, InteractionMode::Pull) => {
                if matches!(status, StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN) {
                    self.emit_failed(
                        dataplane,
                        "DATAPLANE_UPSTREAM_AUTH_REJECTED",
                        format!("backend rejected connector credentials ({status})"),
                    );
                } else if is_provider_finite {
                    if self
                        .tracker
                        .record_transient_failure(dp_id, policy.max_retries)
                        .await
                    {
                        self.emit_failed(
                            dataplane,
                            "DATAPLANE_FINITE_RESOURCE_ERROR",
                            format!("finite pull upstream returned {status}"),
                        );
                    } else {
                        self.tracker.release_finite_slot(dp_id).await;
                    }
                }
            }
            (TransferRole::Provider, InteractionMode::Push)
            | (TransferRole::Consumer, InteractionMode::Push) => {
                if matches!(
                    status,
                    StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN | StatusCode::GONE
                ) {
                    self.emit_failed(
                        dataplane,
                        "DATAPLANE_SINK_REJECTED",
                        format!("push sink permanently rejected delivery ({status})"),
                    );
                } else if is_provider_finite {
                    if self
                        .tracker
                        .record_transient_failure(dp_id, policy.max_retries)
                        .await
                    {
                        self.emit_failed(
                            dataplane,
                            "DATAPLANE_FINITE_PUSH_REJECTED",
                            format!("finite push sink returned {status}"),
                        );
                    } else {
                        self.tracker.release_finite_slot(dp_id).await;
                    }
                }
            }
            (TransferRole::Consumer, InteractionMode::Pull) => {
                if status == StatusCode::UNAUTHORIZED {
                    self.emit_failed(
                        dataplane,
                        "DATAPLANE_PEER_AUTH_REJECTED",
                        "provider proxy rejected consumer capability token (401 Unauthorized)",
                    );
                }
            }
        }
    }

    /// Streams the upstream response back to the client while arming `SignalOnEofStream`
    /// so `DataplaneSignal::Completed` fires once the payload reaches clean `EOF`.
    fn relay_finite_pull_response(
        &self,
        dataplane: &DataplaneTransferDto,
        policy: DeliveryPolicy,
        upstream: UpstreamResponse,
    ) -> Response {
        let status = upstream.status();
        let headers = upstream.headers().clone();
        let expected_bytes = if status == StatusCode::NO_CONTENT {
            Some(0)
        } else {
            headers
                .get(axum::http::header::CONTENT_LENGTH)
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.trim().parse::<u64>().ok())
        };
        let mut wrapped = SignalOnEofStream {
            inner: Box::pin(upstream.bytes_stream()),
            dataplane_id: dataplane.inner.id.clone(),
            transfer_process_id: Urn::from_str(&dataplane.inner.transfer_process_id).ok(),
            max_retries: policy.max_retries,
            expected_bytes,
            delivered_bytes: 0,
            tracker: self.tracker.clone(),
            signals: self.signals.clone(),
            finished: false,
        };
        if expected_bytes == Some(0) {
            wrapped.complete_once();
        }
        let body = Body::from_stream(wrapped);

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

    /// Streams the upstream response back to the client, dropping hop-by-hop
    /// headers that must not be relayed end-to-end (RFC 7230 §6.1).
    fn relay_response(upstream: UpstreamResponse) -> Response {
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

    /// Human-readable label of the configured ingress kind.
    fn ingress_label(dataplane: &DataplaneTransferDto) -> &'static str {
        match serde_json::from_value::<DataplaneProxyIngress>(
            dataplane.inner.ingress_config.clone(),
        ) {
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

    /// Renders a fully redacted, log-safe description of the outbound credentials.
    fn preview_auth(credentials: &ResolvedAuthCredentials) -> String {
        match credentials {
            ResolvedAuthCredentials::NoAuth => "Authorization: <none>".to_string(),
            ResolvedAuthCredentials::BearerToken { .. } => {
                "Authorization: Bearer [REDACTED]".to_string()
            }
            ResolvedAuthCredentials::ApiKey { key, location, .. } => {
                format!("ApiKey({location:?}:{key}=[REDACTED])")
            }
            ResolvedAuthCredentials::BasicAuth { username, .. } => {
                format!("Authorization: Basic <{username}:[REDACTED]>")
            }
            ResolvedAuthCredentials::OAuth2 { token_type, .. } => {
                format!("Authorization: {token_type} [REDACTED]")
            }
        }
    }

    /// Redacts sensitive query-string values (`token`, `key`, `secret`, `password`,
    /// `auth`, `sig`, `credential`) before writing a URL to logs or transfer events.
    pub fn redact_url(raw: &str) -> String {
        let Ok(mut parsed) = url::Url::parse(raw) else {
            return "<invalid-url>".to_string();
        };
        if parsed.password().is_some() {
            let _ = parsed.set_password(Some("[REDACTED]"));
        }
        if parsed.query().is_some() {
            let redacted: Vec<(String, String)> = parsed
                .query_pairs()
                .map(|(k, v)| {
                    let lower = k.to_ascii_lowercase();
                    if lower.contains("token")
                        || lower.contains("key")
                        || lower.contains("secret")
                        || lower.contains("password")
                        || lower.contains("auth")
                        || lower.contains("sig")
                        || lower.contains("credential")
                    {
                        (k.into_owned(), "[REDACTED]".to_string())
                    } else {
                        (k.into_owned(), v.into_owned())
                    }
                })
                .collect();
            parsed.query_pairs_mut().clear().extend_pairs(redacted);
        }
        parsed.to_string()
    }
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (&x, &y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

fn is_global_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_global_ipv4(v4),
        IpAddr::V6(v6) => is_global_ipv6(v6),
    }
}

/// Checks whether an IPv4 address is globally routable per IANA RFC 6890.
fn is_global_ipv4(ip: Ipv4Addr) -> bool {
    let octets = ip.octets();
    if octets[0] == 0 {
        return false; // 0.0.0.0/8 ("this network")
    }
    if ip.is_private() || ip.is_loopback() || ip.is_link_local() || ip.is_multicast() {
        return false;
    }
    if ip.is_documentation() || ip.is_broadcast() {
        return false;
    }
    if octets[0] == 100 && (octets[1] & 0b1100_0000) == 64 {
        return false; // 100.64.0.0/10 Shared Address Space (RFC 6598)
    }
    if octets[0] == 192 && octets[1] == 0 && octets[2] == 0 {
        return false; // 192.0.0.0/24 IETF Protocol Assignments
    }
    if octets[0] == 198 && (octets[1] & 0xfe) == 18 {
        return false; // 198.18.0.0/15 Benchmarking
    }
    if octets[0] >= 240 {
        return false; // 240.0.0.0/4 Reserved for Future Use + Broadcast
    }
    true
}

/// Checks whether an IPv6 address is globally routable per IANA RFC 6890.
fn is_global_ipv6(ip: Ipv6Addr) -> bool {
    if ip.is_unspecified() || ip.is_loopback() || ip.is_multicast() {
        return false;
    }
    if let Some(v4) = ip.to_ipv4_mapped() {
        return is_global_ipv4(v4);
    }
    let segs = ip.segments();
    if (segs[0] & 0xfe00) == 0xfc00 {
        return false; // fc00::/7 Unique Local
    }
    if (segs[0] & 0xffc0) == 0xfe80 {
        return false; // fe80::/10 Link-Local Unicast
    }
    if segs[0] == 0x2001 && segs[1] == 0x0db8 {
        return false; // 2001:db8::/32 Documentation
    }
    true
}
