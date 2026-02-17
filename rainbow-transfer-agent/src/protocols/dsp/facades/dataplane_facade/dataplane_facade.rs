use std::sync::Arc;
use urn::Urn;
use rainbow_common::config::types::roles::RoleConfig;
use rainbow_connector::ConnectorInstanceDto;
use rainbow_dataplane::{DataplaneCommand, DataplaneManager, DataplaneManagerInput};
use crate::protocols::dsp::facades::dataplane_facade::DataPlaneFacadeTrait;
use crate::protocols::dsp::protocol_types::DataAddressDto;

pub struct DataPlaneFacade {
    dataplane_manager: Arc<DataplaneManager>,
}

impl DataPlaneFacade {
    pub fn new(dataplane_manager: Arc<DataplaneManager>) -> DataPlaneFacade {
        DataPlaneFacade { dataplane_manager }
    }

    /// Helper: fire a simple command (no payload) on the DataplaneManager.
    async fn fire_command(
        &self,
        transfer_id: &Urn,
        command: DataplaneCommand,
    ) -> anyhow::Result<()> {
        self.dataplane_manager
            .execute_command(&DataplaneManagerInput {
                transfer_process_id: transfer_id.clone(),
                command,
            })
            .await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl DataPlaneFacadeTrait for DataPlaneFacade {
    // ─── TransferRequest ───

    async fn on_transfer_request_pre(
        &self,
        transfer_id: &Urn,
        _data_address: &Option<DataAddressDto>,
    ) -> anyhow::Result<Option<DataAddressDto>> {
        // Consumer side: register dataplane process
        self.fire_command(
            transfer_id,
            DataplaneCommand::SetInit {
                role: RoleConfig::Consumer,
                connector_instance: None,
                data_address: None, // TODO: convert DataAddressDto → DataplaneAddress for PUSH
            },
        )
        .await?;
        // TODO: for PUSH, return the consumer ingest DataAddress
        Ok(None)
    }

    async fn on_transfer_request_post(
        &self,
        transfer_id: &Urn,
        connector_instance: &ConnectorInstanceDto,
        _data_address: &Option<DataAddressDto>,
    ) -> anyhow::Result<()> {
        // Provider side: register dataplane process with connector
        self.fire_command(
            transfer_id,
            DataplaneCommand::SetInit {
                role: RoleConfig::Provider,
                connector_instance: Some(connector_instance.id.clone()),
                data_address: None, // TODO: convert DataAddressDto → DataplaneAddress
            },
        )
        .await
    }

    // ─── TransferStart ───

    async fn on_transfer_start_pre(
        &self,
        transfer_id: &Urn,
    ) -> anyhow::Result<Option<DataAddressDto>> {
        self.fire_command(transfer_id, DataplaneCommand::SetStarted).await?;
        // TODO: return DataAddress (proxy URL for PULL provider)
        Ok(None)
    }

    async fn on_transfer_start_post(
        &self,
        transfer_id: &Urn,
    ) -> anyhow::Result<Option<DataAddressDto>> {
        self.fire_command(transfer_id, DataplaneCommand::SetStarted).await?;
        Ok(None)
    }

    // ─── TransferSuspension → SetStopped ───

    async fn on_transfer_suspension_pre(&self, transfer_id: &Urn) -> anyhow::Result<()> {
        self.fire_command(transfer_id, DataplaneCommand::SetStopped).await
    }

    async fn on_transfer_suspension_post(&self, transfer_id: &Urn) -> anyhow::Result<()> {
        self.fire_command(transfer_id, DataplaneCommand::SetStopped).await
    }

    // ─── TransferCompletion → SetTerminated ───

    async fn on_transfer_completion_pre(&self, transfer_id: &Urn) -> anyhow::Result<()> {
        self.fire_command(transfer_id, DataplaneCommand::SetTerminated).await
    }

    async fn on_transfer_completion_post(&self, transfer_id: &Urn) -> anyhow::Result<()> {
        self.fire_command(transfer_id, DataplaneCommand::SetTerminated).await
    }

    // ─── TransferTermination → SetTerminated ───

    async fn on_transfer_termination_pre(&self, transfer_id: &Urn) -> anyhow::Result<()> {
        self.fire_command(transfer_id, DataplaneCommand::SetTerminated).await
    }

    async fn on_transfer_termination_post(&self, transfer_id: &Urn) -> anyhow::Result<()> {
        self.fire_command(transfer_id, DataplaneCommand::SetTerminated).await
    }
}