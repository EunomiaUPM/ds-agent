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

//! HTTP and gRPC servers as background workers; sockets are bound before spawning.

use std::net::SocketAddr;
use std::time::Duration;

use axum::Router;
use axum_server::tls_rustls::RustlsConfig;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tonic::codegen::tokio_stream::wrappers::TcpListenerStream;
use tonic::service::Routes;
use tonic::transport::Server;
use ymir::errors::{Errors, Outcome};
use ymir::services::vault::global::VaultService;
use ymir::services::vault::VaultTrait;
use ymir::types::secrets::StringHelper;
use ymir::utils::expect_from_env;

use crate::boot::workers::BackgroundWorker;

/// Time in-flight TLS connections get to finish once shutdown starts.
const TLS_DRAIN: Duration = Duration::from_secs(10);

/// Axum server, plain or TLS-terminated.
pub struct HttpServer {
    listener: TcpListener,
    router: Router,
    tls: Option<RustlsConfig>,
}

impl HttpServer {
    pub async fn bind(port: String, router: Router, tls: Option<RustlsConfig>) -> Outcome<Self> {
        let listener = Self::listen(port, "HTTP").await?;
        Ok(Self {
            listener,
            router,
            tls,
        })
    }

    pub fn local_addr(&self) -> Outcome<SocketAddr> {
        self.listener
            .local_addr()
            .map_err(|e| Errors::crazy("HTTP listener has no local address", Some(Box::new(e))))
    }

    /// TLS material stored in the vault under the keys named by the environment.
    pub async fn tls_from_vault(vault: &VaultService) -> Outcome<RustlsConfig> {
        let cert: StringHelper = vault
            .read(None, &expect_from_env("VAULT_APP_ROOT_CLIENT_KEY"))
            .await?;
        let key: StringHelper = vault
            .read(None, &expect_from_env("VAULT_APP_CLIENT_KEY"))
            .await?;
        let _ = rustls::crypto::ring::default_provider().install_default();
        RustlsConfig::from_pem(
            cert.data().as_bytes().to_vec(),
            key.data().as_bytes().to_vec(),
        )
        .await
        .map_err(|e| Errors::crazy("Error parsing TLS certificate", Some(Box::new(e))))
    }

    /// Binds `0.0.0.0:<port>`, logging under `service`.
    pub(crate) async fn listen(port: String, service: &str) -> Outcome<TcpListener> {
        let addr = format!("0.0.0.0:{port}");
        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| Errors::crazy(format!("Error binding {addr}"), Some(Box::new(e))))?;
        tracing::info!("{service} service listening on {addr}");
        Ok(listener)
    }

    async fn serve_tls(self, tls: RustlsConfig, token: CancellationToken) -> Outcome<()> {
        let listener = self
            .listener
            .into_std()
            .map_err(|e| Errors::crazy("Error detaching TLS listener", Some(Box::new(e))))?;
        let handle = axum_server::Handle::new();
        let drain = handle.clone();
        tokio::spawn(async move {
            token.cancelled().await;
            drain.graceful_shutdown(Some(TLS_DRAIN));
        });
        axum_server::tls_rustls::from_tcp_rustls(listener, tls)
            .map_err(|e| Errors::crazy("Error creating TLS server", Some(Box::new(e))))?
            .handle(handle)
            .serve(self.router.into_make_service())
            .await
            .map_err(|e| Errors::crazy("TLS HTTP server crashed", Some(Box::new(e))))
    }
}

#[async_trait::async_trait]
impl BackgroundWorker for HttpServer {
    fn name(&self) -> &'static str {
        if self.tls.is_some() {
            "https"
        } else {
            "http"
        }
    }

    async fn run(mut self: Box<Self>, token: CancellationToken) -> Outcome<()> {
        if let Some(tls) = self.tls.take() {
            return self.serve_tls(tls, token).await;
        }
        axum::serve(self.listener, self.router)
            .with_graceful_shutdown(token.cancelled_owned())
            .await
            .map_err(|e| Errors::crazy("HTTP server crashed", Some(Box::new(e))))
    }
}

/// Tonic server hosting the composed gRPC routes plus a v1 reflection service.
pub struct GrpcServer {
    listener: TcpListener,
    routes: Routes,
    descriptors: Vec<&'static [u8]>,
}

impl GrpcServer {
    pub async fn bind(
        port: String,
        routes: Routes,
        descriptors: Vec<&'static [u8]>,
    ) -> Outcome<Self> {
        let listener = HttpServer::listen(port, "gRPC").await?;
        Ok(Self {
            listener,
            routes,
            descriptors,
        })
    }
}

#[async_trait::async_trait]
impl BackgroundWorker for GrpcServer {
    fn name(&self) -> &'static str {
        "grpc"
    }

    async fn run(self: Box<Self>, token: CancellationToken) -> Outcome<()> {
        let mut reflection = tonic_reflection::server::Builder::configure();
        for descriptor in self.descriptors {
            reflection = reflection.register_encoded_file_descriptor_set(descriptor);
        }
        let reflection = reflection
            .build_v1()
            .map_err(|e| Errors::crazy("Error building gRPC reflection", Some(Box::new(e))))?;
        Server::builder()
            .add_routes(self.routes)
            .add_service(reflection)
            .serve_with_incoming_shutdown(
                TcpListenerStream::new(self.listener),
                token.cancelled_owned(),
            )
            .await
            .map_err(|e| Errors::crazy("gRPC server crashed", Some(Box::new(e))))
    }
}
