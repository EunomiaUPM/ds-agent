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

use crate::data::factory_sql::TransferAgentRepoForSql;
use crate::entities::transfer_messages::transfer_messages::TransferAgentMessagesService;
use crate::entities::transfer_process::transfer_process::TransferAgentProcessesService;
use crate::grpc::api::transfer_messages::transfer_agent_messages_server::TransferAgentMessagesServer;
use crate::grpc::api::transfer_processes::transfer_agent_processes_server::TransferAgentProcessesServer;
use crate::grpc::api::FILE_DESCRIPTOR_SET;
use crate::grpc::transfer_messages::TransferAgentMessagesGrpc;
use crate::grpc::transfer_process::TransferAgentProcessesGrpc;
use common::config::services::TransferConfig;
use common::config::types::traits::CommonConfigTrait;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tonic::codegen::tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;
use ymir::config::traits::{ConnectionConfigTrait, HostsConfigTrait};
use ymir::config::types::HostType;
use ymir::errors::{Errors, Outcome};
use ymir::services::vault::global::VaultService;
use ymir::services::vault::VaultTrait;

pub struct TransferGrpcWorker {}

impl TransferGrpcWorker {
    pub async fn spawn(
        config: &TransferConfig,
        vault: Arc<VaultService>,
        token: &CancellationToken,
    ) -> Outcome<JoinHandle<()>> {
        let router = Self::create_root_grpc_router(&config, vault.clone()).await?;
        let host = if config.common().is_local() {
            "127.0.0.1"
        } else {
            "0.0.0.0"
        };
        let port = config.common().get_internal_port(HostType::Grpc);
        let addr = format!("{}{}", host, port);

        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| Errors::crazy("Error listening on the socket", Some(Box::new(e))))?;
        let incoming = TcpListenerStream::new(listener);
        tracing::info!("GRPC Transfer Service running on {}", addr);

        let token = token.clone();
        let handle = tokio::spawn(async move {
            let server = router.serve_with_incoming_shutdown(incoming, async move {
                token.cancelled().await;
                tracing::info!("GRPC Service received shutdown signal, draining connections...");
            });
            match server.await {
                Ok(_) => tracing::info!("GRPC Service stopped successfully"),
                Err(e) => tracing::error!("GRPC Service crashed: {}", e),
            }
        });

        Ok(handle)
    }
    pub async fn create_root_grpc_router(
        config: &TransferConfig,
        vault: Arc<VaultService>,
    ) -> Outcome<tonic::transport::server::Router> {
        let db_connection = vault.get_db_connection(config.common()).await;
        let transfer_repo = Arc::new(TransferAgentRepoForSql::create_repo(db_connection.clone()));

        let messages_service = Arc::new(TransferAgentMessagesService::new(transfer_repo.clone()));
        let messages_controller = TransferAgentMessagesGrpc::new(messages_service);
        let processes_service = Arc::new(TransferAgentProcessesService::new(transfer_repo.clone()));
        let processes_controller = TransferAgentProcessesGrpc::new(processes_service);

        let reflection_service = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
            .build_v1()
            .map_err(|e| Errors::crazy("Error building gRPC server", Some(Box::new(e))))?;

        let router = Server::builder()
            .add_service(reflection_service)
            .add_service(TransferAgentProcessesServer::new(processes_controller))
            .add_service(TransferAgentMessagesServer::new(messages_controller));

        Ok(router)
    }
}
