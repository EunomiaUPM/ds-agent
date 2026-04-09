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

use crate::gateway::GatewayHttpRouter;
use axum::extract::Request;
use axum::response::IntoResponse;
use axum::{serve, Router};
use common::config::services::GatewayConfig;
use common::config::types::traits::CommonConfigTrait;
use common::errors::CommonErrors;
use common::well_known::WellKnownRoot;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tower_http::trace::{DefaultOnResponse, TraceLayer};
use uuid::Uuid;
use ymir::config::traits::{ConnectionConfigTrait, HostsConfigTrait};
use ymir::config::types::HostType;
use ymir::errors::{Errors, Outcome};
use ymir::http::HealthRouter;

pub struct GatewayHttpWorker {}

impl GatewayHttpWorker {
    pub async fn spawn(
        config: &GatewayConfig,
        token: &CancellationToken,
    ) -> Outcome<JoinHandle<()>> {
        // well known router
        let well_known_router = WellKnownRoot::get_well_known_router(&config.into())?;
        let health_router = HealthRouter::new().router();
        // module catalog router
        let router = Self::create_root_http_router(&config)
            .await?
            .merge(well_known_router)
            .merge(health_router);
        let host = if config.common().is_local() {
            "127.0.0.1"
        } else {
            "0.0.0.0"
        };
        let port = config.common().get_internal_port(HostType::Http);
        let addr = format!("{}{}", host, port);

        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| Errors::crazy("Error with TCP listener", Some(Box::new(e))))?;
        tracing::info!("HTTP Gateway running on {}", addr);

        let token = token.clone();
        let handle = tokio::spawn(async move {
            let server = serve(listener, router).with_graceful_shutdown(async move {
                token.cancelled().await;
                tracing::info!("HTTP Service received shutdown signal, draining connections...");
            });
            match server.await {
                Ok(_) => tracing::info!("HTTP Service stopped successfully"),
                Err(e) => tracing::error!("HTTP Service crashed: {}", e),
            }
        });
        Ok(handle)
    }

    pub async fn create_root_http_router(config: &GatewayConfig) -> Outcome<Router> {
        let router = create_gateway_http_router(config)
            .await
            .fallback(Self::handler_404)
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(
                        |_req: &Request<_>| tracing::info_span!("request", id = %Uuid::new_v4()),
                    )
                    .on_request(|request: &Request<_>, _span: &tracing::Span| {
                        tracing::info!("{} {}", request.method(), request.uri());
                    })
                    .on_response(DefaultOnResponse::new().level(tracing::Level::TRACE)),
            );
        Ok(router)
    }

    async fn handler_404(uri: axum::http::Uri) -> impl IntoResponse {
        let err = CommonErrors::missing_resource_new(
            &uri.to_string(),
            "Route not found or Method not allowed",
        );
        tracing::info!("404 Not Found: {}", uri);
        err.into_response()
    }
}

pub async fn create_gateway_http_router(config: &GatewayConfig) -> Router {
    let gateway_router = GatewayHttpRouter::new(config.clone()).router();
    Router::new().nest("/admin", gateway_router)
}
