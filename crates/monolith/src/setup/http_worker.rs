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

use std::net::SocketAddr;
use std::sync::Arc;

use axum::serve;
use axum_server::tls_rustls::RustlsConfig;
use common::config::types::traits::CommonConfigTrait;
use common::config::ApplicationConfig;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};
use ymir::config::traits::{ConnectionConfigTrait, HostsConfigTrait};
use ymir::config::types::HostType;
use ymir::errors::{Errors, Outcome};
use ymir::services::vault::global::VaultService;
use ymir::services::vault::VaultTrait;
use ymir::types::secrets::StringHelper;
use ymir::utils::expect_from_env;

use crate::setup::router::create_core_router;

pub struct CoreHttpWorker;

impl CoreHttpWorker {
    pub async fn spawn(
        config: &ApplicationConfig,
        vault: Arc<VaultService>,
        token: &CancellationToken,
    ) -> Outcome<JoinHandle<()>> {
        let server_message = format!(
            "Starting Dataspace http server in {}",
            config.monolith().common().get_host(HostType::Http)
        );
        info!("{}", server_message);

        if config.monolith().common().is_prod() && !config.monolith().common().has_tls_proxy() {
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
    ) -> Outcome<JoinHandle<()>> {
        let cert_key = expect_from_env("VAULT_APP_ROOT_CLIENT_KEY");
        let pkey_key = expect_from_env("VAULT_APP_CLIENT_KEY");
        let cert: StringHelper = vault.read(None, &cert_key).await?;
        let pkey: StringHelper = vault.read(None, &pkey_key).await?;

        let _ = rustls::crypto::ring::default_provider().install_default();

        let tls_config = RustlsConfig::from_pem(
            cert.data().as_bytes().to_vec(),
            pkey.data().as_bytes().to_vec(),
        )
        .await
        .map_err(|e| Errors::crazy("Errors parsing certificate stuff", Some(Box::new(e))))?;

        let router = create_core_router(config, vault.clone()).await;
        let port = config
            .monolith()
            .common()
            .hosts()
            .get_internal_port(HostType::Http);
        let addr = format!("0.0.0.0:{}", port);
        info!("Starting Eunomia DS-Agent server with TLS in {}", addr);

        let addr: SocketAddr = addr
            .parse()
            .map_err(|e| Errors::crazy("Errors with socker address", Some(Box::new(e))))?;

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
    ) -> Outcome<JoinHandle<()>> {
        let router = create_core_router(config, vault.clone()).await;

        let port = config
            .monolith()
            .common()
            .hosts()
            .get_internal_port(HostType::Http);
        let addr = format!("0.0.0.0:{}", port);
        info!("Starting Eunomia DS-Agent server in {}", addr);

        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| Errors::crazy("Error with tcp listener", Some(Box::new(e))))?;
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
