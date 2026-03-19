use crate::protocols::dsp::facades::well_known_rpc_facade::WellKnownRPCFacadeTrait;
use rainbow_common::config::services::CatalogConfig;
use rainbow_common::config::types::traits::CommonConfigTrait;
use rainbow_common::dsp_common::well_known_types::{Version, VersionPath};
use rainbow_common::http_client::HttpClient;
use rainbow_common::well_known::rpc::WellKnownRPCRequest;
use std::sync::Arc;
use ymir::config::traits::HostsConfigTrait;
use ymir::config::types::HostType;

const RPC_WELL_KNOWN_PATH: &str = "/rpc/.well-known/dspace-version/path";

pub struct WellKnownRPCFacadeForDSProtocol {
    config: Arc<CatalogConfig>,
    client: Arc<HttpClient>,
}

impl WellKnownRPCFacadeForDSProtocol {
    pub fn new(config: Arc<CatalogConfig>, client: Arc<HttpClient>) -> Self {
        Self { config, client }
    }
}

#[async_trait::async_trait]
impl WellKnownRPCFacadeTrait for WellKnownRPCFacadeForDSProtocol {
    async fn resolve_dataspace_current_path(
        &self,
        input: &WellKnownRPCRequest,
    ) -> anyhow::Result<String> {
        let host = self.config.common().get_host(HostType::Http);
        let url = format!("{}{}", host, RPC_WELL_KNOWN_PATH);
        let provider_address =
            self.client.post_json::<WellKnownRPCRequest, VersionPath>(&url, input).await?;
        Ok(provider_address.path)
    }
}
