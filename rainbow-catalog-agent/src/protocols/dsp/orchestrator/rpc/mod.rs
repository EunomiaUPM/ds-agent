use ymir::errors::Outcome;
use crate::protocols::dsp::orchestrator::rpc::types::{
    RpcCatalogRequestMessageDto, RpcCatalogResponseMessageDto, RpcDatasetRequestMessageDto,
};
use crate::protocols::dsp::protocol_types::{CatalogMessageWrapper, CatalogRequestMessageDto};
use crate::protocols::dsp::types::catalog_definition::Catalog;
use crate::protocols::dsp::types::dataset_definition::Dataset;

pub(crate) mod persistence;
pub(crate) mod rpc;
pub(crate) mod types;

#[async_trait::async_trait]
pub trait RPCOrchestratorTrait: Send + Sync + 'static {
    async fn setup_catalog_request_rpc(
        &self,
        input: &RpcCatalogRequestMessageDto,
    ) -> Outcome<RpcCatalogResponseMessageDto<RpcCatalogRequestMessageDto, Catalog>>;

    async fn setup_dataset_request_rpc(
        &self,
        input: &RpcDatasetRequestMessageDto,
    ) -> Outcome<RpcCatalogResponseMessageDto<RpcDatasetRequestMessageDto, Dataset>>;
}
