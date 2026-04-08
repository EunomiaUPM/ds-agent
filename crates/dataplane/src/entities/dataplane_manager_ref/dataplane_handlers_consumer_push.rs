use std::sync::Arc;
use crate::entities::dataplane_manager_ref::dataplane_commands::{
    DataplaneCommandStateMachine, DataplaneInitCommandTypes,
};
use crate::entities::dataplane_manager_ref::dataplane_context::DataplaneContext;
use ymir::errors::Outcome;
use common::config::services::TransferConfig;
use connector::ConnectorInstanceTrait;
use crate::DataplaneTransfersEntitiesTrait;

pub struct DataplaneHandlerConsumerPush {
    pub(super) dataplane_entity: Arc<dyn DataplaneTransfersEntitiesTrait>,
    pub(super) connector_entity: Arc<dyn ConnectorInstanceTrait>,
    config: Arc<TransferConfig>,
}

impl DataplaneHandlerConsumerPush {
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
}

#[async_trait::async_trait]
impl DataplaneCommandStateMachine for DataplaneHandlerConsumerPush {
    fn dataplane_entity(&self) -> Arc<dyn DataplaneTransfersEntitiesTrait> {
        self.dataplane_entity.clone()
    }

    fn connector_entity(&self) -> Arc<dyn ConnectorInstanceTrait> {
        self.connector_entity.clone()
    }

    fn transfer_config(&self) -> Arc<TransferConfig> {
        self.config.clone()
    }

    async fn set_configuring(&self, context: DataplaneContext) -> Outcome<DataplaneContext> {
        todo!()
    }
}
