use crate::entities::dataplane_drivers::DriverPubSubTrait;
use crate::entities::dataplane_manager::dataplane_context::DataplaneContext;
use crate::entities::dataplane_manager::dataplane_driver_factory::{
    DataplaneDriverFactory, DataplaneDriverFactoryTrait,
};
use crate::entities::dataplane_manager::dataplane_runtime::{DataplaneRuntime, RuntimeSecretVault};
use crate::entities::dataplane_transfers::{EditDataplaneTransferDto, TransferState};
use crate::{DataplaneAddress, DataplaneTransfersEntitiesTrait};
use common::config::services::TransferConfig;
use connector::{ConnectorInstanceDto, ConnectorInstanceTrait};
use keystore::SecretStore;
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::Outcome;

#[derive(Clone, Debug)]
pub enum DataplaneCommand {
    GetAssociated(DataplaneContinuation),
    SetInit(DataplaneInitCommandTypes),
    SetConfiguring(Option<(DataplaneContinuation, DataplaneAddress)>), // just in case egress is configured from outside
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
    Pull {
        data_address: Option<DataplaneAddress>,
    },
    Push {
        data_address: Option<DataplaneAddress>,
    },
}

#[derive(Clone, Debug)]
pub struct DataplaneContinuation {
    pub transfer_dto_urn: Urn,
}

#[derive(Debug)]
pub enum DataplaneCommandResponse {
    Ok,
    OkWithAddress(DataplaneAddress),
    Err(Box<dyn std::error::Error + Sync + Send + 'static>),
}

impl std::fmt::Display for DataplaneCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataplaneCommand::GetAssociated(_) => write!(f, "GetAssociated"),
            DataplaneCommand::SetInit(_) => write!(f, "SetInit"),
            DataplaneCommand::SetConfiguring(_) => write!(f, "SetConfiguring"),
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
    fn secret_store(&self) -> Option<Arc<dyn SecretStore>>;

    async fn get_associated(&self, mut context: DataplaneContext) -> Outcome<DataplaneContext> {
        context.set_forward_dataplane_address_from_ingress();
        Ok(context)
    }

    async fn set_init(&self, context: DataplaneContext) -> Outcome<DataplaneContext> {
        let ctx = self.set_configuring(context).await?;
        let ctx = self.set_auth(ctx).await?;
        let ctx = self.set_ready(ctx).await?;
        Ok(ctx)
    }
    async fn set_configuring(&self, context: DataplaneContext) -> Outcome<DataplaneContext> {
        set_configuring_helper(self.dataplane_entity(), context).await
    }
    async fn set_auth(&self, context: DataplaneContext) -> Outcome<DataplaneContext> {
        let driver = DataplaneDriverFactory::new().get_or_create_driver(&context)?;
        // Use the context returned by authenticate — it carries the resolved runtime.
        let mut context = driver.authenticator.authenticate(&context).await?;
        let dataplane_urn = Urn::from_str(&*context.dataplane_process().inner.id)?;
        let new_state = TransferState::Auth;
        let flow_control =
            if let (Some(runtime), Some(store)) = (context.runtime(), self.secret_store()) {
                let transfer_id = context.dataplane_process().inner.id.clone();
                let exported = RuntimeSecretVault::new(store.as_ref())
                    .export(runtime, &transfer_id)
                    .await?;
                serde_json::to_value(&exported).ok()
            } else {
                context.runtime().and_then(|r| serde_json::to_value(r).ok())
            };
        let dataplane_process = self
            .dataplane_entity()
            .put_dataplane_transfer_by_id(
                &dataplane_urn,
                &EditDataplaneTransferDto {
                    state: Some(new_state),
                    flow_control,
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
        context.set_forward_dataplane_address_from_ingress();
        Ok(context)
    }
    async fn set_started(&self, mut context: DataplaneContext) -> Outcome<DataplaneContext> {
        let dataplane_urn = Urn::from_str(&*context.dataplane_process().inner.id)?;
        let new_state = TransferState::Started;
        let flow_control =
            if let (Some(runtime), Some(store)) = (context.runtime(), self.secret_store()) {
                let transfer_id = context.dataplane_process().inner.id.clone();
                let exported = RuntimeSecretVault::new(store.as_ref())
                    .export(runtime, &transfer_id)
                    .await?;
                serde_json::to_value(&exported).ok()
            } else {
                context.runtime().and_then(|r| serde_json::to_value(r).ok())
            };
        let dataplane_process = self
            .dataplane_entity()
            .put_dataplane_transfer_by_id(
                &dataplane_urn,
                &EditDataplaneTransferDto {
                    state: Some(new_state),
                    flow_control,
                    ..EditDataplaneTransferDto::default()
                },
            )
            .await?;
        context.set_dataplane_process(dataplane_process);
        context.set_forward_dataplane_address_from_ingress();
        Ok(context)
    }
    async fn set_subscribing(&self, mut context: DataplaneContext) -> Outcome<DataplaneContext> {
        if context.dataplane_process().inner.state == TransferState::Started {
            return Ok(context);
        }
        let subscriber = context.driver().and_then(|d| d.subscriber.clone());
        let Some(subscriber) = subscriber else {
            return Ok(context);
        };
        let dataplane_urn = Urn::from_str(&*context.dataplane_process().inner.id)?;
        let dataplane_process = self
            .dataplane_entity()
            .put_dataplane_transfer_by_id(
                &dataplane_urn,
                &EditDataplaneTransferDto {
                    state: Some(TransferState::Subscribing),
                    ..EditDataplaneTransferDto::default()
                },
            )
            .await?;
        context.set_dataplane_process(dataplane_process);
        match subscriber.subscribe(&context).await {
            Ok(ctx) => {
                let new_context = self.set_started(ctx).await?;
                Ok(new_context)
            }
            Err(e) => {
                let _ = self.set_terminating(context).await;
                Err(e)
            }
        }
    }
    async fn set_unsubscribing(&self, mut context: DataplaneContext) -> Outcome<DataplaneContext> {
        if context.dataplane_process().inner.state == TransferState::Stopped {
            return Ok(context);
        }
        let subscriber = context.driver().and_then(|d| d.subscriber.clone());
        let Some(subscriber) = subscriber else {
            return Ok(context);
        };
        let dataplane_urn = Urn::from_str(&*context.dataplane_process().inner.id)?;
        let dataplane_process = self
            .dataplane_entity()
            .put_dataplane_transfer_by_id(
                &dataplane_urn,
                &EditDataplaneTransferDto {
                    state: Some(TransferState::Unsubscribing),
                    ..EditDataplaneTransferDto::default()
                },
            )
            .await?;
        context.set_dataplane_process(dataplane_process);
        match subscriber.unsubscribe(&context).await {
            Ok(ctx) => {
                let new_context = self.set_stopped(ctx).await?;
                Ok(new_context)
            }
            Err(e) => {
                let _ = self.set_terminating(context).await;
                Err(e)
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
        let new_state = TransferState::Terminated;
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
        if let Some(store) = self.secret_store() {
            let transfer_id = context.dataplane_process().inner.id.clone();
            let _ = RuntimeSecretVault::new(store.as_ref())
                .cleanup(&transfer_id)
                .await;
        }
        Ok(context)
    }
}

pub(crate) async fn set_configuring_helper(
    dp_trait: Arc<dyn DataplaneTransfersEntitiesTrait>,
    context: DataplaneContext,
) -> Outcome<DataplaneContext> {
    // driver
    let driver = DataplaneDriverFactory::new().get_or_create_driver(&context)?;
    // proxy
    let mut context = driver.proxy_configurator.configure_proxy(&context).await?;
    // runtime
    let runtime = DataplaneRuntime::default();
    // setters
    context.set_driver(driver);
    context.set_runtime(runtime);
    // dataplane
    let dataplane_urn = Urn::from_str(&*context.dataplane_process().inner.id)?;
    let new_state = TransferState::Configuring;
    let dataplane_process = dp_trait
        .put_dataplane_transfer_by_id(
            &dataplane_urn,
            &EditDataplaneTransferDto {
                state: Some(new_state),
                ingress_config: context
                    .proxy()
                    .map(|p| p.ingress().clone())
                    .and_then(|p| serde_json::to_value(p).ok()),
                egress_config: context
                    .proxy()
                    .map(|p| p.egress().clone())
                    .and_then(|p| serde_json::to_value(p).ok()),
                ..EditDataplaneTransferDto::default()
            },
        )
        .await?;
    context.set_dataplane_process(dataplane_process);
    Ok(context)
}
