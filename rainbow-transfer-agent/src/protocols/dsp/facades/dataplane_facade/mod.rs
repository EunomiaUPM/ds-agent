pub(crate) mod dataplane_facade;

use crate::protocols::dsp::protocol_types::DataAddressDto;
use rainbow_connector::ConnectorInstanceDto;
use urn::Urn;

#[mockall::automock]
#[async_trait::async_trait]
pub trait DataPlaneFacadeTrait: Send + Sync {
    /// Called before the consumer sends a TransferRequest (RPC/OUTBOUND side).
    /// Registers dataplane for consumer with no connector.
    async fn on_transfer_request_pre(
        &self,
        transfer_id: &Urn,
        data_address: &Option<DataAddressDto>,
    ) -> anyhow::Result<()>;

    /// Called after a TransferRequest is persisted (provider/INBOUND side).
    /// Registers the dataplane process for the provider with the resolved connector instance.
    async fn on_transfer_request_post(
        &self,
        transfer_id: &Urn,
        connector_instance: &ConnectorInstanceDto,
        data_address: &Option<DataAddressDto>,
    ) -> anyhow::Result<()>;

    async fn on_transfer_start_pre(&self, transfer_id: &Urn) -> anyhow::Result<Option<DataAddressDto>>;
    async fn on_transfer_start_post(&self, transfer_id: &Urn) -> anyhow::Result<Option<DataAddressDto>>;
    async fn on_transfer_suspension_pre(&self, transfer_id: &Urn) -> anyhow::Result<()>;
    async fn on_transfer_suspension_post(&self, transfer_id: &Urn) -> anyhow::Result<()>;
    async fn on_transfer_completion_pre(&self, transfer_id: &Urn) -> anyhow::Result<()>;
    async fn on_transfer_completion_post(&self, transfer_id: &Urn) -> anyhow::Result<()>;
    async fn on_transfer_termination_pre(&self, transfer_id: &Urn) -> anyhow::Result<()>;
    async fn on_transfer_termination_post(&self, transfer_id: &Urn) -> anyhow::Result<()>;
}