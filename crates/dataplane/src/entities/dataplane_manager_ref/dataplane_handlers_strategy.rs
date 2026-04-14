use crate::entities::dataplane_manager_ref::dataplane_commands::{
    DataplaneCommandStateMachine, DataplaneInitCommandTypes,
};
use crate::entities::dataplane_manager_ref::dataplane_context::DataplaneContext;
use crate::entities::dataplane_manager_ref::dataplane_handlers_consumer_pull::DataplaneHandlerConsumerPull;
use crate::entities::dataplane_manager_ref::dataplane_handlers_consumer_push::DataplaneHandlerConsumerPush;
use crate::entities::dataplane_manager_ref::dataplane_handlers_provider_pull::DataplaneHandlerProviderPull;
use crate::entities::dataplane_manager_ref::dataplane_handlers_provider_push::DataplaneHandlerProviderPush;
use crate::entities::dataplane_transfers::{InteractionMode, TransferRole};
use crate::DataplaneTransfersEntitiesTrait;
use common::config::services::TransferConfig;
use connector::ConnectorInstanceTrait;
use std::sync::Arc;
use ymir::errors::Outcome;

pub struct DataplaneStrategyFactory {
    pub(super) dataplane_entity: Arc<dyn DataplaneTransfersEntitiesTrait>,
    pub(super) connector_entity: Arc<dyn ConnectorInstanceTrait>,
    config: Arc<TransferConfig>,
}
impl DataplaneStrategyFactory {
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
    pub fn get_strategy(
        &self,
        context: &DataplaneContext,
    ) -> Box<dyn DataplaneCommandStateMachine> {
        let role = context.dataplane_process_role();
        let interaction_mode = context.dataplane_process_interaction_mode();
        match (role, interaction_mode) {
            (TransferRole::Provider, InteractionMode::Pull) => Box::new(DataplaneHandlerProviderPull::new(
                self.dataplane_entity.clone(),
                self.connector_entity.clone(),
                self.config.clone(),
            )),
            (TransferRole::Provider, InteractionMode::Push) => Box::new(DataplaneHandlerProviderPush::new(
                self.dataplane_entity.clone(),
                self.connector_entity.clone(),
                self.config.clone(),
            )),
            (TransferRole::Consumer, InteractionMode::Pull) => Box::new(DataplaneHandlerConsumerPull::new(
                self.dataplane_entity.clone(),
                self.connector_entity.clone(),
                self.config.clone(),
            )),
            (TransferRole::Consumer, InteractionMode::Push) => Box::new(DataplaneHandlerConsumerPush::new(
                self.dataplane_entity.clone(),
                self.connector_entity.clone(),
                self.config.clone(),
            )),
        }
    }
}
