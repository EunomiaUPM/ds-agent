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

use crate::grpc::api::FILE_DESCRIPTOR_SET;
use common::config::services::TransferConfig;
use common::config::types::traits::CommonConfigTrait;
use common::worker_utils::{bind_listener, shutdown_signal, spawn_server};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tonic::codegen::tokio_stream::wrappers::TcpListenerStream;
use tonic::service::Routes;
use tonic::transport::Server;
use ymir::config::traits::HostsConfigTrait;
use ymir::config::types::HostType;
use ymir::errors::{Errors, Outcome};

pub struct TransferGrpcWorker {}

impl TransferGrpcWorker {
    pub async fn spawn(
        config: &TransferConfig,
        routes: Routes,
        token: &CancellationToken,
    ) -> Outcome<JoinHandle<()>> {
        let reflection = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
            .build_v1()
            .map_err(|e| Errors::crazy("Error building gRPC reflection", Some(Box::new(e))))?;
        let router = Server::builder().add_routes(routes).add_service(reflection);

        let listener =
            bind_listener(config.common().get_internal_port(HostType::Grpc), "gRPC").await?;
        let incoming = TcpListenerStream::new(listener);
        let server =
            router.serve_with_incoming_shutdown(incoming, shutdown_signal(token.clone(), "gRPC"));
        Ok(spawn_server("gRPC", server))
    }
}
