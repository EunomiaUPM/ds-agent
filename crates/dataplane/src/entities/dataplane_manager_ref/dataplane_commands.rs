use crate::entities::dataplane_drivers::DriverPubSubTrait;
use crate::entities::dataplane_manager_ref::dataplane_context::DataplaneContext;
use crate::entities::dataplane_manager_ref::dataplane_driver_factory::{
    DataplaneDriverFactory, DataplaneDriverFactoryTrait,
};
use crate::entities::dataplane_transfers::{EditDataplaneTransferDto, TransferState};
use crate::{DataplaneAddress, DataplaneTransfersEntitiesTrait};
use common::config::services::TransferConfig;
use connector::{ConnectorInstanceDto, ConnectorInstanceTrait};
use serde_json::json;
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::Outcome;

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

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub enum DataplaneInitCommandDirection {
    Pull { data_address: DataplaneAddress },
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
    fn handler_name(&self) -> &'static str;
    fn dataplane_entity(&self) -> Arc<dyn DataplaneTransfersEntitiesTrait>;
    fn connector_entity(&self) -> Arc<dyn ConnectorInstanceTrait>;
    fn transfer_config(&self) -> Arc<TransferConfig>;
    async fn set_init(&self, context: DataplaneContext) -> Outcome<DataplaneContext> {
        dbg!(&context);
        let ctx = self.set_configuring(context).await?;
        dbg!(&ctx);
        let ctx = self.set_auth(ctx).await?;
        dbg!(&ctx);
        let ctx = self.set_ready(ctx).await?;
        dbg!(&ctx);
        Ok(ctx)
    }
    async fn set_configuring(&self, context: DataplaneContext) -> Outcome<DataplaneContext> {
        // driver
        let driver = DataplaneDriverFactory.get_or_create_driver(&context)?;
        // proxy
        let mut context = driver.proxy_configurator.configure_proxy(&context).await?;
        context.set_driver(driver);
        context.set_runtime(json!({}));
        // dataplane
        let dataplane_urn = Urn::from_str(&*context.dataplane_process().inner.id)?;
        let new_state = TransferState::Configuring;
        let dataplane_process = self
            .dataplane_entity()
            .put_dataplane_transfer_by_id(
                &dataplane_urn,
                &EditDataplaneTransferDto {
                    state: Some(new_state),
                    // METER INGRESS
                    ..EditDataplaneTransferDto::default()
                },
            )
            .await?;
        context.set_dataplane_process(dataplane_process);
        Ok(context)
    }
    async fn set_auth(&self, mut context: DataplaneContext) -> Outcome<DataplaneContext> {
        let driver = DataplaneDriverFactory.get_or_create_driver(&context)?;
        driver.authenticator.authenticate(&context).await?;
        let dataplane_urn = Urn::from_str(&*context.dataplane_process().inner.id)?;
        let new_state = TransferState::Auth;
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
    async fn set_subscribing(&self, mut context: DataplaneContext) -> Outcome<DataplaneContext> {
        match context.driver().and_then(|d| d.subscriber.clone()) {
            None => Ok(context),
            Some(subscriber) => {
                let dataplane_urn = Urn::from_str(&*context.dataplane_process().inner.id)?;
                // state
                let new_state = TransferState::Subscribing;
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
                let ctx = subscriber.subscribe(&context).await?;
                let new_context = self.set_started(ctx).await?;
                Ok(new_context)
            }
        }
    }
    async fn set_unsubscribing(&self, mut context: DataplaneContext) -> Outcome<DataplaneContext> {
        match context.driver().and_then(|d| d.subscriber.clone()) {
            None => Ok(context),
            Some(subscriber) => {
                let dataplane_urn = Urn::from_str(&*context.dataplane_process().inner.id)?;
                // state
                let new_state = TransferState::Unsubscribing;
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
                let ctx = subscriber.subscribe(&context).await?;
                let new_context = self.set_stopped(ctx).await?;
                Ok(new_context)
            }
        }
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
