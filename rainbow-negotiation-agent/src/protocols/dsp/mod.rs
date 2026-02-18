#![allow(unused)]
/*
 *
 *  * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

mod errors;
pub(crate) mod facades;
pub(crate) mod http;
pub(crate) mod orchestrator;
mod persistence;
pub(crate) mod protocol_types;
pub(crate) mod validator;

use crate::entities::agreement::NegotiationAgentAgreementsTrait;
use crate::entities::negotiation_message::NegotiationAgentMessagesTrait;
use crate::entities::negotiation_process::NegotiationAgentProcessesTrait;
use crate::entities::offer::NegotiationAgentOffersTrait;
use crate::protocols::dsp::facades::FacadeService;
use crate::protocols::dsp::http::protocol::DspRouter;
use crate::protocols::dsp::http::rpc::RpcRouter;
use crate::protocols::dsp::orchestrator::orchestrator::OrchestratorService;
use crate::protocols::dsp::orchestrator::protocol::persistence::OrchestrationPersistenceForProtocol;
use crate::protocols::dsp::orchestrator::protocol::protocol::ProtocolOrchestratorService;
use crate::protocols::dsp::orchestrator::rpc::persistence::OrchestrationPersistenceForRpc;
use crate::protocols::dsp::orchestrator::rpc::rpc::RPCOrchestratorService;
use crate::protocols::dsp::persistence::persistence_protocol::NegotiationPersistenceForProtocolService;
use crate::protocols::dsp::persistence::persistence_rpc::NegotiationPersistenceForRpcService;
use crate::protocols::dsp::validator::validators::protocol::validate_state_transition::ValidatedStateTransitionServiceForDsp;
use crate::protocols::dsp::validator::validators::protocol::validation_dsp_steps::ValidationDspStepsService;
use crate::protocols::dsp::validator::validators::rpc::validate_state_transition::ValidatedStateTransitionServiceForRcp;
use crate::protocols::dsp::validator::validators::rpc::validation_rpc_steps::ValidationRpcStepsService;
use crate::protocols::dsp::validator::validators::validate_payload::ValidatePayloadService;
use crate::protocols::dsp::validator::validators::validation_helpers::ValidationHelperService;
use crate::protocols::protocol::ProtocolPluginTrait;
use axum::Router;
use rainbow_common::config::services::ContractsConfig;
use rainbow_common::facades::ssi_auth_facade::{MatesFacadeTrait, SSIAuthFacadeTrait};
use rainbow_common::http_client::HttpClient;
use std::sync::Arc;

pub struct NegotiationDSP {
    negotiation_agent_process_entities: Arc<dyn NegotiationAgentProcessesTrait>,
    negotiation_agent_message_service: Arc<dyn NegotiationAgentMessagesTrait>,
    negotiation_offer_service: Arc<dyn NegotiationAgentOffersTrait>,
    negotiation_agreement_service: Arc<dyn NegotiationAgentAgreementsTrait>,
    config: Arc<ContractsConfig>,
    ssi_auth_service: Arc<dyn SSIAuthFacadeTrait>,
    mates_service: Arc<dyn MatesFacadeTrait>,
}

impl NegotiationDSP {
    pub fn new(
        negotiation_agent_process_entities: Arc<dyn NegotiationAgentProcessesTrait>,
        negotiation_agent_message_service: Arc<dyn NegotiationAgentMessagesTrait>,
        negotiation_offer_service: Arc<dyn NegotiationAgentOffersTrait>,
        negotiation_agreement_service: Arc<dyn NegotiationAgentAgreementsTrait>,
        config: Arc<ContractsConfig>,
        ssi_auth_service: Arc<dyn SSIAuthFacadeTrait>,
        mates_service: Arc<dyn MatesFacadeTrait>,
    ) -> Self {
        Self {
            negotiation_agent_message_service,
            negotiation_agent_process_entities,
            negotiation_offer_service,
            negotiation_agreement_service,
            config,
            ssi_auth_service,
            mates_service,
        }
    }
}

#[async_trait::async_trait]
impl ProtocolPluginTrait for NegotiationDSP {
    fn name(&self) -> &'static str {
        "Dataspace Protocol"
    }

    fn version(&self) -> &'static str {
        "1.0"
    }

    fn short_name(&self) -> &'static str {
        "DSP"
    }

    async fn build_router(&self) -> anyhow::Result<Router> {
        let http_client = Arc::new(HttpClient::new(10, 10));

        // Validator
        let validator_helper = Arc::new(ValidationHelperService::new(
            self.negotiation_agent_process_entities.clone(),
        ));
        let validator_payload = Arc::new(ValidatePayloadService::new(validator_helper.clone()));
        let validator_state_machine_dsp =
            Arc::new(ValidatedStateTransitionServiceForDsp::new(validator_helper.clone()));
        let dsp_validator = Arc::new(ValidationDspStepsService::new(
            validator_payload.clone(),
            validator_state_machine_dsp.clone(),
            validator_helper.clone(),
        ));
        let validator_state_machine_rpc =
            Arc::new(ValidatedStateTransitionServiceForRcp::new(validator_helper.clone()));
        let rpc_validator = Arc::new(ValidationRpcStepsService::new(
            validator_payload.clone(),
            validator_state_machine_rpc.clone(),
            validator_helper.clone(),
        ));

        // http service
        let persistence_protocol_service = Arc::new(OrchestrationPersistenceForProtocol::new(
            self.negotiation_agent_process_entities.clone(),
            self.negotiation_agent_message_service.clone(),
            self.negotiation_offer_service.clone(),
            self.negotiation_agreement_service.clone(),
        ));
        let persistence_rpc_service = Arc::new(OrchestrationPersistenceForRpc::new(
            self.negotiation_agent_process_entities.clone(),
            self.negotiation_agent_message_service.clone(),
            self.negotiation_offer_service.clone(),
            self.negotiation_agreement_service.clone(),
        ));

        // facades
        let facades = Arc::new(FacadeService::new());

        // orchestrators
        let http_orchestator = Arc::new(ProtocolOrchestratorService::new(
            dsp_validator.clone(),
            persistence_protocol_service.clone(),
            facades.clone(),
            self.config.clone(),
        ));
        let rpc_orchestator = Arc::new(RPCOrchestratorService::new(
            rpc_validator.clone(),
            persistence_rpc_service,
            self.config.clone(),
            http_client.clone(),
            self.mates_service.clone(),
        ));
        let orchestrator_service = Arc::new(OrchestratorService::new(
            http_orchestator.clone(),
            rpc_orchestator.clone(),
        ));

        // router
        let dsp_router = DspRouter::new(
            orchestrator_service.clone(),
            self.config.clone(),
            self.ssi_auth_service.clone(),
        );
        let rcp_router = RpcRouter::new(orchestrator_service.clone(), self.config.clone());

        Ok(Router::new().merge(dsp_router.router()).merge(rcp_router.router()))
    }

    fn build_grpc_router(&self) -> anyhow::Result<Option<Router>> {
        todo!()
    }
}
