/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

use std::sync::Arc;

use common::config::services::TransferConfig;
use common::config::types::traits::CommonConfigTrait;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tonic::codegen::tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;
use ymir::config::traits::{ConnectionConfigTrait, HostsConfigTrait};
use ymir::config::types::HostType;
use ymir::errors::{Errors, Outcome};
use ymir::services::vault::VaultTrait;
use ymir::services::vault::global::VaultService;

use crate::data::factory::DataFactory;
use crate::data::sea_orm::factory::SeaOrmDataFactory;
use crate::grpc::api::FILE_DESCRIPTOR_SET;
use crate::grpc::api::transfer_messages::transfer_messages_ref_server::TransferMessagesRefServer;
use crate::grpc::api::transfer_processes::transfer_processes_ref_server::TransferProcessesRefServer;
use crate::grpc::transfer_messages::TransferMessagesGrpc;
use crate::grpc::transfer_process::TransferProcessGrpc;
use crate::services::transfer_message::service::TransferMessageService;
use crate::services::transfer_process::service::TransferProcessService;

pub struct TransferGrpcWorker {}

impl TransferGrpcWorker {
    pub async fn spawn(
        config: &TransferConfig,
        vault: Arc<VaultService>,
        token: &CancellationToken,
    ) -> Outcome<JoinHandle<()>> {
        let router = Self::create_root_grpc_router(config, vault).await?;

        let host = if config.common().is_local() { "127.0.0.1" } else { "0.0.0.0" };
        let port = config.common().get_internal_port(HostType::Grpc);
        let addr = format!("{host}:{port}");

        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| Errors::crazy("Error binding gRPC socket", Some(Box::new(e))))?;
        let incoming = TcpListenerStream::new(listener);
        tracing::info!("gRPC Transfer-Agent-Ref Service running on {}", addr);

        let token = token.clone();
        let handle = tokio::spawn(async move {
            let server = router.serve_with_incoming_shutdown(incoming, async move {
                token.cancelled().await;
                tracing::info!("gRPC Service received shutdown signal, draining connections...");
            });
            match server.await {
                Ok(_) => tracing::info!("gRPC Service stopped successfully"),
                Err(e) => tracing::error!("gRPC Service crashed: {}", e),
            }
        });

        Ok(handle)
    }

    pub async fn create_root_grpc_router(
        config: &TransferConfig,
        vault: Arc<VaultService>,
    ) -> Outcome<tonic::transport::server::Router> {
        let db = vault.get_db_connection(config.common()).await;
        let factory = SeaOrmDataFactory::new(db);

        let validator = oauth::setup::OAuthSetup::new().build_validator(config.jwt_secret());

        let process_svc = Arc::new(TransferProcessService::new(
            factory.transfer_process_repo(),
            factory.transfer_identifier_repo(),
        ));
        let message_svc = Arc::new(TransferMessageService::new(factory.transfer_message_repo()));

        let process_handler = TransferProcessGrpc::new(process_svc, validator.clone());
        let message_handler = TransferMessagesGrpc::new(message_svc, validator);

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
