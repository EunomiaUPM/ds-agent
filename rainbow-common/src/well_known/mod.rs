use crate::config::types::min_known_config::MinKnownConfig;
use crate::facades::ssi_auth_facade::mates_facade::MatesFacadeService;
use crate::http_client::HttpClient;
use crate::well_known::dspace_version::dspace_version::WellKnownDSpaceVersionService;
use crate::well_known::router::WellKnownRouter;
use crate::well_known::rpc::rpc::WellKnownRPCService;
use std::sync::Arc;
use ymir::errors::Outcome;

pub mod dspace_version;
pub mod router;
pub mod rpc;

pub struct WellKnownRoot;
impl WellKnownRoot {
    pub fn get_well_known_router(config: &MinKnownConfig) -> Outcome<axum::Router> {
        let config = Arc::new(config.clone());
        let http_client = Arc::new(HttpClient::new(2, 3));
        let mates_facade = Arc::new(MatesFacadeService::new(config.clone(), http_client.clone()));

        let dspace_version_service = WellKnownDSpaceVersionService::new();
        let dspace_version_rpc =
            Arc::new(WellKnownRPCService::new(http_client.clone(), mates_facade.clone()));
        let router = WellKnownRouter::new(dspace_version_service, dspace_version_rpc.clone());
        Ok(router.router())
    }
}
