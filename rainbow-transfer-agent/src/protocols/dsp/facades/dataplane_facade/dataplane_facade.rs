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
}

#[async_trait::async_trait]
impl DataPlaneFacadeTrait for DataPlaneFacade {
    async fn on_transfer_request_pre(
        &self,
        transfer_id: &Urn,
        _data_address: &Option<DataAddressDto>,
    ) -> anyhow::Result<()> {
        // Consumer side: no connector, no distribution
        self.dataplane_manager
            .execute_command(&DataplaneManagerInput {
                transfer_process_id: transfer_id.clone(),
                command: DataplaneCommand::SetInit {
                    role: RoleConfig::Consumer,
                    connector_instance: None,
                    data_address: None,
                },
            })
            .await?;
        Ok(())
    }

    async fn on_transfer_request_post(
        &self,
        transfer_id: &Urn,
        connector_instance: &ConnectorInstanceDto,
        _data_address: &Option<DataAddressDto>,
    ) -> anyhow::Result<()> {
        // Provider side: pass the connector instance id
        self.dataplane_manager
            .execute_command(&DataplaneManagerInput {
                transfer_process_id: transfer_id.clone(),
                command: DataplaneCommand::SetInit {
                    role: RoleConfig::Provider,
                    connector_instance: Some(connector_instance.id.clone()),
                    data_address: None, // TODO: convert DataAddressDto to DataplaneAddress
                },
            })
            .await?;
        Ok(())
    }

    async fn on_transfer_start_pre(
        &self,
        _transfer_id: &Urn,
    ) -> anyhow::Result<Option<DataAddressDto>> {
        Ok(None)
    }

    async fn on_transfer_start_post(
        &self,
        _transfer_id: &Urn,
    ) -> anyhow::Result<Option<DataAddressDto>> {
        Ok(None)
    }

    async fn on_transfer_suspension_pre(&self, _transfer_id: &Urn) -> anyhow::Result<()> {
        Ok(())
    }

    async fn on_transfer_suspension_post(&self, _transfer_id: &Urn) -> anyhow::Result<()> {
        Ok(())
    }

    async fn on_transfer_completion_pre(&self, _transfer_id: &Urn) -> anyhow::Result<()> {
        Ok(())
    }

    async fn on_transfer_completion_post(&self, _transfer_id: &Urn) -> anyhow::Result<()> {
        Ok(())
    }

    async fn on_transfer_termination_pre(&self, _transfer_id: &Urn) -> anyhow::Result<()> {
        Ok(())
    }

    async fn on_transfer_termination_post(&self, _transfer_id: &Urn) -> anyhow::Result<()> {
        Ok(())
    }
}