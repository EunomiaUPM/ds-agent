pub(crate) mod dataplane_facade;

use crate::protocols::dsp::protocol_types::DataAddressDto;
use rainbow_connector::ConnectorInstanceDto;
use urn::Urn;

#[mockall::automock]
#[async_trait::async_trait]
pub trait DataPlaneFacadeTrait: Send + Sync {
    // ─── TransferRequest ───

    /// Consumer OUTBOUND: init consumer DP (SetInit Consumer).
    /// Returns DataAddress for PUSH mode (ingest URL).
    async fn on_transfer_request_pre(
        &self,
        transfer_id: &Urn,
        data_address: &Option<DataAddressDto>,
    ) -> anyhow::Result<Option<DataAddressDto>>;

    /// Provider INBOUND: init provider DP with connector (SetInit Provider).
    async fn on_transfer_request_post(
        &self,
        transfer_id: &Urn,
        connector_instance: &ConnectorInstanceDto,
        data_address: &Option<DataAddressDto>,
    ) -> anyhow::Result<()>;

    // ─── TransferStart ───

    /// OUTBOUND: start local DP (SetStarted).
    /// Returns DataAddress (proxy URL for PULL).
    async fn on_transfer_start_pre(
        &self,
        transfer_id: &Urn,
    ) -> anyhow::Result<Option<DataAddressDto>>;

    /// INBOUND: start local DP (SetStarted).
    async fn on_transfer_start_post(
        &self,
        transfer_id: &Urn,
    ) -> anyhow::Result<Option<DataAddressDto>>;

    // ─── TransferSuspension ───

    async fn on_transfer_suspension_pre(&self, transfer_id: &Urn) -> anyhow::Result<()>;
    async fn on_transfer_suspension_post(&self, transfer_id: &Urn) -> anyhow::Result<()>;

    // ─── TransferCompletion ───

    async fn on_transfer_completion_pre(&self, transfer_id: &Urn) -> anyhow::Result<()>;
    async fn on_transfer_completion_post(&self, transfer_id: &Urn) -> anyhow::Result<()>;

    // ─── TransferTermination ───

    async fn on_transfer_termination_pre(&self, transfer_id: &Urn) -> anyhow::Result<()>;
    async fn on_transfer_termination_post(&self, transfer_id: &Urn) -> anyhow::Result<()>;
}