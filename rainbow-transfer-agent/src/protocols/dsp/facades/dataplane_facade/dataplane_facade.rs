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
    pub fn new(
        dataplane_manager: Arc<DataplaneManager>,
    ) -> DataPlaneFacade {
        DataPlaneFacade {
            dataplane_manager,
        }
    }
}

#[async_trait::async_trait]
impl DataPlaneFacadeTrait for DataPlaneFacade {
    async fn on_transfer_request_pre(&self, transfer_id: &Urn, _data_service: &Option<ConnectorInstanceDto>, data_address: &Option<DataAddressDto>) -> anyhow::Result<()> {
        self.dataplane_manager
            .execute_command(&DataplaneManagerInput {
                transfer_process_id: transfer_id.clone(), 
                command: DataplaneCommand::SetInit {
                    role: RoleConfig::Consumer,
                    connector_instance: None,
                    data_address: None, // TODO: convert data_address
                } 
            })
            .await?;
        Ok(())
    }

    async fn on_transfer_request_post(&self, _transfer_id: &Urn, _data_service: &Option<ConnectorInstanceDto>, _data_address: &Option<DataAddressDto>) -> anyhow::Result<()> {
        todo!()
    }

    async fn on_transfer_start_pre(&self, _transfer_id: &Urn) -> anyhow::Result<Option<DataAddressDto>> {
        todo!()
    }

    async fn on_transfer_start_post(&self, _transfer_id: &Urn) -> anyhow::Result<Option<DataAddressDto>> {
        todo!()
    }

    async fn on_transfer_suspension_pre(&self, _transfer_id: &Urn) -> anyhow::Result<()> {
        todo!()
    }

    async fn on_transfer_suspension_post(&self, _transfer_id: &Urn) -> anyhow::Result<()> {
        todo!()
    }

    async fn on_transfer_completion_pre(&self, _transfer_id: &Urn) -> anyhow::Result<()> {
        todo!()
    }

    async fn on_transfer_completion_post(&self, _transfer_id: &Urn) -> anyhow::Result<()> {
        todo!()
    }

    async fn on_transfer_termination_pre(&self, _transfer_id: &Urn) -> anyhow::Result<()> {
        todo!()
    }

    async fn on_transfer_termination_post(&self, _transfer_id: &Urn) -> anyhow::Result<()> {
        todo!()
    }
}