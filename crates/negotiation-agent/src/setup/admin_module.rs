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

//! Management plane of the negotiation agent: the `{api}/negotiation-agent` HTTP API and its
//! gRPC mirror.

use std::sync::Arc;

use axum::Router;
use common::config::types::traits::CommonConfigTrait;
use common::module_loader::service_module::ServiceModuleTrait;
use tonic::service::RoutesBuilder;
use ymir::config::traits::ApiConfigTrait;

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
use crate::http::agreement::NegotiationAgentAgreementsRouter;
use crate::http::negotiation_message::NegotiationAgentMessagesRouter;
use crate::http::negotiation_process::NegotiationAgentProcessesRouter;
use crate::http::offer::NegotiationAgentOffersRouter;
use crate::setup::context::AppContext;

pub(crate) struct NegotiationAdminModule {
    ctx: Arc<AppContext>,
}

impl NegotiationAdminModule {
    pub(crate) fn new(ctx: Arc<AppContext>) -> Self {
        Self { ctx }
    }

    /// Mount prefix of this agent's own API, e.g. `/api/v1/negotiation-agent`.
    fn base_path(&self) -> String {
        format!(
            "{}/{SERVICE_NAME}",
            self.ctx.config.common().get_api_version()
        )
    }
}

impl ServiceModuleTrait for NegotiationAdminModule {
    fn name(&self) -> &'static str {
        "negotiation-admin"
    }

    fn http(&self) -> Option<(String, Router)> {
        let ctx = &self.ctx;
        let router = Router::new()
            .nest(
                "/negotiation-messages",
                NegotiationAgentMessagesRouter::new(ctx.message_svc.clone()).router(),
            )
            .nest(
                "/negotiation-processes",
                NegotiationAgentProcessesRouter::new(ctx.process_svc.clone()).router(),
            )
            .nest(
                "/offers",
                NegotiationAgentOffersRouter::new(ctx.offer_svc.clone()).router(),
            )
            .nest(
                "/agreements",
                NegotiationAgentAgreementsRouter::new(ctx.agreement_svc.clone()).router(),
            )
            .route_layer(axum::middleware::from_fn_with_state(
                ctx.oauth_validator.clone(),
                common::auth::http::AuthHttpMiddleware::run,
            ));
        Some((self.base_path(), router))
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
