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

use crate::http::build_router;
use crate::http::transfer_message_router::TransferMessageRouter;
use crate::http::transfer_process_router::TransferProcessRouter;
use crate::setup::common_worker::{
    bind_listener, build_domain_services, shutdown_signal, spawn_server,
};
use axum::extract::Request;
use axum::response::IntoResponse;
use axum::{Router, serve};
use common::config::services::TransferConfig;
use common::config::types::traits::CommonConfigTrait;
use common::errors::CommonErrors;
use common::http_global_404::global_handler_404;
use common::http_tracing::trace_layer;
use common::well_known::WellKnownRoot;
use oauth::config::OAuthConfig;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tower_http::trace::{DefaultOnResponse, TraceLayer};
use uuid::Uuid;
use ymir::config::traits::{ApiConfigTrait, HostsConfigTrait};
use ymir::config::types::HostType;
use ymir::errors::Outcome;
use ymir::services::vault::global::VaultService;

const SERVICE: &str = "HTTP";
const OAUTH_AUDIENCE: &str = "transfer-agent-ref";
const OAUTH_ROUTER: &str = "/oauth";
const TRANSFER_PROCESSES_ROUTER: &str = "/transfer-processes";
const TRANSFER_MESSAGES_ROUTER: &str = "/transfer-messages";
const API_BASE: &str = "/transfer-agent-ref";

pub struct TransferHttpWorker {}

impl TransferHttpWorker {
    pub async fn spawn(
        config: &TransferConfig,
        vault: Arc<VaultService>,
        token: &CancellationToken,
    ) -> Outcome<JoinHandle<()>> {
        let router = Self::create_root_http_router(config, vault).await?;
        let listener =
            bind_listener(config.common().get_internal_port(HostType::Http), SERVICE).await?;
        let server =
            serve(listener, router).with_graceful_shutdown(shutdown_signal(token.clone(), SERVICE));
        Ok(spawn_server(SERVICE, server))
    }

    pub async fn create_root_http_router(
        config: &TransferConfig,
        vault: Arc<VaultService>,
    ) -> Outcome<Router> {
        let well_known = WellKnownRoot::get_well_known_router(&config.into())?;
        let api = build_api_router(config, vault).await?;
        let router = api
            .merge(well_known)
            .fallback(global_handler_404)
            .layer(trace_layer());
        Ok(router)
    }
}

/// Builds the versioned API router (transfer processes + messages) plus OAuth.
async fn build_api_router(config: &TransferConfig, vault: Arc<VaultService>) -> Outcome<Router> {
    let svc = build_domain_services(config, &vault).await?;
    // oauth_config
    let oauth_config = OAuthConfig::new(
        config.jwt_secret(),
        config.common().get_host(HostType::Http),
        OAUTH_AUDIENCE,
    );
    let oauth_router = oauth::setup::OAuthSetup::new().build_router(oauth_config, svc.db);
    let process_router = Router::new().nest(
        TRANSFER_PROCESSES_ROUTER,
        TransferProcessRouter::new(svc.process).router(),
    );
    let message_router = Router::new().nest(
        TRANSFER_MESSAGES_ROUTER,
        TransferMessageRouter::new(svc.message).router(),
    );
    let api_base = format!("{}{}", config.common().get_api_version(), API_BASE);
    Ok(Router::new()
        .nest(
            &api_base,
            build_router(svc.validator, process_router, message_router),
        )
        .nest(OAUTH_ROUTER, oauth_router))
}
