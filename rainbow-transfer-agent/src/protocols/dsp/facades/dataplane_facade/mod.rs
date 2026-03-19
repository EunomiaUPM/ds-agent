pub(crate) mod dataplane_facade;

use crate::protocols::dsp::protocol_types::DataAddressDto;
use rainbow_connector::ConnectorInstanceDto;
use rainbow_dataplane::DataplaneAddress;
use urn::Urn;
use ymir::errors::Outcome;
use crate::entities::transfer_process::TransferProcessDto;

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
    ) -> Outcome<Option<DataAddressDto>>;

    /// Provider INBOUND: init provider DP with connector (SetInit Provider).
    async fn on_transfer_request_post(
        &self,
        transfer_process: &TransferProcessDto,
        connector_instance: &ConnectorInstanceDto,
        data_address: &Option<DataAddressDto>,
    ) -> Outcome<()>;

    // ─── TransferStart ───

    /// OUTBOUND: start local DP (SetStarted).
    /// Returns DataAddress (proxy URL for PULL).
    async fn on_transfer_start_pre(
        &self,
        transfer_process: &TransferProcessDto,
    ) -> Outcome<Option<DataAddressDto>>;

    /// INBOUND: start local DP (SetStarted).
    /// Accepts the peer's DataAddress (PULL consumer: provider proxy URL) to set as egress.
    async fn on_transfer_start_post(
        &self,
        transfer_process: &TransferProcessDto,
        data_address: Option<DataAddressDto>,
    ) -> Outcome<Option<DataAddressDto>>;

    // ─── TransferSuspension ───

    async fn on_transfer_suspension_pre(&self, transfer_process: &TransferProcessDto,) -> Outcome<()>;
    async fn on_transfer_suspension_post(&self, transfer_process: &TransferProcessDto,) -> Outcome<()>;

    // ─── TransferCompletion ───

    async fn on_transfer_completion_pre(&self, transfer_process: &TransferProcessDto,) -> Outcome<()>;
    async fn on_transfer_completion_post(&self, transfer_process: &TransferProcessDto,) -> Outcome<()>;

    // ─── TransferTermination ───

    async fn on_transfer_termination_pre(&self, transfer_process: &TransferProcessDto,) -> Outcome<()>;
    async fn on_transfer_termination_post(&self, transfer_process: &TransferProcessDto,) -> Outcome<()>;

    // ─── Config updates ───

    /// Update the egress config for a transfer (e.g. after receiving peer's DataAddress)
    async fn set_egress(
        &self,
        transfer_process: &TransferProcessDto,
        data_address: DataplaneAddress,
    ) -> Outcome<()>;
}
