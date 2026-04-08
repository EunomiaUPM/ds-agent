use crate::entities::dataplane_manager::driver_factory::DataplaneDriverFactory;
use crate::entities::dataplane_manager_ref::dataplane_commands::{
    DataplaneCommandStateMachine, DataplaneInitCommandTypes,
};
use crate::entities::dataplane_manager_ref::dataplane_context::DataplaneContext;
use crate::entities::dataplane_manager_ref::dataplane_driver::{
    DataplaneDriver, DataplaneDriverAuth, DataplaneDriverInteraction,
};
use crate::entities::dataplane_manager_ref::dataplane_proxy::DataplaneProxy;
use crate::entities::dataplane_transfers::{EditDataplaneTransferDto, TransferState};
use crate::DataplaneTransfersEntitiesTrait;
use common::config::services::TransferConfig;
use connector::ConnectorInstanceTrait;
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::{Errors, Outcome};

pub struct DataplaneHandlerProviderPull {
    pub(super) dataplane_entity: Arc<dyn DataplaneTransfersEntitiesTrait>,
    pub(super) connector_entity: Arc<dyn ConnectorInstanceTrait>,
    config: Arc<TransferConfig>,
}

impl DataplaneHandlerProviderPull {
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
impl DataplaneCommandStateMachine for DataplaneHandlerProviderPull {
    fn dataplane_entity(&self) -> Arc<dyn DataplaneTransfersEntitiesTrait> {
        self.dataplane_entity.clone()
    }

    fn connector_entity(&self) -> Arc<dyn ConnectorInstanceTrait> {
        self.connector_entity.clone()
    }

    fn transfer_config(&self) -> Arc<TransferConfig> {
        self.config.clone()
    }

    async fn set_configuring(&self, mut context: DataplaneContext) -> Outcome<DataplaneContext> {
        let dataplane_urn = Urn::from_str(&*context.dataplane_process().inner.id)?;
        // state
        let new_state = TransferState::Configuring;
        // driver
        let driver = DataplaneDriver::new(
            DataplaneDriverAuth {
                r#type: "NONE".to_string(),
            },
            DataplaneDriverInteraction {
                r#type: "NONE".to_string(),
            },
        );
        // proxy
        let proxy = DataplaneProxy::new();
        // Serialize before moving into context
        let ingress_val = serde_json::to_value(proxy.ingress())?;
        let egress_val = serde_json::to_value(proxy.egress())?;
        let driver_val = serde_json::to_value(&driver)?;
        context.set_driver(driver);
        context.set_proxy(proxy);
        // update
        let dataplane_process = self
            .dataplane_entity
            .put_dataplane_transfer_by_id(
                &dataplane_urn,
                &EditDataplaneTransferDto {
                    state: Some(new_state),
                    ingress_config: Some(ingress_val),
                    egress_config: Some(egress_val),
                    flow_control: Some(driver_val),
                    ..EditDataplaneTransferDto::default()
                },
            )
            .await?;
        context.set_dataplane_process(dataplane_process);
        Ok(context)
    }

    async fn set_auth(&self, mut context: DataplaneContext) -> Outcome<DataplaneContext> {
        let dataplane_urn = Urn::from_str(&*context.dataplane_process().inner.id)?;
        // state
        let new_state = TransferState::Auth;
        // driver
        let driver = match context.driver() {
            Some(driver) => driver,
            None => return Err(Errors::crazy("Driver should exists by now", None))
        };
        // perform auth
        // driver.auth
        let dataplane_process = self
            .dataplane_entity
            .put_dataplane_transfer_by_id(
                &dataplane_urn,
                &EditDataplaneTransferDto {
                    state: Some(new_state),
                    ..EditDataplaneTransferDto::default()
                },
            )
            .await?;
        context.set_dataplane_process(dataplane_process);
        Ok(context)
    }
}
