use crate::entities::dataplane_manager::driver_factory::{DataplaneDriver, DataplaneDriverFactory};
use crate::entities::dataplane_manager::{
    DataplaneCommand, DataplaneManagerInput, DataplaneResponse,
};
use crate::entities::dataplane_transfers::{
    DataplaneTransferDto, DataplaneTransfersEntitiesTrait, InteractionMode,
    NewDataplaneTransferDto, TransferRole, TransferState,
};
use anyhow::anyhow;
use rainbow_connector::{ConnectorInstanceTrait, InteractionConfig};
use std::sync::Arc;
use urn::Urn;
use crate::entities::dataplane_manager::config_builder::DataplaneConfigBuilder;

pub struct DataplaneManager {
    dataplane_entity: Arc<dyn DataplaneTransfersEntitiesTrait>,
    connector_entity: Arc<dyn ConnectorInstanceTrait>,
    driver_factory: Arc<DataplaneDriverFactory>,
}

impl DataplaneManager {
    pub fn new(
        dataplane_entity: Arc<dyn DataplaneTransfersEntitiesTrait>,
        connector_entity: Arc<dyn ConnectorInstanceTrait>,
        driver_factory: Arc<DataplaneDriverFactory>,
    ) -> Self {
        Self {
            dataplane_entity,
            connector_entity,
            driver_factory,
        }
    }

    pub async fn execute_command(
        &self,
        input: &DataplaneManagerInput,
    ) -> anyhow::Result<DataplaneResponse> {
        // 1. load dataplane process
        let dataplane_process_opt = self
            .dataplane_entity
            .get_dataplane_transfer_by_process_id(&input.transfer_process_id)
            .await
            .map_err(anyhow::Error::from)?;

        // 2. resolve connector URN (Fixes borrowed temporary value error)
        let connector_urn = match &dataplane_process_opt {
            None => match &input.command {
                DataplaneCommand::SetInit { connector_instance, .. } => connector_instance.clone(),
                _ => {
                    return Err(anyhow!(
                        "DataplaneManager does not support this command without an existing process"
                    ))
                }
            },
            Some(dataplane_process) => match &dataplane_process.inner.connector_instance_id {
                Some(id_str) => id_str.parse::<Urn>()?,
                None => return Err(anyhow!("Dataplane process has no connector instance ID")),
            },
        };

        // 3. fetch connector instance
        let connector_instance = self
            .connector_entity
            .get_instance_by_id(&connector_urn)
            .await?
            .ok_or_else(|| anyhow!("Connector instance not found"))?;

        // 4. guard for creation
        if dataplane_process_opt.is_none() {
            if let DataplaneCommand::SetInit {
                role,
                connector_instance: connector_urn,
                data_address: _,
            } = &input.command
            {
                let interaction = connector_instance.interaction.clone();
                let interaction_mode = match interaction {
                    InteractionConfig::Pull(_) => InteractionMode::Pull,
                    InteractionConfig::Push(_) => InteractionMode::Push,
                };

                let _new_dataplane_process = self
                    .dataplane_entity
                    .create_dataplane_transfer(&NewDataplaneTransferDto {
                        id: None,
                        transfer_process_id: input.transfer_process_id.to_string(),
                        role: TransferRole::try_from(role.clone())?,
                        interaction_mode,
                        state: TransferState::Init,
                        connector_instance_id: Some(connector_urn.clone()),
                        ingress_config: serde_json::Value::Null,
                        egress_config: serde_json::Value::Null,
                    })
                    .await?;

                return Ok(DataplaneResponse::Ok);
            }
        }

        // 5. dataplane process is guaranteed to be some here
        let dataplane_process = dataplane_process_opt
            .ok_or_else(|| anyhow!("Dataplane process not found after creation check"))?;

        // 6. create driver
        let driver = self
            .driver_factory
            .create_driver(&dataplane_process, &connector_instance)?;

        // 7. handle command
        let cmd = self
            .handle_command(&input.command, &dataplane_process, &driver)
            .await?;

        // 8. trigger autonomous transitions
        let _ = self
            .trigger_autonomous_transition(&input.command, &dataplane_process, &driver)
            .await?;

        Ok(cmd)
    }

    async fn handle_command(
        &self,
        cmd: &DataplaneCommand,
        process: &DataplaneTransferDto,
        _driver: &DataplaneDriver,
    ) -> anyhow::Result<DataplaneResponse> {
        let mut builder = DataplaneConfigBuilder::from_process(process);
        match cmd {
            DataplaneCommand::SetInit { .. } => {}
            DataplaneCommand::SetConfiguring => {}
            DataplaneCommand::SetAuth => {}
            DataplaneCommand::SetReady => {}
            DataplaneCommand::SetSubscribing => {}
            DataplaneCommand::SetStarted => {}
            DataplaneCommand::SetUnsubscribing => {}
            DataplaneCommand::SetStopped => {}
            DataplaneCommand::SetTerminated => {}
        };
        Ok(DataplaneResponse::Ok)
    }

    async fn trigger_autonomous_transition(
        &self,
        _cmd: &DataplaneCommand,
        process: &DataplaneTransferDto,
        _driver: &DataplaneDriver,
    ) -> anyhow::Result<DataplaneResponse> {
        match process.inner.state {
            TransferState::Init => {}
            TransferState::Configuring => {}
            TransferState::Auth => {}
            TransferState::Ready => {}
            TransferState::Subscribing => {}
            TransferState::Started => {}
            TransferState::Unsubscribing => {}
            TransferState::Stopped => {}
            TransferState::Terminated => {}
            TransferState::Error => {}
        };
        Ok(DataplaneResponse::Ok)
    }
}
