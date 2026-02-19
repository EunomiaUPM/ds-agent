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

use crate::http::router::create_core_router;
use axum::serve;
use axum_server::tls_rustls::RustlsConfig;
use rainbow_common::config::traits::CommonConfigTrait;
use rainbow_common::config::ApplicationConfig;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::info;
use ymir::config::traits::{ConnectionConfigTrait, HostsConfigTrait};
use ymir::config::types::HostType;
use ymir::services::vault::vault_rs::VaultService;
use ymir::services::vault::VaultTrait;
use ymir::types::secrets::StringHelper;
use ymir::utils::expect_from_env;

pub struct CoreHttpWorker;

impl CoreHttpWorker {
    pub async fn spawn(
        config: &ApplicationConfig,
        vault: Arc<VaultService>,
        token: &CancellationToken,
    ) -> anyhow::Result<()> {
        // ) -> anyhow::Result<JoinHandle<()>> {
        // message
        let server_message = format!(
            "Starting Dataspace http server in {}",
            config.monolith().common().get_host(HostType::Http)
        );
        info!("{}", server_message);
        match config.monolith().common().is_tls_enabled() {
            true => {
                info!("Running with TLS active");
                Self::run_tls(config, vault).await
            }
            false => {
                info!("Running without TLS");
                Self::run(config, vault, token).await
            }
        }
    }

    pub async fn run_tls(
        config: &ApplicationConfig,
        vault: Arc<VaultService>,
    ) -> anyhow::Result<()> {
        let cert = expect_from_env("VAULT_APP_ROOT_CLIENT_KEY");
        let pkey = expect_from_env("VAULT_APP_CLIENT_KEY");
        let cert: StringHelper = vault.read(None, &cert).await?;
        let pkey: StringHelper = vault.read(None, &pkey).await?;

        rustls::crypto::ring::default_provider()
            .install_default()
            .expect("Unable to install crypto utils");

        let tls_config = RustlsConfig::from_pem(
            cert.data().as_bytes().to_vec(),
            pkey.data().as_bytes().to_vec(),
        )
        .await?;

        let router = create_core_router(config, vault.clone()).await;

        let port = config.monolith().common().hosts().get_tls_port(HostType::Http);
        let addr_str = if config.monolith().common().is_local() {
            format!("127.0.0.1:{}", port)
        } else {
            format!("0.0.0.0:{}", port)
        };
        let addr: SocketAddr = addr_str.parse()?;
        info!("Starting Authority server with TLS in {}", addr);

        axum_server::bind_rustls(addr, tls_config).serve(router.into_make_service()).await?;
        Ok(())
    }
    pub async fn run(
        config: &ApplicationConfig,
        vault: Arc<VaultService>,
        token: &CancellationToken,
    ) -> anyhow::Result<()> {
        // ) -> anyhow::Result<JoinHandle<()>> {
        // router
        let router = create_core_router(config, vault.clone()).await;
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

        //serve(listener, router).await?;
        Ok(())
    }
}
