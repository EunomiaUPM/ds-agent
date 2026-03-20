use crate::DataServiceDto;
use rainbow_common::dcat_formats::DctFormats;
use rainbow_common::well_known::rpc::WellKnownRPCRequest;
use urn::Urn;
use ymir::errors::Outcome;

pub(crate) mod well_known_rpc_facade;

#[async_trait::async_trait]
#[allow(unused)]
pub trait WellKnownRPCFacadeTrait: Send + Sync {
    async fn resolve_dataspace_current_path(
        &self,
        input: &WellKnownRPCRequest,
    ) -> Outcome<String>;
}
