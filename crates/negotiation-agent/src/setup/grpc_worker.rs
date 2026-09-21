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

use crate::data::factory_sql::NegotiationAgentRepoForSql;
use crate::data::factory_trait::NegotiationAgentRepoTrait;
use crate::grpc::agreement::NegotiationAgentAgreementGrpc;
use crate::grpc::api::FILE_DESCRIPTOR_SET;
use crate::grpc::api::negotiation_agent::negotiation_agent_agreements_service_server::NegotiationAgentAgreementsServiceServer;
use crate::grpc::api::negotiation_agent::negotiation_agent_messages_service_server::NegotiationAgentMessagesServiceServer;
use crate::grpc::api::negotiation_agent::negotiation_agent_offers_service_server::NegotiationAgentOffersServiceServer;
use crate::grpc::api::negotiation_agent::negotiation_agent_processes_service_server::NegotiationAgentProcessesServiceServer;
use crate::grpc::negotiation_message::NegotiationAgentMessagesGrpc;
use crate::grpc::negotiation_process::NegotiationAgentProcessesGrpc;
use crate::grpc::offer::NegotiationAgentOfferGrpc;
use crate::services::agreement::service::AgreementService;
use crate::services::negotiation_message::service::NegotiationMessageService;
use crate::services::negotiation_process::service::NegotiationProcessService;
use crate::services::offer::service::OfferService;
use common::config::services::ContractsConfig;
use common::config::types::traits::CommonConfigTrait;
use sea_orm::Database;
use std::sync::Arc;
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

pub struct NegotiationGrpcWorker {}

impl NegotiationGrpcWorker {
    pub async fn spawn(
        config: &ContractsConfig,
        vault: Arc<VaultService>,
        token: &CancellationToken,
    ) -> Outcome<JoinHandle<()>> {
        let router = Self::create_root_grpc_router(config, vault.clone()).await?;

        let port = config.common().get_internal_port(HostType::Grpc);
        let grpc_port = format!("{port}1");
        let addr = format!("0.0.0.0:{grpc_port}");

        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| Errors::crazy("Error listening on the socket", Some(Box::new(e))))?;
        let incoming = TcpListenerStream::new(listener);
        tracing::info!("GRPC Negotiation Service running on {}", addr);

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
        config: &ContractsConfig,
        vault: Arc<VaultService>,
    ) -> Outcome<tonic::transport::server::Router> {
        let db_connection = vault.get_db_connection(config.common()).await?;
        let config = Arc::new(config.clone());
        let negotiation_repo = Arc::new(NegotiationAgentRepoForSql::create_repo(
            db_connection.clone(),
        ));

        let validator: Arc<dyn common::auth::OauthTokenValidator> =
            oauth::setup::composition::OAuthSetup::new()
                .build_token_service(config.common().clone().into(), db_connection.clone());

        let process_service = Arc::new(NegotiationProcessService::new(
            negotiation_repo.get_negotiation_process_repo(),
            negotiation_repo.get_negotiation_process_identifiers_repo(),
            negotiation_repo.get_negotiation_message_repo(),
            negotiation_repo.get_offer_repo(),
            negotiation_repo.get_agreement_repo(),
        ));
        let processes_controller =
            NegotiationAgentProcessesGrpc::new(process_service, validator.clone());

        let message_service = Arc::new(NegotiationMessageService::new(
            negotiation_repo.get_negotiation_message_repo(),
            negotiation_repo.get_offer_repo(),
            negotiation_repo.get_agreement_repo(),
        ));
        let message_controller =
            NegotiationAgentMessagesGrpc::new(message_service, validator.clone());

        let offer_service = Arc::new(OfferService::new(negotiation_repo.get_offer_repo()));
        let offer_controller = NegotiationAgentOfferGrpc::new(offer_service, validator.clone());

        let agreement_service =
            Arc::new(AgreementService::new(negotiation_repo.get_agreement_repo()));
        let agreement_controller =
            NegotiationAgentAgreementGrpc::new(agreement_service, validator.clone());

        let reflection_service = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
            .build_v1()
            .map_err(|e| Errors::crazy("Error building gRPC server", Some(Box::new(e))))?;

        let router = Server::builder()
            .add_service(reflection_service)
            .add_service(NegotiationAgentProcessesServiceServer::new(
                processes_controller,
            ))
            .add_service(NegotiationAgentMessagesServiceServer::new(
                message_controller,
            ))
            .add_service(NegotiationAgentOffersServiceServer::new(offer_controller))
            .add_service(NegotiationAgentAgreementsServiceServer::new(
                agreement_controller,
            ));

        Ok(router)
    }
}
