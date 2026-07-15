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

use common::config::services::TransferConfig;
use common::config::types::traits::CommonConfigTrait;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tonic::codegen::tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;
use ymir::config::traits::HostsConfigTrait;
use ymir::config::types::HostType;
use ymir::errors::{Errors, Outcome};
use ymir::services::vault::global::VaultService;

use crate::grpc::api::FILE_DESCRIPTOR_SET;
use crate::grpc::api::transfer_messages::transfer_messages_ref_server::TransferMessagesRefServer;
use crate::grpc::api::transfer_processes::transfer_processes_ref_server::TransferProcessesRefServer;
use crate::grpc::transfer_messages::TransferMessagesGrpc;
use crate::grpc::transfer_process::TransferProcessGrpc;
use crate::setup::common_worker::{
    bind_listener, build_domain_services, shutdown_signal, spawn_server,
};

const SERVICE: &str = "gRPC";

pub struct TransferGrpcWorker {}

impl TransferGrpcWorker {
    pub async fn spawn(
        config: &TransferConfig,
        vault: Arc<VaultService>,
        token: &CancellationToken,
    ) -> Outcome<JoinHandle<()>> {
        let router = Self::create_root_grpc_router(config, vault).await?;
        let listener =
            bind_listener(config.common().get_internal_port(HostType::Grpc), SERVICE).await?;
        let incoming = TcpListenerStream::new(listener);
        let server =
            router.serve_with_incoming_shutdown(incoming, shutdown_signal(token.clone(), SERVICE));
        Ok(spawn_server(SERVICE, server))
    }

    pub async fn create_root_grpc_router(
        config: &TransferConfig,
        vault: Arc<VaultService>,
    ) -> Outcome<tonic::transport::server::Router> {
        let svc = build_domain_services(config, &vault).await?;
        let process_handler = TransferProcessGrpc::new(svc.process, svc.validator.clone());
        let message_handler = TransferMessagesGrpc::new(svc.message, svc.validator);
        let reflection = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
            .build_v1()
            .map_err(|e| Errors::crazy("Error building gRPC reflection", Some(Box::new(e))))?;
        let router = Server::builder()
            .add_service(reflection)
            .add_service(TransferProcessesRefServer::new(process_handler))
            .add_service(TransferMessagesRefServer::new(message_handler));
        Ok(router)
    }
}
