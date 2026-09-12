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

use std::sync::Arc;

use axum::body::Body;
use axum::extract::Request;
use axum::response::Response;
use common::config::services::GatewayConfig;
use tokio::sync::broadcast;

use crate::proxy::HttpProxyDispatcher;

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

/// Service delegating gateway routing to the underlying proxy dispatcher.
pub struct GatewayService {
    config: GatewayConfig,
    proxy: Arc<HttpProxyDispatcher>,
    notification_tx: broadcast::Sender<String>,
}

impl GatewayService {
    /// Initialize gateway service with configuration and proxy dispatcher.
    pub fn new(config: GatewayConfig) -> Self {
        let proxy = Arc::new(HttpProxyDispatcher::new(config.clone()));
        let (notification_tx, _) = broadcast::channel(100);
        Self {
            config,
            proxy,
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
        self.proxy
            .proxy_request(service_prefix, extra_opt, req)
            .await
    }

    async fn proxy_dsp_request(
        &self,
        service_prefix: String,
        extra_opt: Option<String>,
        req: Request<Body>,
    ) -> Response {
        self.proxy
            .proxy_dsp_request(service_prefix, extra_opt, req)
            .await
    }

    async fn proxy_well_known_rpc_request(&self, extra: String, req: Request<Body>) -> Response {
        self.proxy.proxy_well_known_rpc_request(extra, req).await
    }
}
