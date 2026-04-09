use crate::entities::dataplane_manager_ref::dataplane_context::DataplaneContext;
use crate::entities::dataplane_transfers::{EditDataplaneTransferDto, TransferState};
use crate::{DataplaneAddress, DataplaneTransfersEntitiesTrait};
use connector::{ConnectorInstanceDto, ConnectorInstanceTrait};
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::Outcome;
use common::config::services::TransferConfig;

#[derive(Clone)]
pub enum DataplaneCommand {
    SetInit(DataplaneInitCommandTypes),
    SetConfiguring,
    SetAuth,
    SetReady,
    SetStarted(DataplaneContinuation),
    SetSubscribing(DataplaneContinuation),
    SetUnsubscribing(DataplaneContinuation),
    SetStopped(DataplaneContinuation),
    SetTerminating(DataplaneContinuation),
}

#[derive(Clone)]
pub enum DataplaneInitCommandTypes {
    AsProvider {
        transfer_process_id: Urn,
        connector_instance: ConnectorInstanceDto,
        direction: DataplaneInitCommandDirection,
    },
    AsConsumer {
        transfer_process_id: Urn,
        direction: DataplaneInitCommandDirection,
    },
}

#[derive(Clone)]
pub enum DataplaneInitCommandDirection {
    Pull,
    Push { data_address: DataplaneAddress },
}

#[derive(Clone)]
pub struct DataplaneContinuation {
    pub transfer_dto_urn: Urn,
}

pub enum DataplaneCommandResponse {
    Ok,
    OkWithAddress(DataplaneAddress),
    Err(Box<dyn std::error::Error + Sync + Send + 'static>),
}

impl std::fmt::Display for DataplaneCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataplaneCommand::SetInit(_) => write!(f, "SetInit"),
            DataplaneCommand::SetConfiguring => write!(f, "SetConfiguring"),
            DataplaneCommand::SetAuth => write!(f, "SetAuth"),
            DataplaneCommand::SetReady => write!(f, "SetReady"),
            DataplaneCommand::SetStarted(_) => write!(f, "SetStarted"),
            DataplaneCommand::SetSubscribing(_) => write!(f, "SetSubscribing"),
            DataplaneCommand::SetUnsubscribing(_) => write!(f, "SetUnsubscribing"),
            DataplaneCommand::SetStopped(_) => write!(f, "SetStopped"),
            DataplaneCommand::SetTerminating(_) => write!(f, "SetTerminating"),
        }
    }
}

#[async_trait::async_trait]
pub trait DataplaneCommandStateMachine: Send + Sync {
    fn dataplane_entity(&self) -> Arc<dyn DataplaneTransfersEntitiesTrait>;
    fn connector_entity(&self) -> Arc<dyn ConnectorInstanceTrait>;
    fn transfer_config(&self) -> Arc<TransferConfig>;
    async fn set_init(&self, command: DataplaneInitCommandTypes) -> Outcome<DataplaneContext> {
        let context = DataplaneContext::from_init(
            self.dataplane_entity().clone(),
            self.connector_entity().clone(),
            self.transfer_config().clone(),
            command,
        )
            .await?;
        Ok(context)
    }
    async fn set_configuring(&self, context: DataplaneContext) -> Outcome<DataplaneContext> {
        Ok(context)
    }
    async fn set_auth(&self, context: DataplaneContext) -> Outcome<DataplaneContext> {
        Ok(context)
    }
    async fn set_ready(&self, mut context: DataplaneContext) -> Outcome<DataplaneContext> {
        let dataplane_urn = Urn::from_str(&*context.dataplane_process().inner.id)?;
        // state
        let new_state = TransferState::Ready;
        // driver.auth
        let dataplane_process = self
            .dataplane_entity()
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
    async fn set_started(&self, mut context: DataplaneContext) -> Outcome<DataplaneContext> {
        let dataplane_urn = Urn::from_str(&*context.dataplane_process().inner.id)?;
        // state
        let new_state = TransferState::Started;
        // driver.auth
        let dataplane_process = self
            .dataplane_entity()
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
    async fn set_subscribing(&self, context: DataplaneContext) -> Outcome<DataplaneContext> {
        Ok(context)
    }
    async fn set_unsubscribing(&self, context: DataplaneContext) -> Outcome<DataplaneContext> {
        Ok(context)
    }
    async fn set_stopped(&self, mut context: DataplaneContext) -> Outcome<DataplaneContext> {
        let dataplane_urn = Urn::from_str(&*context.dataplane_process().inner.id)?;
        // state
        let new_state = TransferState::Stopped;
        // driver.auth
        let dataplane_process = self
            .dataplane_entity()
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

    async fn set_terminating(&self, mut context: DataplaneContext) -> Outcome<DataplaneContext> {
        let dataplane_urn = Urn::from_str(&*context.dataplane_process().inner.id)?;
        // state
        let new_state = TransferState::Terminated;
        // driver.auth
        let dataplane_process = self
            .dataplane_entity()
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
