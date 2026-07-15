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

use std::fmt::Display;
use std::future::IntoFuture;
use std::sync::Arc;

use common::auth::middleware::TokenValidator;
use common::config::services::TransferConfig;
use common::config::types::traits::CommonConfigTrait;
use sea_orm::DatabaseConnection;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use ymir::errors::{Errors, Outcome};
use ymir::services::vault::VaultTrait;
use ymir::services::vault::global::VaultService;

use crate::data::factory::DataFactory;
use crate::data::sea_orm::factory::SeaOrmDataFactory;
use crate::services::transfer_message::service::TransferMessageService;
use crate::services::transfer_process::service::TransferProcessService;

/// Domain services shared by the HTTP and gRPC workers.
pub(super) struct DomainServices {
    pub process: Arc<TransferProcessService>,
    pub message: Arc<TransferMessageService>,
    pub validator: Arc<dyn TokenValidator>,
    pub db: DatabaseConnection,
}

/// Opens the DB connection and wires the repositories, services and token validator.
pub(super) async fn build_domain_services(
    config: &TransferConfig,
    vault: &VaultService,
) -> Outcome<DomainServices> {
    let db = vault.get_db_connection(config.common()).await?;
    let factory = SeaOrmDataFactory::new(db.clone());
    Ok(DomainServices {
        process: Arc::new(TransferProcessService::new(
            factory.transfer_process_repo(),
            factory.transfer_identifier_repo(),
        )),
        message: Arc::new(TransferMessageService::new(factory.transfer_message_repo())),
        validator: oauth::setup::OAuthSetup::new().build_validator(config.jwt_secret()),
        db,
    })
}

/// Binds `0.0.0.0:<port>`, logging under `service` (e.g. "HTTP", "gRPC").
pub(super) async fn bind_listener(port: String, service: &str) -> Outcome<TcpListener> {
    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr)
        .await
        .map_err(|e| Errors::crazy("Error binding socket", Some(Box::new(e))))?;
    tracing::info!("{service} Transfer-Agent-Ref Service running on {addr}");
    Ok(listener)
}

/// Resolves when `token` is cancelled; the drain future passed to the servers.
pub(super) async fn shutdown_signal(token: CancellationToken, service: &'static str) {
    token.cancelled().await;
    tracing::info!("{service} Service received shutdown signal, draining connections...");
}

/// Spawns the server (any `IntoFuture` resolving to `Result`) and logs its
/// clean stop or crash. Covers both axum's `WithGracefulShutdown` and tonic's
/// server future.
pub(super) fn spawn_server<F, E>(service: &'static str, server: F) -> JoinHandle<()>
where
    F: IntoFuture<Output = Result<(), E>> + Send + 'static,
    F::IntoFuture: Send,
    E: Display,
{
    tokio::spawn(async move {
        match server.await {
            Ok(_) => tracing::info!("{service} Service stopped successfully"),
            Err(e) => tracing::error!("{service} Service crashed: {e}"),
        }
    })
}
