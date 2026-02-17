use crate::entities::transfer_process::TransferAgentProcessesTrait;
use crate::protocols::dsp::facades::data_plane_facade::dataplane_strategy_factory::DataPlaneStrategyFactory;
use crate::protocols::dsp::facades::data_plane_facade::DataPlaneFacadeTrait;
use crate::protocols::dsp::protocol_types::DataAddressDto;
use rainbow_catalog_agent::DataServiceDto;
use rainbow_common::config::types::roles::RoleConfig;
use rainbow_common::dcat_formats::DctFormats;
use std::ops::Deref;
use std::sync::Arc;
use urn::Urn;

pub struct DataPlaneProviderFacadeForDSProtocol {
    dataplane_strategy_factory: Arc<DataPlaneStrategyFactory>,
    transfer_process_entities: Arc<dyn TransferAgentProcessesTrait>,
}
impl DataPlaneProviderFacadeForDSProtocol {
    pub fn new(
        dataplane_strategy_factory: Arc<DataPlaneStrategyFactory>,
        transfer_process_entities: Arc<dyn TransferAgentProcessesTrait>,
    ) -> Self {
        Self { dataplane_strategy_factory, transfer_process_entities }
    }
}

#[async_trait::async_trait]
impl DataPlaneFacadeTrait for DataPlaneProviderFacadeForDSProtocol {
    async fn get_dataplane_address(&self, session_id: &Urn) -> anyhow::Result<DataAddressDto> {
        todo!()
    }

    async fn on_transfer_request_pre(
        &self,
        session_id: &Urn,
        format: &DctFormats,
        data_service: &Option<DataServiceDto>,
        data_address: &Option<DataAddressDto>,
    ) -> anyhow::Result<()> {
        let strategy = self.dataplane_strategy_factory.get_strategy(&RoleConfig::Provider, &format);
        strategy.on_transfer_request_pre(session_id, format, data_service, data_address).await?;
        Ok(())
    }

    async fn on_transfer_request_post(
        &self,
        session_id: &Urn,
        format: &DctFormats,
        data_service: &Option<DataServiceDto>,
        data_address: &Option<DataAddressDto>,
    ) -> anyhow::Result<()> {
        let process = self
            .transfer_process_entities
            .get_transfer_process_by_id(session_id)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        let format = process.inner.transfer_direction.parse::<DctFormats>()?;
        let role = process.inner.role.parse::<RoleConfig>()?;
        let strategy = self.dataplane_strategy_factory.get_strategy(&role, &format);
        strategy.on_transfer_request_post(session_id, &format, data_service, data_address).await?;
        Ok(())
    }

    async fn on_transfer_start_pre(&self, session_id: &Urn) -> anyhow::Result<()> {
        let process = self
            .transfer_process_entities
            .get_transfer_process_by_id(session_id)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        let format = process.inner.transfer_direction.parse::<DctFormats>()?;
        let role = process.inner.role.parse::<RoleConfig>()?;
        let strategy = self.dataplane_strategy_factory.get_strategy(&role, &format);
        strategy.on_transfer_start_pre(session_id).await?;
        Ok(())
    }

    async fn on_transfer_start_post(&self, session_id: &Urn) -> anyhow::Result<()> {
        let process = self
            .transfer_process_entities
            .get_transfer_process_by_id(session_id)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        let format = process.inner.transfer_direction.parse::<DctFormats>()?;
        let role = process.inner.role.parse::<RoleConfig>()?;
        let strategy = self.dataplane_strategy_factory.get_strategy(&role, &format);
        strategy.on_transfer_start_post(session_id).await?;
        Ok(())
    }

    async fn on_transfer_suspension_pre(&self, session_id: &Urn) -> anyhow::Result<()> {
        let process = self
            .transfer_process_entities
            .get_transfer_process_by_id(session_id)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        let format = process.inner.transfer_direction.parse::<DctFormats>()?;
        let role = process.inner.role.parse::<RoleConfig>()?;
        let strategy = self.dataplane_strategy_factory.get_strategy(&role, &format);
        strategy.on_transfer_suspension_pre(session_id).await?;
        Ok(())
    }

    async fn on_transfer_suspension_post(&self, session_id: &Urn) -> anyhow::Result<()> {
        let process = self
            .transfer_process_entities
            .get_transfer_process_by_id(session_id)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        let format = process.inner.transfer_direction.parse::<DctFormats>()?;
        let role = process.inner.role.parse::<RoleConfig>()?;
        let strategy = self.dataplane_strategy_factory.get_strategy(&role, &format);
        strategy.on_transfer_suspension_post(session_id).await?;
        Ok(())
    }

    async fn on_transfer_completion_pre(&self, session_id: &Urn) -> anyhow::Result<()> {
        let process = self
            .transfer_process_entities
            .get_transfer_process_by_id(session_id)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        let format = process.inner.transfer_direction.parse::<DctFormats>()?;
        let role = process.inner.role.parse::<RoleConfig>()?;
        let strategy = self.dataplane_strategy_factory.get_strategy(&role, &format);
        strategy.on_transfer_completion_pre(session_id).await?;
        Ok(())
    }

    async fn on_transfer_completion_post(&self, session_id: &Urn) -> anyhow::Result<()> {
        let process = self
            .transfer_process_entities
            .get_transfer_process_by_id(session_id)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        let format = process.inner.transfer_direction.parse::<DctFormats>()?;
        let role = process.inner.role.parse::<RoleConfig>()?;
        let strategy = self.dataplane_strategy_factory.get_strategy(&role, &format);
        strategy.on_transfer_completion_post(session_id).await?;
        Ok(())
    }

    async fn on_transfer_termination_pre(&self, session_id: &Urn) -> anyhow::Result<()> {
        let process = self
            .transfer_process_entities
            .get_transfer_process_by_id(session_id)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        let format = process.inner.transfer_direction.parse::<DctFormats>()?;
        let role = process.inner.role.parse::<RoleConfig>()?;
        let strategy = self.dataplane_strategy_factory.get_strategy(&role, &format);
        strategy.on_transfer_termination_pre(session_id).await?;
        Ok(())
    }

    async fn on_transfer_termination_post(&self, session_id: &Urn) -> anyhow::Result<()> {
        let process = self
            .transfer_process_entities
            .get_transfer_process_by_key_value(session_id)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        let format = process.inner.transfer_direction.parse::<DctFormats>()?;
        let role = process.inner.role.parse::<RoleConfig>()?;
        let strategy = self.dataplane_strategy_factory.get_strategy(&role, &format);
        strategy.on_transfer_termination_post(session_id).await?;
        Ok(())
    }
}
