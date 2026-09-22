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

//! Negotiation agent as a composable module. Today it contributes the gRPC plane and
//! migrations; the HTTP plane still comes from `create_root_http_router_with_bus`.

use crate::SERVICE_NAME;
use crate::grpc::agreement::NegotiationAgentAgreementGrpc;
use crate::grpc::api::FILE_DESCRIPTOR_SET;
use crate::grpc::api::negotiation_agent::negotiation_agent_agreements_service_server::NegotiationAgentAgreementsServiceServer;
use crate::grpc::api::negotiation_agent::negotiation_agent_messages_service_server::NegotiationAgentMessagesServiceServer;
use crate::grpc::api::negotiation_agent::negotiation_agent_offers_service_server::NegotiationAgentOffersServiceServer;
use crate::grpc::api::negotiation_agent::negotiation_agent_processes_service_server::NegotiationAgentProcessesServiceServer;
use crate::grpc::negotiation_message::NegotiationAgentMessagesGrpc;
use crate::grpc::negotiation_process::NegotiationAgentProcessesGrpc;
use crate::grpc::offer::NegotiationAgentOfferGrpc;
use crate::setup::context::AppContext;
use common::config::services::ContractsConfig;
use common::module_loader::service_module::ServiceModuleTrait;
use sea_orm_migration::MigrationTrait;
use tonic::service::RoutesBuilder;
use ymir::errors::Outcome;
use ymir::services::vault::global::VaultService;

pub struct NegotiationAgentModule {
    ctx: AppContext,
}

impl NegotiationAgentModule {
    pub async fn compose(config: &ContractsConfig, vault: &VaultService) -> Outcome<Self> {
        Self::compose_with_bus(config, vault, None).await
    }

    pub async fn compose_with_bus(
        config: &ContractsConfig,
        vault: &VaultService,
        event_bus: Option<events::EventBus>,
    ) -> Outcome<Self> {
        Ok(Self::new(
            AppContext::build_with_bus(config, vault, event_bus).await?,
        ))
    }

    pub(crate) fn new(ctx: AppContext) -> Self {
        Self { ctx }
    }

    pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        crate::data::migrations::get_negotiation_agent_migrations()
    }
}

impl ServiceModuleTrait for NegotiationAgentModule {
    fn name(&self) -> &'static str {
        SERVICE_NAME
    }

    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        Self::migrations()
    }

    fn grpc(&self, routes: &mut RoutesBuilder) {
        let ctx = &self.ctx;
        let validator = || ctx.oauth_validator.clone();
        routes
            .add_service(NegotiationAgentProcessesServiceServer::new(
                NegotiationAgentProcessesGrpc::new(ctx.process_svc.clone(), validator()),
            ))
            .add_service(NegotiationAgentMessagesServiceServer::new(
                NegotiationAgentMessagesGrpc::new(ctx.message_svc.clone(), validator()),
            ))
            .add_service(NegotiationAgentOffersServiceServer::new(
                NegotiationAgentOfferGrpc::new(ctx.offer_svc.clone(), validator()),
            ))
            .add_service(NegotiationAgentAgreementsServiceServer::new(
                NegotiationAgentAgreementGrpc::new(ctx.agreement_svc.clone(), validator()),
            ));
    }

    fn grpc_descriptors(&self) -> Vec<&'static [u8]> {
        vec![FILE_DESCRIPTOR_SET]
    }
}
