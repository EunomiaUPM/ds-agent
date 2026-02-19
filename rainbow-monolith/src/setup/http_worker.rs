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
use tracing::{error, info};
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
    ) -> anyhow::Result<JoinHandle<()>> {
        let server_message = format!(
            "Starting Dataspace http server in {}",
            config.monolith().common().get_host(HostType::Http)
        );
        info!("{}", server_message);

        if config.monolith().common().is_tls_enabled() {
            info!("Running with TLS active");
            Self::run_tls(config, vault, token).await
        } else {
            info!("Running without TLS");
            Self::run(config, vault, token).await
        }
    }

    pub async fn run_tls(
        config: &ApplicationConfig,
        vault: Arc<VaultService>,
        token: &CancellationToken,
    ) -> anyhow::Result<JoinHandle<()>> {
        let cert_key = expect_from_env("VAULT_APP_ROOT_CLIENT_KEY");
        let pkey_key = expect_from_env("VAULT_APP_CLIENT_KEY");
        let cert: StringHelper = vault.read(None, &cert_key).await?;
        let pkey: StringHelper = vault.read(None, &pkey_key).await?;

        // Evitar doble instalación del provider de Ring
        let _ = rustls::crypto::ring::default_provider().install_default();

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

        let server_handle = axum_server::Handle::new();
        let shutdown_token = token.clone();
        let axum_handle_clone = server_handle.clone();

        // Tarea de monitoreo de señal de apagado
        tokio::spawn(async move {
            shutdown_token.cancelled().await;
            info!("TLS HTTP Service received shutdown signal, draining connections...");
            axum_handle_clone.graceful_shutdown(Some(std::time::Duration::from_secs(10)));
        });

        // Retornamos el JoinHandle de la tarea del servidor
        let handle = tokio::spawn(async move {
            let server = axum_server::bind_rustls(addr, tls_config)
                .handle(server_handle)
                .serve(router.into_make_service());

            if let Err(e) = server.await {
                error!("TLS HTTP Service crashed: {}", e);
            } else {
                info!("TLS HTTP Service stopped successfully");
            }
        });

        Ok(handle)
    }

    pub async fn run(
        config: &ApplicationConfig,
        vault: Arc<VaultService>,
        token: &CancellationToken,
    ) -> anyhow::Result<JoinHandle<()>> {
        let router = create_core_router(config, vault.clone()).await;

        let port = config.monolith().common().get_weird_port(HostType::Http);
        let host = if config.monolith().common().is_local() { "127.0.0.1" } else { "0.0.0.0" };
        let addr: SocketAddr = format!("{}:{}", host, port).parse()?;

        let listener = TcpListener::bind(&addr).await?;
        let token_clone = token.clone();

        let handle = tokio::spawn(async move {
            let server = axum::serve(listener, router).with_graceful_shutdown(async move {
                token_clone.cancelled().await;
            });

            if let Err(e) = server.await {
                error!("HTTP Service crashed: {}", e);
            } else {
                info!("HTTP Service stopped successfully");
            }
        });

        Ok(handle)
    }
}
