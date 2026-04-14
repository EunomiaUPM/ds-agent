use crate::data::entities::dataplane_transfers::Model;
use crate::entities::dataplane_manager_ref::dataplane_commands::{
    DataplaneContinuation, DataplaneInitCommandDirection, DataplaneInitCommandTypes,
};
use crate::entities::dataplane_manager_ref::dataplane_driver_factory::DataplaneDriverFactoryTrait;
use crate::entities::dataplane_manager_ref::dataplane_proxy::{
    DataplaneProxy, DataplaneProxyEgress, DataplaneProxyIngress,
};
use crate::entities::dataplane_transfers::{
    DataplaneTransferDto, InteractionMode, NewDataplaneTransferDto, TransferRole, TransferState,
};
use crate::{
    DataplaneAddress, DataplaneInitCommandType, DataplaneTransfersEntitiesTrait, EgressConfig,
    IngressConfig,
};
use common::config::services::TransferConfig;
use connector::{ConnectorInstanceDto, ConnectorInstanceTrait};
use std::str::FromStr;
use std::sync::Arc;
use urn::{Urn, UrnBuilder};
use ymir::errors::{Errors, Outcome};
use crate::entities::dataplane_drivers::DataplaneDriver;

#[derive(Clone, Debug)]
pub struct DataplaneContext {
    config: Arc<TransferConfig>,
    dataplane_process: DataplaneTransferDto,
    connector_instance: Option<ConnectorInstanceDto>,
    driver: Option<DataplaneDriver>,
    proxy: Option<DataplaneProxy>,
    forward_dataplane_address: Option<DataplaneAddress>,
}

impl DataplaneContext {
    pub async fn from_init(
        dataplane_entity: Arc<dyn DataplaneTransfersEntitiesTrait>,
        _connector_entity: Arc<dyn ConnectorInstanceTrait>,
        config: Arc<TransferConfig>,
        init: DataplaneInitCommandTypes,
    ) -> Outcome<Self> {
        let id = Urn::from_str(&*format!("urn:dataplane-transfer:{}", uuid::Uuid::new_v4()))?;
        let transfer_id = match &init {
            DataplaneInitCommandTypes::AsProvider {
                transfer_process_id,
                ..
            } => transfer_process_id,
            DataplaneInitCommandTypes::AsConsumer {
                transfer_process_id,
                ..
            } => transfer_process_id,
        };
        let transfer_role = match &init {
            DataplaneInitCommandTypes::AsProvider { .. } => TransferRole::Provider,
            DataplaneInitCommandTypes::AsConsumer { .. } => TransferRole::Consumer,
        };
        let interaction_mode = match &init {
            DataplaneInitCommandTypes::AsProvider { direction, .. } => match direction {
                DataplaneInitCommandDirection::Pull { .. } => InteractionMode::Pull,
                DataplaneInitCommandDirection::Push { .. } => InteractionMode::Push,
            },
            DataplaneInitCommandTypes::AsConsumer { direction, .. } => match direction {
                DataplaneInitCommandDirection::Pull { .. } => InteractionMode::Pull,
                DataplaneInitCommandDirection::Push { .. } => InteractionMode::Push,
            },
        };
        let connector_instance = match &init {
            DataplaneInitCommandTypes::AsProvider {
                connector_instance, ..
            } => Some(connector_instance.clone()),
            DataplaneInitCommandTypes::AsConsumer { .. } => None,
        };
        let data_plane_address = match &init {
            DataplaneInitCommandTypes::AsProvider { direction, .. } => match direction {
                DataplaneInitCommandDirection::Pull { data_address } => Some(data_address),
                DataplaneInitCommandDirection::Push { data_address } => Some(data_address),
            },
            DataplaneInitCommandTypes::AsConsumer { direction, .. } => match direction {
                DataplaneInitCommandDirection::Pull { data_address } => Some(data_address),
                DataplaneInitCommandDirection::Push { data_address } => Some(data_address),
            },
        };

        // db access
        let dataplane_process = dataplane_entity
            .create_dataplane_transfer(&NewDataplaneTransferDto {
                id: Some(id),
                transfer_process_id: transfer_id.to_string(),
                role: transfer_role,
                interaction_mode,
                state: TransferState::Init,
                connector_instance_id: connector_instance.as_ref().map(|c| c.id.clone()),
                ingress_config: Default::default(),
                egress_config: Default::default(),
            })
            .await?;

        // context
        Ok(Self {
            config,
            dataplane_process,
            connector_instance,
            driver: None,
            proxy: None,
            forward_dataplane_address: data_plane_address.map(|addr| addr.clone()),
        })
    }

    pub async fn from_continuation(
        dataplane_entity: Arc<dyn DataplaneTransfersEntitiesTrait>,
        connector_entity: Arc<dyn ConnectorInstanceTrait>,
        driver_factory: Arc<dyn DataplaneDriverFactoryTrait>,
        config: Arc<TransferConfig>,
        continuation: DataplaneContinuation,
    ) -> Outcome<Self> {
        // db access
        let dataplane_process = dataplane_entity
            .get_dataplane_transfer_by_process_id(&continuation.transfer_dto_urn)
            .await?;
        if let None = dataplane_process {
            return Err(Errors::crazy("Dataplane Process not found", None));
        }
        let dataplane_process = dataplane_process.unwrap();

        // connector
        let connector_id = &dataplane_process.inner.connector_instance_id;
        let connector = match connector_id {
            Some(connector_id) => {
                let connector_urn = Urn::from_str(&connector_id)?;
                connector_entity.get_instance_by_id(&connector_urn).await?
            }
            None => None,
        };

        // context
        let mut context = Self {
            config,
            dataplane_process: dataplane_process.clone(),
            connector_instance: connector.clone(),
            driver: None,
            proxy: None,
            forward_dataplane_address: None,
        };

        // driver to context
        context.driver = match &connector {
            Some(conn) => Some(driver_factory.get_or_create_driver(&context)?),
            None => None, // Consumer no tiene driver
        };

        // proxy to context
        let ingress = serde_json::from_value::<DataplaneProxyIngress>(
            dataplane_process.clone().inner.ingress_config,
        )?;
        let egress = serde_json::from_value::<DataplaneProxyEgress>(
            dataplane_process.clone().inner.egress_config,
        )?;
        context.proxy = Some(DataplaneProxy { ingress, egress });

        Ok(context)
    }

    pub fn set_dataplane_process(&mut self, dataplane_process: DataplaneTransferDto) -> &mut Self {
        self.dataplane_process = dataplane_process;
        self
    }

    pub fn set_connector_instance(
        &mut self,
        connector_instance: ConnectorInstanceDto,
    ) -> &mut Self {
        self.connector_instance = Some(connector_instance);
        self
    }

    pub fn set_driver(&mut self, driver: DataplaneDriver) -> &mut Self {
        self.driver = Some(driver);
        self
    }

    pub fn set_proxy(&mut self, proxy: DataplaneProxy) -> &mut Self {
        self.proxy = Some(proxy);
        self
    }

    pub fn connector_instance(&self) -> Option<&ConnectorInstanceDto> {
        self.connector_instance.as_ref()
    }

    pub fn dataplane_process(&self) -> &DataplaneTransferDto {
        &self.dataplane_process
    }

    pub fn dataplane_process_state(&self) -> TransferState {
        self.dataplane_process.inner.state.clone()
    }

    pub fn dataplane_process_interaction_mode(&self) -> InteractionMode {
        self.dataplane_process.inner.interaction_mode.clone()
    }

    pub fn dataplane_process_role(&self) -> TransferRole {
        self.dataplane_process.inner.role.clone()
    }

    pub fn forward_dataplane_address(&self) -> Option<&DataplaneAddress> {
        self.forward_dataplane_address.as_ref()
    }

    pub fn driver(&self) -> Option<&DataplaneDriver> {
        self.driver.as_ref()
    }

    pub fn proxy(&self) -> Option<&DataplaneProxy> {
        self.proxy.as_ref()
    }

    pub fn config(&self) -> Arc<TransferConfig> {
        self.config.clone()
    }
}
