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

use std::time::Duration;

use axum::body::Body;
use axum::extract::Request;
use axum::http::header::HeaderName;
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use common::auth::claims::Claims;
use common::config::services::traits::GatewayConfigTrait;
use common::config::services::GatewayConfig;
use common::config::types::traits::{CommonConfigTrait, MinKnownConfigTrait};
use futures_util::TryStreamExt;
use reqwest::Client;
use tracing::{debug, error, info};
use uuid::Uuid;
use ymir::config::traits::SingleHostTrait;
use ymir::config::types::HostType;

/// Reverse proxy dispatcher with connection pooling, tracing headers, and user propagation.
#[derive(Clone)]
pub struct HttpProxyDispatcher {
    config: GatewayConfig,
    client: Client,
}

impl HttpProxyDispatcher {
    /// Initialize dispatcher with tuned connection pooling and timeouts.
    pub fn new(config: GatewayConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(32)
            .tcp_keepalive(Duration::from_secs(60))
            .build()
            .unwrap_or_default();

        Self { config, client }
    }

    /// Proxy generic microservice requests matching service prefix.
    pub async fn proxy_request(
        &self,
        service_prefix: String,
        extra_opt: Option<String>,
        req: Request<Body>,
    ) -> Response {
        let (base_url, api_path) = match self.resolve_upstream(&service_prefix) {
            Some(pair) => pair,
            None => return (StatusCode::NOT_FOUND, "service prefix not found").into_response(),
        };

        self.execute(&base_url, &api_path, extra_opt, req).await
    }

    /// Proxy Dataspace Protocol (DSP) version-specific requests.
    pub async fn proxy_dsp_request(
        &self,
        service_prefix: String,
        extra_opt: Option<String>,
        req: Request<Body>,
    ) -> Response {
        let base_url = match service_prefix.as_str() {
            "catalogs" => self.config.catalog().get_host(HostType::Http),
            "negotiations" => self.config.contracts().get_host(HostType::Http),
            "transfers" => self.config.transfer().get_host(HostType::Http),
            _ => return (StatusCode::NOT_FOUND, "dsp service prefix not found").into_response(),
        };

        let api_path = match service_prefix.as_str() {
            "catalogs" => "dsp/current/catalog",
            "negotiations" => "dsp/current/negotiations",
            "transfers" => "dsp/current/transfers",
            _ => return (StatusCode::NOT_FOUND, "dsp path not found").into_response(),
        };

        self.execute(&base_url, api_path, extra_opt, req).await
    }

    /// Proxy well-known RPC requests.
    pub async fn proxy_well_known_rpc_request(
        &self,
        extra: String,
        req: Request<Body>,
    ) -> Response {
        let base_url = self.config.common().hosts.http.get_host();
        let api_path = format!("rpc/.well-known/{extra}");
        self.execute(&base_url, &api_path, None, req).await
    }

    fn resolve_upstream(&self, prefix: &str) -> Option<(String, String)> {
        match prefix {
            "catalogs" => Some((
                self.config.catalog().get_host(HostType::Http),
                "api/v1/catalog-agent/catalogs".to_string(),
            )),
            "datasets" => Some((
                self.config.catalog().get_host(HostType::Http),
                "api/v1/catalog-agent/datasets".to_string(),
            )),
            "data-services" => Some((
                self.config.catalog().get_host(HostType::Http),
                "api/v1/catalog-agent/data-services".to_string(),
            )),
            "distributions" => Some((
                self.config.catalog().get_host(HostType::Http),
                "api/v1/catalog-agent/distributions".to_string(),
            )),
            "odrl-policies" => Some((
                self.config.catalog().get_host(HostType::Http),
                "api/v1/catalog-agent/odrl-policies".to_string(),
            )),
            "connector" => Some((
                self.config.catalog().get_host(HostType::Http),
                "api/v1/connector".to_string(),
            )),
            "datahub" => Some((
                self.config.catalog().get_host(HostType::Http),
                "api/v1/catalog-agent/datahub".to_string(),
            )),
            "peer-catalogs" => Some((
                self.config.catalog().get_host(HostType::Http),
                "api/v1/catalog-agent/peer-catalogs".to_string(),
            )),
            "negotiations" => Some((
                self.config.contracts().get_host(HostType::Http),
                "api/v1/negotiation-agent".to_string(),
            )),
            "transfers" => Some((
                self.config.transfer().get_host(HostType::Http),
                "api/v1/transfer-agent".to_string(),
            )),
            "dataplane" => Some((
                self.config.transfer().get_host(HostType::Http),
                "api/v1/dataplane".to_string(),
            )),
            "mates" => Some((
                self.config.ssi_auth().get_host(HostType::Http),
                "api/v1/mates".to_string(),
            )),
            "wallet" => Some((
                self.config.ssi_auth().get_host(HostType::Http),
                "api/v1/wallet".to_string(),
            )),
            "vc-request" => Some((
                self.config.ssi_auth().get_host(HostType::Http),
                "api/v1/vc-request".to_string(),
            )),
            "peer-connection" | "onboard" => Some((
                self.config.ssi_auth().get_host(HostType::Http),
                "api/v1/peer-connection".to_string(),
            )),
            "gate" => Some((
                self.config.ssi_auth().get_host(HostType::Http),
                "api/v1/gate".to_string(),
            )),
            "gaia" => Some((
                self.config.ssi_auth().get_host(HostType::Http),
                "api/v1/gaia".to_string(),
            )),
            "subscriptions" => Some((
                self.config.transfer().get_host(HostType::Http),
                "api/v1/contract-negotiation/subscriptions".to_string(),
            )),
            "notifications" => Some((
                self.config.transfer().get_host(HostType::Http),
                "api/v1/contract-negotiation/notifications".to_string(),
            )),
            "oauth" | "auth" => Some((
                self.config.common().hosts.http.get_host(),
                "oauth".to_string(),
            )),
            "well-known" => Some((
                self.config.common().hosts.http.get_host(),
                ".well-known".to_string(),
            )),
            "v1" => Some((
                self.config.common().hosts.http.get_host(),
                "api/v1".to_string(),
            )),
            _ => None,
        }
    }

    async fn execute(
        &self,
        base_url: &str,
        api_path: &str,
        extra_opt: Option<String>,
        req: Request<Body>,
    ) -> Response {
        let mut target_url = format!(
            "{}/{}",
            base_url.trim_end_matches('/'),
            api_path.trim_matches('/')
        );
        if let Some(extra) = extra_opt {
            let trimmed = extra.trim_matches('/');
            if !trimmed.is_empty() {
                target_url.push('/');
                target_url.push_str(trimmed);
            }
        }

        if let Some(query) = req.uri().query() {
            target_url.push('?');
            target_url.push_str(query);
        }

        let method = req.method().clone();
        let mut headers = req.headers().clone();

        let correlation_id = headers
            .get("x-correlation-id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("urn:uuid:{}", Uuid::new_v4()));

        let request_id = Uuid::new_v4().to_string();

        let hop_by_hop = [
            "connection",
            "keep-alive",
            "proxy-authenticate",
            "proxy-authorization",
            "te",
            "trailers",
            "transfer-encoding",
            "upgrade",
            "host",
        ];
        for h in hop_by_hop {
            headers.remove(h);
        }

        if let Ok(v) = HeaderValue::from_str(&correlation_id) {
            headers.insert(HeaderName::from_static("x-correlation-id"), v);
        }
        if let Ok(v) = HeaderValue::from_str(&request_id) {
            headers.insert(HeaderName::from_static("x-request-id"), v);
        }

        if let Some(claims) = req.extensions().get::<Claims>() {
            if let Ok(v) = HeaderValue::from_str(&claims.sub) {
                headers.insert(HeaderName::from_static("x-user-id"), v);
            }
            if let Ok(v) = HeaderValue::from_str(&claims.role.to_string()) {
                headers.insert(HeaderName::from_static("x-user-role"), v);
            }
        }

        let body_stream = http_body_util::BodyStream::new(req.into_body())
            .try_filter_map(|frame| futures_util::future::ready(Ok(frame.into_data().ok())))
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync + 'static>);

        let reqwest_body = reqwest::Body::wrap_stream(body_stream);

        match self
            .client
            .request(method, &target_url)
            .headers(headers)
            .body(reqwest_body)
            .send()
            .await
        {
            Ok(upstream_res) => {
                let status = upstream_res.status();
                let version = upstream_res.version();
                let mut resp_headers = upstream_res.headers().clone();

                for h in hop_by_hop {
                    resp_headers.remove(h);
                }

                if let Ok(v) = HeaderValue::from_str(&correlation_id) {
                    resp_headers.insert(HeaderName::from_static("x-correlation-id"), v);
                }

                let body = Body::from_stream(upstream_res.bytes_stream());
                let mut builder = Response::builder().status(status).version(version);
                if let Some(h_mut) = builder.headers_mut() {
                    *h_mut = resp_headers;
                }
                builder.body(body).unwrap_or_else(|e| {
                    error!(error = %e, "Failed to assemble proxy response");
                    (StatusCode::INTERNAL_SERVER_ERROR, "proxy response error").into_response()
                })
            }
            Err(e) => {
                error!(target = %target_url, error = %e, "Upstream microservice request failed");
                (
                    StatusCode::BAD_GATEWAY,
                    format!("Upstream microservice unreachable: {e}"),
                )
                    .into_response()
            }
        }
    }
}
