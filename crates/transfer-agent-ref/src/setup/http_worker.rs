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
 *  * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use std::sync::Arc;

use axum::extract::Request;
use axum::response::IntoResponse;
use axum::{Router, serve};
use common::config::services::TransferConfig;
use common::config::types::traits::CommonConfigTrait;
use common::errors::CommonErrors;
use common::well_known::WellKnownRoot;
use oauth::config::OAuthConfig;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tower_http::trace::{DefaultOnResponse, TraceLayer};
use uuid::Uuid;
use ymir::config::traits::{ApiConfigTrait, ConnectionConfigTrait, HostsConfigTrait};
use ymir::config::types::HostType;
use ymir::errors::{Errors, Outcome};
use ymir::services::vault::VaultTrait;
use ymir::services::vault::global::VaultService;

use crate::data::factory::DataFactory;
use crate::data::sea_orm::factory::SeaOrmDataFactory;
use crate::http::build_router;
use crate::http::transfer_message_router::TransferMessageRouter;
use crate::http::transfer_process_router::TransferProcessRouter;
use crate::services::transfer_message::service::TransferMessageService;
use crate::services::transfer_process::service::TransferProcessService;

pub struct TransferHttpWorker {}

impl TransferHttpWorker {
    pub async fn spawn(
        config: &TransferConfig,
        vault: Arc<VaultService>,
        token: &CancellationToken,
    ) -> Outcome<JoinHandle<()>> {
        let well_known_router = WellKnownRoot::get_well_known_router(&config.into())?;
        let router = Self::create_root_http_router(config, vault.clone())
            .await?
            .merge(well_known_router);

        let host = "0.0.0.0";
        let port = config.common().get_internal_port(HostType::Http);
        let addr = format!("{host}:{port}");

        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| Errors::crazy("Error listening on the socket", Some(Box::new(e))))?;
        tracing::info!("HTTP Transfer-Agent-Ref Service running on {}", addr);

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

    pub async fn create_root_http_router(
        config: &TransferConfig,
        vault: Arc<VaultService>,
    ) -> Outcome<Router> {
        let router = create_root_http_router(config, vault.clone())
            .await?
            .fallback(Self::handler_404)
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(|req: &Request<_>| {
                        let request_id = req
                            .headers()
                            .get("x-request-id")
                            .and_then(|v| v.to_str().ok())
                            .map(|s| s.to_owned())
                            .unwrap_or_else(|| Uuid::new_v4().to_string());
                        tracing::info_span!("request", id = %request_id)
                    })
                    .on_request(|request: &Request<_>, _span: &tracing::Span| {
                        tracing::info!("{} {}", request.method(), request.uri());
                    })
                    .on_response(DefaultOnResponse::new().level(tracing::Level::INFO)),
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

pub(crate) async fn create_root_http_router(
    config: &TransferConfig,
    vault: Arc<VaultService>,
) -> Outcome<Router> {
    let db_connection = vault.get_db_connection(config.common()).await?;

    let oauth_config = OAuthConfig::new(
        config.jwt_secret(),
        config.common().get_host(HostType::Http),
        "transfer-agent-ref",
    );
    let oauth_router =
        oauth::setup::OAuthSetup::new().build_router(oauth_config, db_connection.clone());

    let factory = SeaOrmDataFactory::new(db_connection);

    let process_svc = Arc::new(TransferProcessService::new(
        factory.transfer_process_repo(),
        factory.transfer_identifier_repo(),
    ));
    let message_svc = Arc::new(TransferMessageService::new(factory.transfer_message_repo()));

    let process_router = Router::new().nest(
        "/transfer-processes",
        TransferProcessRouter::new(process_svc).router(),
    );
    let message_router = Router::new().nest(
        "/transfer-messages",
        TransferMessageRouter::new(message_svc).router(),
    );

    let token_svc = oauth::setup::OAuthSetup::new().build_validator(config.jwt_secret());

    let api_base = format!("{}/transfer-agent-ref", config.common().get_api_version());
    let router = Router::new()
        .nest(
            &api_base,
            build_router(token_svc, process_router, message_router),
        )
        .nest("/oauth", oauth_router);

    Ok(router)
}
