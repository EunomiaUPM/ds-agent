use crate::entities::dataplane_manager_ref::dataplane_commands::{
    DataplaneCommand, DataplaneCommandResponse,
};
use crate::entities::dataplane_manager_ref::dataplane_context::DataplaneContext;
use crate::entities::dataplane_manager_ref::dataplane_handlers_strategy::DataplaneStrategyFactory;
use crate::DataplaneTransfersEntitiesTrait;
use common::config::services::TransferConfig;
use connector::ConnectorInstanceTrait;
use std::sync::Arc;
use ymir::errors::{Errors, Outcome};

pub struct DataplaneManagerRef {
    pub(super) dataplane_entity: Arc<dyn DataplaneTransfersEntitiesTrait>,
    pub(super) connector_entity: Arc<dyn ConnectorInstanceTrait>,
    config: Arc<TransferConfig>,
}

impl DataplaneManagerRef {
    pub fn new(
        dataplane_entity: Arc<dyn DataplaneTransfersEntitiesTrait>,
        connector_entity: Arc<dyn ConnectorInstanceTrait>,
        config: Arc<TransferConfig>,
    ) -> Self {
        Self {
            dataplane_entity,
            connector_entity,
            config,
        }
    }

    pub async fn execute_command(
        &self,
        command: DataplaneCommand,
    ) -> Outcome<DataplaneCommandResponse> {
        // SetInit creates a brand-new context — handle it before the continuation path
        if let DataplaneCommand::SetInit(init) = command {
            DataplaneContext::from_init(
                self.dataplane_entity.clone(),
                self.connector_entity.clone(),
                self.config.clone(),
                init,
            )
            .await?;
            return Ok(DataplaneCommandResponse::Ok);
        }

        // All remaining commands carry a continuation that identifies the existing process
        let context = match command.clone() {
            DataplaneCommand::SetStarted(continuation)
            | DataplaneCommand::SetSubscribing(continuation)
            | DataplaneCommand::SetUnsubscribing(continuation)
            | DataplaneCommand::SetStopped(continuation)
            | DataplaneCommand::SetTerminating(continuation) => {
                DataplaneContext::from_continuation(
                    self.dataplane_entity.clone(),
                    self.connector_entity.clone(),
                    self.config.clone(),
                    continuation,
                )
            }
            cmd => {
                return Ok(DataplaneCommandResponse::Err(
                    Errors::crazy(
                        format!("Dataplane command {} not expected here", cmd),
                        None,
                    )
                    .into(),
                ))
            }
        }
        .await?;

        // Select strategy based on (role, interaction_mode) from the loaded context
        let handler_strategy = DataplaneStrategyFactory::new(
            self.dataplane_entity.clone(),
            self.connector_entity.clone(),
            self.config.clone(),
        )
        .get_strategy(&context);

        // Dispatch to the appropriate handler method
        match command {
            DataplaneCommand::SetStarted(_) => { handler_strategy.set_started(context).await?; }
            DataplaneCommand::SetSubscribing(_) => { handler_strategy.set_subscribing(context).await?; }
            DataplaneCommand::SetUnsubscribing(_) => { handler_strategy.set_unsubscribing(context).await?; }
            DataplaneCommand::SetStopped(_) => { handler_strategy.set_stopped(context).await?; }
            DataplaneCommand::SetTerminating(_) => { handler_strategy.set_terminating(context).await?; }
            _ => unreachable!(),
        };

        Ok(DataplaneCommandResponse::Ok)
    }
}
