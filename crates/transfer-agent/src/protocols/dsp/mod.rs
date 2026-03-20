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

pub(crate) mod facades;
pub(crate) mod http;
pub(crate) mod orchestrator;
mod persistence;
pub(crate) mod protocol_types;
pub(crate) mod transfer_types;
pub(crate) mod validator;

use crate::entities::transfer_messages::TransferAgentMessagesTrait;
use crate::entities::transfer_process::TransferAgentProcessesTrait;
// use crate::protocols::dsp::facades::data_plane_facade::data_plane_facade::DataPlaneProviderFacadeForDSProtocol;
// use crate::protocols::dsp::facades::data_plane_facade::dataplane_strategy_factory::DataPlaneStrategyFactory;
use crate::protocols::dsp::facades::data_service_resolver_facade::data_service_resolver_facade::DataServiceFacadeServiceForDSProtocol;
use crate::protocols::dsp::facades::FacadeService;
use crate::protocols::dsp::http::protocol::DspRouter;
use crate::protocols::dsp::http::rpc::RpcRouter;
use crate::protocols::dsp::orchestrator::orchestrator::OrchestratorService;
use crate::protocols::dsp::orchestrator::protocol::protocol::ProtocolOrchestratorService;
use crate::protocols::dsp::orchestrator::rpc::rpc::RPCOrchestratorService;
use crate::protocols::dsp::persistence::persistence_protocol::TransferPersistenceForProtocolService;
use crate::protocols::dsp::persistence::persistence_rpc::TransferPersistenceForRpcService;
use crate::protocols::dsp::validator::validators::protocol::validation_dsp_steps::ValidationDspStepsService;
use crate::protocols::dsp::validator::validators::rpc::validate_state_transition::ValidatedStateTransitionServiceForRcp;
use crate::protocols::dsp::validator::validators::validate_payload::ValidatePayloadService;
use crate::protocols::dsp::validator::validators::validation_helpers::ValidationHelperService;

use crate::protocols::dsp::facades::dataplane_facade::dataplane_facade::DspDataPlaneFacade;
use crate::protocols::protocol::ProtocolPluginTrait;
use axum::Router;
use common::config::services::traits::TransferConfigTrait;
use common::config::services::TransferConfig;
use common::config::types::traits::CommonConfigTrait;
use common::facades::ssi_auth_facade::ssi_auth_facade::SSIAuthFacadeService;
use common::facades::ssi_auth_facade::MatesFacadeTrait;
use common::http_client::HttpClient;
use connector::ConnectorSetup;
use dataplane::setup::DataplaneSetup;
use std::sync::Arc;
use validator::validators::protocol::validate_state_transition::ValidatedStateTransitionServiceForDsp;
use validator::validators::rpc::validation_rpc_steps::ValidationRpcStepsService;
use ymir::config::traits::HostsConfigTrait;
use ymir::errors::Outcome;
use ymir::services::vault::global::VaultService;

pub struct TransferDSP {
    transfer_agent_process_entities: Arc<dyn TransferAgentProcessesTrait>,
    transfer_agent_message_service: Arc<dyn TransferAgentMessagesTrait>,
    config: Arc<TransferConfig>,
    vault: Arc<VaultService>,
    mates_facade: Arc<dyn MatesFacadeTrait>,
}

impl TransferDSP {
    pub fn new(
        transfer_agent_message_service: Arc<dyn TransferAgentMessagesTrait>,
        transfer_agent_process_entities: Arc<dyn TransferAgentProcessesTrait>,
        config: Arc<TransferConfig>,
        vault: Arc<VaultService>,
        mates_facade: Arc<dyn MatesFacadeTrait>,
    ) -> Self {
        Self {
            transfer_agent_message_service,
            transfer_agent_process_entities,
            config,
            vault,
            mates_facade,
        }
    }
}

#[async_trait::async_trait]
impl ProtocolPluginTrait for TransferDSP {
    fn name(&self) -> &'static str {
        "Dataspace Protocol"
    }

    fn version(&self) -> &'static str {
        "1.0"
    }

    fn short_name(&self) -> &'static str {
        "DSP"
    }

    async fn build_router(&self) -> Outcome<Router> {
        let http_client = Arc::new(HttpClient::new(10, 10));

        // Validator
        let validator_helper = Arc::new(ValidationHelperService::new(
            self.transfer_agent_process_entities.clone(),
        ));
        let validator_payload = Arc::new(ValidatePayloadService::new(validator_helper.clone()));
        let validator_state_machine_dsp = Arc::new(ValidatedStateTransitionServiceForDsp::new(
            validator_helper.clone(),
        ));
        let dsp_validator = Arc::new(ValidationDspStepsService::new(
            validator_payload.clone(),
            validator_state_machine_dsp.clone(),
            validator_helper.clone(),
        ));
        let validator_state_machine_rcp = Arc::new(ValidatedStateTransitionServiceForRcp::new(
            validator_helper.clone(),
        ));
        let rcp_validator = Arc::new(ValidationRpcStepsService::new(
            validator_payload.clone(),
            validator_state_machine_rcp.clone(),
            validator_helper.clone(),
        ));

        // http service
        let persistence_protocol_service = Arc::new(TransferPersistenceForProtocolService::new(
            self.transfer_agent_message_service.clone(),
            self.transfer_agent_process_entities.clone(),
        ));
        let persistence_rpc_service = Arc::new(TransferPersistenceForRpcService::new(
            self.transfer_agent_message_service.clone(),
            self.transfer_agent_process_entities.clone(),
        ));

        // connector
        let connector_setup = ConnectorSetup::new();
        let connector_entity = connector_setup
            .get_connector_instance_entity(
                self.config.as_ref(),
                self.vault.clone(),
                http_client.clone(),
            )
            .await;

        // dataplane
        let dataplane = DataplaneSetup::new();
        let dataplane_manager = Arc::new(
            dataplane
                .get_data_plane_manager(
                    self.config.clone(),
                    self.vault.clone(),
                    connector_entity.clone(),
                    http_client.clone(),
                )
                .await,
        );
        let http = self.config.common().http();
        let proxy_base_url = match &http.port {
            Some(port) => format!("{}://{}:{}", http.protocol, http.url, port),
            None => format!("{}://{}", http.protocol, http.url),
        };
        let dataplane_facade = Arc::new(DspDataPlaneFacade::new(
            dataplane_manager.clone(),
            proxy_base_url,
        ));

        // data service resolver (resolves agreement → dataset → distribution → connector)
        let data_service_resolver = Arc::new(DataServiceFacadeServiceForDSProtocol::new(
            self.config.clone(),
            http_client.clone(),
            connector_entity,
        ));

        // facades
        let facades = Arc::new(FacadeService::new(
            data_service_resolver.clone(),
            dataplane_facade.clone(),
        ));

        // orchestrators
        let http_orchestator = Arc::new(ProtocolOrchestratorService::new(
            dsp_validator.clone(),
            persistence_protocol_service.clone(),
            facades.clone(),
        ));
        let rpc_orchestator = Arc::new(RPCOrchestratorService::new(
            rcp_validator.clone(),
            persistence_rpc_service,
            http_client.clone(),
            facades.clone(),
            self.mates_facade.clone(),
        ));
        let orchestrator_service = Arc::new(OrchestratorService::new(
            http_orchestator.clone(),
            rpc_orchestator.clone(),
        ));

        // router
        // ssi auth
        let ssi_auth = Arc::new(SSIAuthFacadeService::new(
            Arc::new(self.config.ssi_auth().clone()),
            http_client,
        ));
        let dsp_router =
            DspRouter::new(orchestrator_service.clone(), self.config.clone(), ssi_auth);
        let rcp_router = RpcRouter::new(orchestrator_service.clone());

        Ok(Router::new()
            .merge(dsp_router.router())
            .merge(rcp_router.router()))
    }

    fn build_grpc_router(&self) -> Outcome<Option<Router>> {
        todo!()
    }
}
