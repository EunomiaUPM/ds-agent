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

use axum::body::Body;
use axum::extract::Request;
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use common::config::services::GatewayConfig;
use futures_util::TryStreamExt;

use reqwest::Client;

use common::config::services::traits::GatewayConfigTrait;
use common::config::types::traits::{CommonConfigTrait, MinKnownConfigTrait};
use std::time::Duration;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};
use ymir::config::traits::SingleHostTrait;
use ymir::config::types::HostType;

#[async_trait::async_trait]
pub trait GatewayServiceTrait: Send + Sync {
    async fn proxy_request(
        &self,
        service_prefix: String,
        extra_opt: Option<String>,
        req: Request<Body>,
    ) -> Response;

    async fn proxy_dsp_request(
        &self,
        service_prefix: String,
        extra_opt: Option<String>,
        req: Request<Body>,
    ) -> Response;

    async fn proxy_well_known_rpc_request(&self, extra: String, req: Request<Body>) -> Response;

    fn get_notification_sender(&self) -> broadcast::Sender<String>;
    fn get_config(&self) -> GatewayConfig;
}

pub struct GatewayService {
    config: GatewayConfig,
    client: Client,
    notification_tx: broadcast::Sender<String>,
}

impl GatewayService {
    pub fn new(config: GatewayConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to build reqwest client");
        let (notification_tx, _) = broadcast::channel(100);
        Self {
            config,
            client,
            notification_tx,
        }
    }
}

#[async_trait::async_trait]
impl GatewayServiceTrait for GatewayService {
    fn get_notification_sender(&self) -> broadcast::Sender<String> {
        self.notification_tx.clone()
    }

    fn get_config(&self) -> GatewayConfig {
        self.config.clone()
    }

    async fn proxy_request(
        &self,
        service_prefix: String,
        extra_opt: Option<String>,
        req: Request<Body>,
    ) -> Response {
        let microservice_base_url = match service_prefix.as_str() {
            "catalogs" => self.config.catalog().get_host(HostType::Http),
            "datasets" => self.config.catalog().get_host(HostType::Http),
            "data-services" => self.config.transfer().get_host(HostType::Http),
            "distributions" => self.config.transfer().get_host(HostType::Http),
            "odrl-policies" => self.config.transfer().get_host(HostType::Http),
            "connector" => self.config.catalog().get_host(HostType::Http),
            "datahub" => self.config.catalog().get_host(HostType::Http),
            "negotiations" => self.config.contracts().get_host(HostType::Http),
            "transfers" => self.config.transfer().get_host(HostType::Http),
            "dataplane" => self.config.transfer().get_host(HostType::Http),
            "mates" => self.config.ssi_auth().get_host(HostType::Http),
            "wallet" => self.config.ssi_auth().get_host(HostType::Http),
            "vc-request" => self.config.ssi_auth().get_host(HostType::Http),
            "peer-connection" => self.config.ssi_auth().get_host(HostType::Http),
            "gate" => self.config.ssi_auth().get_host(HostType::Http),
            "subscriptions" => self.config.transfer().get_host(HostType::Http),
            "notifications" => self.config.transfer().get_host(HostType::Http),
            "peer-catalogs" => self.config.catalog().get_host(HostType::Http),
            "gaia" => self.config.ssi_auth().get_host(HostType::Http),
            "well-known" => self.config.common().hosts.http.get_host(),
            "v1" => self.config.common().hosts.http.get_host(),
            _ => return (StatusCode::NOT_FOUND, "prefix not found").into_response(),
        };

        let microservice_api_path = match service_prefix.as_str() {
            "catalogs" => "api/v1/catalog-agent/catalogs",
            "datasets" => "api/v1/catalog-agent/datasets",
            "data-services" => "api/v1/catalog-agent/data-services",
            "distributions" => "api/v1/catalog-agent/distributions",
            "odrl-policies" => "api/v1/catalog-agent/odrl-policies",
            "connector" => "api/v1/connector",
            "datahub" => "api/v1/catalog-agent/datahub",
            "negotiations" => "api/v1/negotiation-agent",
            "transfers" => "api/v1/transfer-agent",
            "dataplane" => "api/v1/dataplane",
            "mates" => "api/v1/mates",
            "wallet" => "api/v1/wallet",
            "vc-request" => "api/v1/vc-request",
            "peer-connection" => "api/v1/peer-connection",
            "gate" => "api/v1/gate",
            "notifications" => "api/v1/contract-negotiation/notifications",
            "subscriptions" => "api/v1/contract-negotiation/subscriptions",
            "peer-catalogs" => "api/v1/catalog-agent/peer-catalogs",
            "gaia" => "api/v1/gaia",
            "well-known" => ".well-known",
            "v1" => "api/v1",
            _ => {
                return (StatusCode::NOT_FOUND, "prefix not found in microservice").into_response()
            }
        };

        execute_proxy(
            self.client.clone(),
            microservice_base_url,
            microservice_api_path.to_string(),
            extra_opt,
            req,
        )
        .await
    }

    async fn proxy_dsp_request(
        &self,
        service_prefix: String,
        extra_opt: Option<String>,
        req: Request<Body>,
    ) -> Response {
        let microservice_base_url = match service_prefix.as_str() {
            "catalogs" => self.config.catalog().get_host(HostType::Http),
            "negotiations" => self.config.contracts().get_host(HostType::Http),
            "transfers" => self.config.transfer().get_host(HostType::Http),
            _ => return (StatusCode::NOT_FOUND, "prefix not found").into_response(),
        };

        let microservice_api_path = match service_prefix.as_str() {
            "catalogs" => "dsp/current/catalog",
            "negotiations" => "dsp/current/negotiations",
            "transfers" => "dsp/current/transfers",
            _ => {
                return (StatusCode::NOT_FOUND, "prefix not found in microservice").into_response()
            }
        };

        execute_proxy(
            self.client.clone(),
            microservice_base_url,
            microservice_api_path.to_string(),
            extra_opt,
            req,
        )
        .await
    }

    async fn proxy_well_known_rpc_request(&self, extra: String, req: Request<Body>) -> Response {
        let microservice_base_url = self.config.common().hosts.http.get_host();
        let microservice_api_path = format!("rpc/.well-known/{}", extra);

        execute_proxy(
            self.client.clone(),
            microservice_base_url,
            microservice_api_path,
            None,
            req,
        )
        .await
    }
}

pub async fn execute_proxy(
    client: Client,
    microservice_base_url: String,
    microservice_api_path: String,
    extra_opt: Option<String>,
    req: Request<Body>,
) -> Response {
    let method = req.method();
    let path = req.uri().path();
    let query = req.uri().query(); // Get the raw query string
    info!("{} {}", method, path);

    // Prepare target url
    let mut target_url_str = format!(
        "{}/{}",
        microservice_base_url.trim_end_matches('/'),
        microservice_api_path.trim_matches('/')
    );

    if let Some(extra_path) = extra_opt {
        let trimmed_extra = extra_path.trim_matches('/');
        if !trimmed_extra.is_empty() {
            target_url_str.push('/');
            target_url_str.push_str(trimmed_extra);
        }
    }

    // Prepare query
    if let Some(q) = query {
        target_url_str.push('?');
        target_url_str.push_str(q);
        debug!("Target URL with original query: {}", target_url_str);
    } else {
        debug!("No query parameters in original request.");
    }

    debug!("Redirecting to {}", target_url_str);
    // Prepare method
    let original_method = req.method().clone();

    // Prepare headers
    let mut original_headers = req.headers().clone();
    let headers_to_remove = [
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
    for header_name in headers_to_remove.iter() {
        original_headers.remove(*header_name);
    }
    if let Ok(host_val) = HeaderValue::from_str(&microservice_base_url) {
        original_headers.insert(axum::http::header::HOST, host_val);
    } else {
        warn!(
            "Host header not possible to be created Host: {}",
            microservice_base_url
        );
    }
    // X-Forwarded-*
    if let Some(client_ip) = req
        .extensions()
        .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
    {
        if let Ok(x_forwarded_for_val) = HeaderValue::from_str(&client_ip.0.ip().to_string()) {
            original_headers.insert("x-forwarded-for", x_forwarded_for_val);
        }
    }
    // X-Forwarded-Proto & X-Forwarded-Host
    if let Ok(val) = HeaderValue::try_from("http") {
        original_headers.insert("x-forwarded-proto", val);
    }
    if let Some(original_host) = req.headers().get(axum::http::header::HOST) {
        original_headers.insert("x-forwarded-host", original_host.clone());
    }

    // Body
    let axum_body = req.into_body();
    let stream_for_reqwest = http_body_util::BodyStream::new(axum_body)
        .try_filter_map(|frame: http_body::Frame<bytes::Bytes>| {
            futures_util::future::ready(Ok(frame.into_data().ok()))
        })
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync + 'static>);

    let reqwest_body = reqwest::Body::wrap_stream(stream_for_reqwest);

    // Send
    let backend_request = client
        .request(original_method, target_url_str.clone())
        .headers(original_headers)
        .body(reqwest_body);

    // Handle request
    match backend_request.send().await {
        Ok(backend_res) => {
            let status = backend_res.status();
            let version = backend_res.version();
            let mut headers = backend_res.headers().clone();

            for header_name in headers_to_remove.iter() {
                headers.remove(*header_name);
            }

            let axum_body = Body::from_stream(backend_res.bytes_stream());
            let mut response_builder = Response::builder().status(status).version(version);
            if let Some(response_headers_mut) = response_builder.headers_mut() {
                *response_headers_mut = headers;
            }

            response_builder.body(axum_body).unwrap_or_else(|e| {
                error!("Error construyendo la respuesta del proxy: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Error interno del proxy".to_string(),
                )
                    .into_response()
            })
        }
        Err(e) => {
            error!(
                "Error on requesting microservice '{}': {}",
                target_url_str.clone(),
                e
            );
            (
                StatusCode::BAD_GATEWAY,
                format!(
                    "Error on requesting backend '{}': {}",
                    target_url_str.clone(),
                    e
                ),
            )
                .into_response()
        }
    }
}
