/*
 *
 *  * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

use crate::http::router::create_core_provider_router;
use axum::serve;
use rainbow_common::config::traits::CommonConfigTrait;
use rainbow_common::config::ApplicationConfig;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use ymir::config::traits::{ConnectionConfigTrait, HostsConfigTrait};
use ymir::config::types::HostType;
use ymir::services::vault::vault_rs::VaultService;

pub struct CoreHttpWorker;

impl CoreHttpWorker {
    pub async fn spawn(
        config: &ApplicationConfig,
        vault: Arc<VaultService>,
        token: &CancellationToken,
    ) -> anyhow::Result<JoinHandle<()>> {
        // message
        let server_message = format!(
            "Starting Dataspace http server in {}",
            config.monolith().common().get_host(HostType::Http)
        );
        tracing::info!("{}", server_message);
        // router
        let router = create_core_provider_router(config, vault.clone()).await;
        // config
        let host = if config.monolith().common().is_local() { "127.0.0.1" } else { "0.0.0.0" };
        let port = config.monolith().common().get_weird_port(HostType::Http);
        let addr = format!("{}{}", host, port);
        // listener
        let listener = TcpListener::bind(&addr).await?;
        // gracefully cancelation token
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
}
