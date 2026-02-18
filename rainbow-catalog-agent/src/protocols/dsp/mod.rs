use crate::entities::catalogs::CatalogEntityTrait;
use crate::entities::data_services::DataServiceEntityTrait;
use crate::entities::datasets::DatasetEntityTrait;
use crate::entities::distributions::DistributionEntityTrait;
use crate::entities::odrl_policies::OdrlPolicyEntityTrait;
use crate::entities::peer_catalogs::PeerCatalogTrait;
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
use axum::Router;
use rainbow_common::config::services::CatalogConfig;
use rainbow_common::facades::ssi_auth_facade::ssi_auth_facade::SSIAuthFacadeService;
use rainbow_common::facades::ssi_auth_facade::MatesFacadeTrait;
use rainbow_common::http_client::HttpClient;
use std::sync::Arc;

mod errors;
pub(crate) mod facades;
pub(crate) mod http;
pub(crate) mod orchestrator;
pub(crate) mod protocol_types;
pub(crate) mod types;
pub(crate) mod validator;

pub struct CatalogDSP {
    pub catalog_entities_service: Arc<dyn CatalogEntityTrait>,
    pub data_service_entities_service: Arc<dyn DataServiceEntityTrait>,
    pub dataset_entities_service: Arc<dyn DatasetEntityTrait>,
    pub odrl_policies_service: Arc<dyn OdrlPolicyEntityTrait>,
    pub distributions_entity_service: Arc<dyn DistributionEntityTrait>,
    pub peer_catalog_entity_service: Arc<dyn PeerCatalogTrait>,
    pub mates_facade: Arc<dyn MatesFacadeTrait>,
    config: Arc<CatalogConfig>,
}

impl CatalogDSP {
    pub fn new(
        catalog_entities_service: Arc<dyn CatalogEntityTrait>,
        data_service_entities_service: Arc<dyn DataServiceEntityTrait>,
        dataset_entities_service: Arc<dyn DatasetEntityTrait>,
        odrl_policies_service: Arc<dyn OdrlPolicyEntityTrait>,
        distributions_entity_service: Arc<dyn DistributionEntityTrait>,
        peer_catalog_entity_service: Arc<dyn PeerCatalogTrait>,
        mates_facade: Arc<dyn MatesFacadeTrait>,
        config: Arc<CatalogConfig>,
    ) -> Self {
        Self {
            catalog_entities_service,
            data_service_entities_service,
            dataset_entities_service,
            odrl_policies_service,
            distributions_entity_service,
            peer_catalog_entity_service,
            mates_facade,
            config: config,
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

    async fn build_router(&self) -> anyhow::Result<Router> {
        // http
        let http_client = Arc::new(HttpClient::new(10, 3));

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
            self.config.clone(),
            http_client.clone(),
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
            http_client.clone(),
            facades.clone(),
            rpc_persistence.clone(),
            self.mates_facade.clone(),
        ));
        let orchestrator_service = Arc::new(OrchestratorService::new(
            dsp_orchestator.clone(),
            rpc_orchestrator.clone(),
        ));

        // router
        let ssi_auth = Arc::new(SSIAuthFacadeService::new(
            Arc::new(self.config.ssi_auth().clone()),
            http_client.clone(),
        ));
        let dsp_router =
            DspRouter::new(orchestrator_service.clone(), self.config.clone(), ssi_auth);
        let rpc_router = RpcRouter::new(orchestrator_service.clone());

        Ok(Router::new().merge(dsp_router.router()).merge(rpc_router.router()))
    }

    fn build_grpc_router(&self) -> anyhow::Result<Option<Router>> {
        todo!()
    }
}
