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
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tonic::codegen::tokio_stream::wrappers::TcpListenerStream;
use tonic::service::Routes;
use tonic::transport::Server;
use ymir::errors::{Errors, Outcome};

/// Tonic server hosting a composer's gRPC routes plus a v1 reflection service.
pub struct GrpcServer;

impl GrpcServer {
    /// Serves `routes` on `0.0.0.0:<port>` until `token` is cancelled.
    pub async fn spawn(
        port: String,
        routes: Routes,
        descriptors: Vec<&'static [u8]>,
        token: &CancellationToken,
    ) -> Outcome<JoinHandle<()>> {
        let mut reflection = tonic_reflection::server::Builder::configure();
        for descriptor in descriptors {
            reflection = reflection.register_encoded_file_descriptor_set(descriptor);
        }
        let reflection = reflection
            .build_v1()
            .map_err(|e| Errors::crazy("Error building gRPC reflection", Some(Box::new(e))))?;

        let router = Server::builder().add_routes(routes).add_service(reflection);
        let incoming = TcpListenerStream::new(bind_listener(port, "gRPC").await?);
        let server =
            router.serve_with_incoming_shutdown(incoming, shutdown_signal(token.clone(), "gRPC"));
        Ok(spawn_server("gRPC", server))
    }

    /// Resolves when an optional server task ends; pends forever when no server was spawned.
    pub async fn supervise(handle: Option<JoinHandle<()>>) {
        match handle {
            Some(handle) => {
                let _ = handle.await;
            }
            None => std::future::pending().await,
        }
    }
}

/// Binds `0.0.0.0:<port>`, logging under `service` (e.g. "HTTP", "gRPC").
pub async fn bind_listener(port: String, service: &str) -> Outcome<TcpListener> {
    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr)
        .await
        .map_err(|e| Errors::crazy("Error binding socket", Some(Box::new(e))))?;
    tracing::info!("{service} service running on {addr}");
    Ok(listener)
}

/// Resolves when `token` is cancelled; the drain future passed to the servers.
pub async fn shutdown_signal(token: CancellationToken, service: &'static str) {
    token.cancelled().await;
    tracing::info!("{service} Service received shutdown signal, draining connections...");
}

/// Spawns the server (any `IntoFuture` resolving to `Result`) and logs its
/// clean stop or crash. Covers both axum's `WithGracefulShutdown` and tonic's
/// server future.
pub fn spawn_server<F, E>(service: &'static str, server: F) -> JoinHandle<()>
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
