use crate::entities::dataplane_manager_ref::dataplane_commands::{
    DataplaneCommandStateMachine, DataplaneInitCommandTypes,
};
use crate::entities::dataplane_manager_ref::dataplane_context::DataplaneContext;
use crate::DataplaneTransfersEntitiesTrait;
use common::config::services::TransferConfig;
use connector::ConnectorInstanceTrait;
use std::sync::Arc;
use ymir::errors::Outcome;

pub struct DataplaneHandlerProviderPush {
    pub(super) dataplane_entity: Arc<dyn DataplaneTransfersEntitiesTrait>,
    pub(super) connector_entity: Arc<dyn ConnectorInstanceTrait>,
    config: Arc<TransferConfig>,
}

impl DataplaneHandlerProviderPush {
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
impl DataplaneCommandStateMachine for DataplaneHandlerProviderPush {
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

    async fn set_auth(&self, context: DataplaneContext) -> Outcome<DataplaneContext> {
        todo!()
    }

    async fn set_subscribing(&self, context: DataplaneContext) -> Outcome<DataplaneContext> {
        todo!()
    }

    async fn set_unsubscribing(&self, context: DataplaneContext) -> Outcome<DataplaneContext> {
        todo!()
    }
}
