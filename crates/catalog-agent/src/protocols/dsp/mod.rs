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

use crate::protocols::dsp::facades::well_known_rpc_facade::well_known_rpc_facade::WellKnownRPCFacadeForDSProtocol;
use crate::protocols::dsp::facades::FacadeService;
use crate::protocols::dsp::http::protocol::DspRouter;
use crate::protocols::dsp::http::rpc::RpcRouter;
use crate::protocols::dsp::orchestrator::orchestrator::OrchestratorService;
use crate::protocols::dsp::orchestrator::protocol::persistence::OrchestrationPersistenceForProtocol;
use crate::protocols::dsp::orchestrator::protocol::protocol::ProtocolOrchestratorService;
use crate::protocols::dsp::orchestrator::rpc::persistence::OrchestrationPersistenceForProtocolForRPC;
use crate::protocols::dsp::orchestrator::rpc::rpc::RPCOrchestratorService;
use crate::protocols::dsp::validator::validators::protocol::validation_dsp_steps::ValidationDspStepsService;
use crate::protocols::dsp::validator::validators::rpc::validation_rpc_steps::ValidationRpcStepsService;
use crate::protocols::dsp::validator::validators::validate_payload::ValidatePayloadService;
use crate::protocols::dsp::validator::validators::validation_helpers::ValidationHelperService;
use crate::protocols::protocol::ProtocolPluginTrait;
use crate::services::catalogs::CatalogServiceTrait;
use crate::services::data_services::DataServiceServiceTrait;
use crate::services::datasets::DatasetServiceTrait;
use crate::services::distributions::DistributionServiceTrait;
use crate::services::odrl_policies::OdrlPolicyServiceTrait;
use crate::services::peer_catalogs::PeerCatalogServiceTrait;
use axum::Router;
use common::auth::http::AuthHttpMiddleware;
use common::auth::OauthTokenValidator;
use common::config::services::CatalogConfig;
use common::facades::mates_facade::MatesFacadeTrait;
use common::facades::ssi_auth_facade::SSIAuthFacadeTrait;
use common::well_known::rpc::rpc::WellKnownRPCService;
use std::sync::Arc;
use ymir::errors::Outcome;

mod errors;
pub(crate) mod facades;
pub(crate) mod http;
pub(crate) mod orchestrator;
pub(crate) mod protocol_types;
pub(crate) mod setup;
pub(crate) mod types;
pub(crate) mod validator;

pub struct CatalogDSP {
    pub catalog_entities_service: Arc<dyn CatalogServiceTrait>,
    pub data_service_entities_service: Arc<dyn DataServiceServiceTrait>,
    pub dataset_entities_service: Arc<dyn DatasetServiceTrait>,
    pub odrl_policies_service: Arc<dyn OdrlPolicyServiceTrait>,
    pub distributions_entity_service: Arc<dyn DistributionServiceTrait>,
    pub peer_catalog_entity_service: Arc<dyn PeerCatalogServiceTrait>,
    pub mates_facade: Arc<dyn MatesFacadeTrait>,
    ssi_auth_facade: Arc<dyn SSIAuthFacadeTrait>,
    config: Arc<CatalogConfig>,
    validator: Arc<dyn OauthTokenValidator>,
}

impl CatalogDSP {
    pub fn new(
        catalog_entities_service: Arc<dyn CatalogServiceTrait>,
        data_service_entities_service: Arc<dyn DataServiceServiceTrait>,
        dataset_entities_service: Arc<dyn DatasetServiceTrait>,
        odrl_policies_service: Arc<dyn OdrlPolicyServiceTrait>,
        distributions_entity_service: Arc<dyn DistributionServiceTrait>,
        peer_catalog_entity_service: Arc<dyn PeerCatalogServiceTrait>,
        mates_facade: Arc<dyn MatesFacadeTrait>,
        ssi_auth_facade: Arc<dyn SSIAuthFacadeTrait>,
        config: Arc<CatalogConfig>,
        validator: Arc<dyn OauthTokenValidator>,
    ) -> Self {
        Self {
            catalog_entities_service,
            data_service_entities_service,
            dataset_entities_service,
            odrl_policies_service,
            distributions_entity_service,
            peer_catalog_entity_service,
            mates_facade,
            ssi_auth_facade,
            config,
            validator,
        }
    }
}

#[async_trait::async_trait]
impl ProtocolPluginTrait for CatalogDSP {
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
        // Validator
        let validator_helper = Arc::new(ValidationHelperService::new());
        let validator_payload = Arc::new(ValidatePayloadService::new(validator_helper.clone()));
        let dsp_validator = Arc::new(ValidationDspStepsService::new(
            validator_payload.clone(),
            validator_helper.clone(),
        ));
        let rpc_validation = Arc::new(ValidationRpcStepsService::new(
            validator_payload.clone(),
            validator_helper.clone(),
        ));

        // facades
        let catalog_well_known_rpc_facade = Arc::new(WellKnownRPCFacadeForDSProtocol::new(
            Arc::new(WellKnownRPCService::new(self.mates_facade.clone())),
        ));
        let facades = Arc::new(FacadeService::new(catalog_well_known_rpc_facade.clone()));

        // persistence
        let dsp_persistence = Arc::new(OrchestrationPersistenceForProtocol::new(
            self.catalog_entities_service.clone(),
            self.data_service_entities_service.clone(),
            self.dataset_entities_service.clone(),
            self.odrl_policies_service.clone(),
            self.distributions_entity_service.clone(),
        ));
        let rpc_persistence = Arc::new(OrchestrationPersistenceForProtocolForRPC::new(
            self.peer_catalog_entity_service.clone(),
        ));

        // orchestrators
        let dsp_orchestator = Arc::new(ProtocolOrchestratorService::new(
            dsp_validator.clone(),
            facades.clone(),
            dsp_persistence.clone(),
        ));
        let rpc_orchestrator = Arc::new(RPCOrchestratorService::new(
            rpc_validation.clone(),
            facades.clone(),
            rpc_persistence.clone(),
            self.mates_facade.clone(),
        ));
        let orchestrator_service = Arc::new(OrchestratorService::new(
            dsp_orchestator.clone(),
            rpc_orchestrator.clone(),
        ));

        // router
        let dsp_router = DspRouter::new(
            orchestrator_service.clone(),
            self.config.clone(),
            self.ssi_auth_facade.clone(),
        );
        let rpc_router = RpcRouter::new(orchestrator_service.clone())
            .router()
            .route_layer(axum::middleware::from_fn_with_state(
                self.validator.clone(),
                AuthHttpMiddleware::run,
            ));

        Ok(Router::new().merge(dsp_router.router()).merge(rpc_router))
    }

    fn build_grpc_router(&self) -> Outcome<Option<Router>> {
        todo!()
    }
}
