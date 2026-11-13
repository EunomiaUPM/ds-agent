/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

use crate::engine::dataplane_manager::dataplane_context::DataplaneContext;
use crate::engine::dataplane_manager::dataplane_driver_factory::DataplaneDriverFactoryTrait;
use crate::engine::dataplane_manager::dataplane_runtime::{
    DataplaneRuntime, DeliveryPolicy, RuntimeSecretVault,
};
use crate::entities::dataplane_transfers::{EditDataplaneTransferDto, TransferState};
use crate::errors::DataplaneError;
use crate::services::dataplane_transfers::DataplaneTransferServiceTrait;
use crate::signals::{DataplaneSignal, DataplaneSignalSender};
use crate::DataplaneAddress;
use common::config::services::traits::TransferConfigTrait;
use common::config::services::TransferConfig;
use ymir::oauth::Owner;
use connector::{ConnectorInstanceDto, ConnectorInstanceFacadeTrait, TransferLifetime};
use keystore::SecretStore;
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::Outcome;

/// Command the control plane sends to move a dataplane process through its states.
#[derive(Clone, Debug)]
pub enum DataplaneCommand {
    GetAssociated(DataplaneContinuation),
    SetInit(DataplaneInitCommandTypes),
    SetConfiguring((DataplaneContinuation, DataplaneAddress)),
    SetAuth,
    SetReady,
    SetStarted(DataplaneContinuation),
    SetSubscribing(DataplaneContinuation),
    SetUnsubscribing(DataplaneContinuation),
    SetStopped(DataplaneContinuation),
    SetTerminating(DataplaneContinuation),
}

/// Data needed to create a process, depending on the side it serves.
#[derive(Clone, Debug)]
pub enum DataplaneInitCommandTypes {
    AsProvider {
        /// Owner of the transfer, and so of its dataplane process.
        owner: Owner,
        transfer_process_id: Urn,
        connector_instance: ConnectorInstanceDto,
        direction: DataplaneInitCommandDirection,
    },
    AsConsumer {
        /// Owner of the transfer, and so of its dataplane process.
        owner: Owner,
        transfer_process_id: Urn,
        direction: DataplaneInitCommandDirection,
    },
}

impl DataplaneInitCommandTypes {
    pub fn owner(&self) -> &Owner {
        match self {
            Self::AsProvider { owner, .. } => owner,
            Self::AsConsumer { owner, .. } => owner,
        }
    }
}

/// Pull or push, with the peer's data address when it is already known.
#[derive(Clone, Debug)]
pub enum DataplaneInitCommandDirection {
    Pull {
        data_address: Option<DataplaneAddress>,
    },
    Push {
        data_address: Option<DataplaneAddress>,
    },
}

/// Process a command applies to.
#[derive(Clone, Debug)]
pub struct DataplaneContinuation {
    pub transfer_dto_urn: Urn,
    /// Optional diagnostic or lifecycle reason to store on resulting transition logs.
    pub reason: Option<String>,
}

impl DataplaneContinuation {
    /// Creates a continuation without an explicit caller reason.
    pub fn new(transfer_dto_urn: Urn) -> Self {
        Self {
            transfer_dto_urn,
            reason: None,
        }
    }

    /// Creates a continuation carrying an optional transition reason.
    pub fn with_reason(transfer_dto_urn: Urn, reason: Option<String>) -> Self {
        Self {
            transfer_dto_urn,
            reason,
        }
    }
}

/// Answer to a command; some transitions return the address the peer must use.
#[derive(Debug)]
pub enum DataplaneCommandResponse {
    Ok,
    OkWithAddress(DataplaneAddress),
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
    fn dataplane_service(&self) -> Arc<dyn DataplaneTransferServiceTrait>;
    fn connector_service(&self) -> Arc<dyn ConnectorInstanceFacadeTrait>;
    fn transfer_config(&self) -> Arc<TransferConfig>;
    fn secret_store(&self) -> Option<Arc<dyn SecretStore>>;

    /// The manager's factory, shared so every driver sees the same keystore.
    fn driver_factory(&self) -> Arc<dyn DataplaneDriverFactoryTrait>;

    /// Channel to notify the control plane when an asynchronous hook fails irreversibly.
    fn signal_sender(&self) -> DataplaneSignalSender {
        DataplaneSignalSender::noop()
    }

    /// Emits a `DataplaneSignal::Failed` for the transfer process backing `context`.
    fn emit_failed(&self, context: &DataplaneContext, code: &str, reason: String) {
        if let Ok(transfer_process_id) =
            Urn::from_str(&context.dataplane_process().inner.transfer_process_id)
        {
            self.signal_sender().emit(DataplaneSignal::Failed {
                transfer_process_id,
                code: code.to_string(),
                reason,
            });
        }
    }

    /// Whether this `(Role, InteractionMode)` handler accepts an external command from the
    /// control plane in the process's current state. Internal steps (`SetAuth`, `SetReady`)
    /// are never valid external commands.
    fn accepts_command(&self, command: &DataplaneCommand, context: &DataplaneContext) -> bool;

    /// Creates the driver, configures its proxy and moves the process to `Configuring`.
    async fn configure_driver(&self, context: DataplaneContext) -> Outcome<DataplaneContext> {
        let driver = self.driver_factory().get_or_create_driver(&context)?;
        let mut context = driver.proxy_configurator.configure_proxy(&context).await?;
        let default_retries = self.transfer_config().proxy_max_retries();
        let delivery_policy = match context.connector_instance() {
            Some(connector) => {
                let lifetime = connector.interaction.lifetime();
                let max_retries = connector.interaction.max_retries().unwrap_or(match lifetime {
                    TransferLifetime::Finite => 0,
                    TransferLifetime::Infinite => default_retries,
                });
                DeliveryPolicy {
                    lifetime,
                    max_retries,
                }
            }
            None => DeliveryPolicy {
                lifetime: TransferLifetime::Infinite,
                max_retries: default_retries,
            },
        };
        let runtime = DataplaneRuntime {
            delivery_policy,
            ..DataplaneRuntime::default()
        };
        context.set_driver(driver);
        context.set_runtime(runtime);
        let caller_reason = context.take_transition_reason();
        let mut reason = format!(
            "Configured {} ingress -> {} egress (lifetime={:?}, max_retries={})",
            context.describe_ingress(),
            context.describe_egress(),
            delivery_policy.lifetime,
            delivery_policy.max_retries,
        );
        if let Some(caller) = caller_reason {
            reason = format!("{reason}; {caller}");
        }
        let dataplane_urn = Urn::from_str(&context.dataplane_process().inner.id)?;
        let new_state = TransferState::Configuring;
        let dataplane_process = self
            .dataplane_service()
            .edit(
                &context.engine_actor(),
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
                    reason: Some(reason),
                    ..EditDataplaneTransferDto::default()
                },
            )
            .await?;
        context.set_dataplane_process(dataplane_process);
        Ok(context)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(handler = self.handler_name()))]
    async fn get_associated(&self, mut context: DataplaneContext) -> Outcome<DataplaneContext> {
        context.set_forward_dataplane_address_from_ingress();
        Ok(context)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(handler = self.handler_name()))]
    async fn set_init(&self, context: DataplaneContext) -> Outcome<DataplaneContext> {
        let ctx = self.set_configuring(context).await?;
        let ctx = self.set_auth(ctx).await?;
        let ctx = self.set_ready(ctx).await?;
        Ok(ctx)
    }
    #[tracing::instrument(level = "info", skip_all, err, fields(handler = self.handler_name()))]
    async fn set_configuring(&self, context: DataplaneContext) -> Outcome<DataplaneContext> {
        self.configure_driver(context).await
    }
    #[tracing::instrument(level = "info", skip_all, err, fields(handler = self.handler_name()))]
    async fn set_auth(&self, context: DataplaneContext) -> Outcome<DataplaneContext> {
        // Reuse the driver from context when set_configuring already ran; create from factory
        // otherwise (e.g. when set_auth is called directly in tests or retry scenarios).
        let authenticator = match context.driver() {
            Some(driver) => driver.authenticator.clone(),
            None => {
                self.driver_factory()
                    .get_or_create_driver(&context)?
                    .authenticator
            }
        };
        let mut context = authenticator.authenticate(&context).await?;
        let auth_summary = context.describe_auth();
        let reason = match context.take_transition_reason() {
            Some(r) => format!("Authentication completed ({auth_summary}); {r}"),
            None => format!("Authentication completed ({auth_summary})"),
        };
        let dataplane_urn = Urn::from_str(&context.dataplane_process().inner.id)?;
        let new_state = TransferState::Auth;
        let flow_control = self.export_runtime_flow_control(&context).await?;
        let dataplane_process = self
            .dataplane_service()
            .edit(
                &context.engine_actor(),
                &dataplane_urn,
                &EditDataplaneTransferDto {
                    state: Some(new_state),
                    flow_control,
                    reason: Some(reason),
                    ..EditDataplaneTransferDto::default()
                },
            )
            .await?;
        context.set_dataplane_process(dataplane_process);
        Ok(context)
    }

    async fn export_runtime_flow_control(
        &self,
        context: &DataplaneContext,
    ) -> Outcome<Option<serde_json::Value>> {
        if let (Some(runtime), Some(store)) = (context.runtime(), self.secret_store()) {
            let transfer_id = context.dataplane_process().inner.id.clone();
            let exported = RuntimeSecretVault::new(
                store.as_ref(),
                &context.dataplane_process().inner.user_id,
            )
            .export(runtime, &transfer_id)
            .await?;
            let value = serde_json::to_value(&exported).map_err(|e| {
                DataplaneError::RuntimeSerializationFailed {
                    reason: e.to_string(),
                }
            })?;
            Ok(Some(value))
        } else {
            match context.runtime() {
                None => Ok(None),
                Some(r) => {
                    let value = serde_json::to_value(r).map_err(|e| {
                        DataplaneError::RuntimeSerializationFailed {
                            reason: e.to_string(),
                        }
                    })?;
                    Ok(Some(value))
                }
            }
        }
    }
    #[tracing::instrument(level = "info", skip_all, err, fields(handler = self.handler_name()))]
    async fn set_ready(&self, mut context: DataplaneContext) -> Outcome<DataplaneContext> {
        let dataplane_urn = Urn::from_str(&context.dataplane_process().inner.id)?;
        // state
        let new_state = TransferState::Ready;
        let reason = match context.take_transition_reason() {
            Some(r) => format!(
                "Dataplane ready (handler={}, ingress={}); {r}",
                self.handler_name(),
                context.describe_ingress()
            ),
            None => format!(
                "Dataplane ready (handler={}, ingress={})",
                self.handler_name(),
                context.describe_ingress()
            ),
        };
        // driver.auth
        let dataplane_process = self
            .dataplane_service()
            .edit(
                &context.engine_actor(),
                &dataplane_urn,
                &EditDataplaneTransferDto {
                    state: Some(new_state),
                    reason: Some(reason),
                    ..EditDataplaneTransferDto::default()
                },
            )
            .await?;
        context.set_dataplane_process(dataplane_process);
        context.set_forward_dataplane_address_from_ingress();
        Ok(context)
    }
    #[tracing::instrument(level = "info", skip_all, err, fields(handler = self.handler_name()))]
    async fn set_started(&self, mut context: DataplaneContext) -> Outcome<DataplaneContext> {
        let dataplane_urn = Urn::from_str(&context.dataplane_process().inner.id)?;
        let new_state = TransferState::Started;
        let previous_state = context.dataplane_process().inner.state.clone();
        let caller_reason = context.take_transition_reason();
        let reason = match (previous_state, caller_reason) {
            (TransferState::Subscribing, Some(r)) => format!(
                "Subscription active; transfer started (handler={}, egress={}); {r}",
                self.handler_name(),
                context.describe_egress()
            ),
            (TransferState::Subscribing, None) => format!(
                "Subscription active; transfer started (handler={}, egress={})",
                self.handler_name(),
                context.describe_egress()
            ),
            (_, Some(r)) => r,
            (TransferState::Stopped, None) => format!(
                "Transfer resumed from STOPPED (handler={}, egress={})",
                self.handler_name(),
                context.describe_egress()
            ),
            (_, None) => format!(
                "Transfer started (handler={}, egress={})",
                self.handler_name(),
                context.describe_egress()
            ),
        };
        let flow_control = self.export_runtime_flow_control(&context).await?;
        let dataplane_process = self
            .dataplane_service()
            .edit(
                &context.engine_actor(),
                &dataplane_urn,
                &EditDataplaneTransferDto {
                    state: Some(new_state),
                    flow_control,
                    reason: Some(reason),
                    ..EditDataplaneTransferDto::default()
                },
            )
            .await?;
        context.set_dataplane_process(dataplane_process);
        context.set_forward_dataplane_address_from_ingress();
        Ok(context)
    }
    #[tracing::instrument(level = "info", skip_all, err, fields(handler = self.handler_name()))]
    async fn set_subscribing(&self, mut context: DataplaneContext) -> Outcome<DataplaneContext> {
        if context.dataplane_process().inner.state == TransferState::Started {
            return Ok(context);
        }
        let subscriber = context.driver().and_then(|d| d.subscriber.clone());
        let Some(subscriber) = subscriber else {
            return Ok(context);
        };
        let reason = match context.transition_reason() {
            Some(r) => format!(
                "Subscribing push delivery (handler={}, egress={}); {r}",
                self.handler_name(),
                context.describe_egress()
            ),
            None => format!(
                "Subscribing push delivery (handler={}, egress={})",
                self.handler_name(),
                context.describe_egress()
            ),
        };
        let dataplane_urn = Urn::from_str(&context.dataplane_process().inner.id)?;
        let dataplane_process = self
            .dataplane_service()
            .edit(
                &context.engine_actor(),
                &dataplane_urn,
                &EditDataplaneTransferDto {
                    state: Some(TransferState::Subscribing),
                    reason: Some(reason),
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
                let err_msg = e.to_string();
                self.emit_failed(&context, "DATAPLANE_SUBSCRIBE_FAILED", err_msg.clone());
                let _ = self
                    .terminate_process_with_reason(
                        context,
                        format!("Subscription failed [DATAPLANE_SUBSCRIBE_FAILED]: {err_msg}"),
                    )
                    .await;
                Err(e)
            }
        }
    }
    async fn try_unsubscribe(&self, mut context: DataplaneContext) -> Outcome<DataplaneContext> {
        if matches!(
            context.dataplane_process().inner.state,
            TransferState::Ready | TransferState::Stopped | TransferState::Terminated
        ) {
            return Ok(context);
        }
        let subscriber = context.driver().and_then(|d| d.subscriber.clone());
        let Some(subscriber) = subscriber else {
            return Ok(context);
        };
        let reason = match context.transition_reason() {
            Some(r) => format!(
                "Unsubscribing push delivery (handler={}); {r}",
                self.handler_name()
            ),
            None => format!("Unsubscribing push delivery (handler={})", self.handler_name()),
        };
        let dataplane_urn = Urn::from_str(&context.dataplane_process().inner.id)?;
        let dataplane_process = self
            .dataplane_service()
            .edit(
                &context.engine_actor(),
                &dataplane_urn,
                &EditDataplaneTransferDto {
                    state: Some(TransferState::Unsubscribing),
                    reason: Some(reason),
                    ..EditDataplaneTransferDto::default()
                },
            )
            .await?;
        context.set_dataplane_process(dataplane_process);
        let ctx = subscriber.unsubscribe(&context).await?;
        self.set_stopped(ctx).await
    }
    #[tracing::instrument(level = "info", skip_all, err, fields(handler = self.handler_name()))]
    async fn set_unsubscribing(&self, context: DataplaneContext) -> Outcome<DataplaneContext> {
        match self.try_unsubscribe(context.clone()).await {
            Ok(new_context) => Ok(new_context),
            Err(e) => {
                let err_msg = e.to_string();
                self.emit_failed(&context, "DATAPLANE_UNSUBSCRIBE_FAILED", err_msg.clone());
                let _ = self
                    .terminate_process_with_reason(
                        context,
                        format!("Unsubscribe failed [DATAPLANE_UNSUBSCRIBE_FAILED]: {err_msg}"),
                    )
                    .await;
                Err(e)
            }
        }
    }
    #[tracing::instrument(level = "info", skip_all, err, fields(handler = self.handler_name()))]
    async fn set_stopped(&self, mut context: DataplaneContext) -> Outcome<DataplaneContext> {
        if matches!(
            context.dataplane_process().inner.state,
            TransferState::Stopped | TransferState::Terminated
        ) {
            return Ok(context);
        }
        let dataplane_urn = Urn::from_str(&context.dataplane_process().inner.id)?;
        let new_state = TransferState::Stopped;
        let previous_state = context.dataplane_process().inner.state.clone();
        // Borrow transition_reason so Push::set_terminating still carries it into Terminated.
        let reason = match (previous_state, context.transition_reason()) {
            (TransferState::Unsubscribing, Some(r)) => format!(
                "Stopped after unsubscribing push delivery (handler={}); {r}",
                self.handler_name()
            ),
            (TransferState::Unsubscribing, None) => format!(
                "Stopped after unsubscribing push delivery (handler={})",
                self.handler_name()
            ),
            (_, Some(r)) => r.to_string(),
            (_, None) => format!("Transfer stopped (handler={})", self.handler_name()),
        };
        let flow_control = self.export_runtime_flow_control(&context).await?;
        let dataplane_process = self
            .dataplane_service()
            .edit(
                &context.engine_actor(),
                &dataplane_urn,
                &EditDataplaneTransferDto {
                    state: Some(new_state),
                    flow_control,
                    reason: Some(reason),
                    ..EditDataplaneTransferDto::default()
                },
            )
            .await?;
        context.set_dataplane_process(dataplane_process);
        Ok(context)
    }

    async fn terminate_process_with_reason(
        &self,
        mut context: DataplaneContext,
        reason: String,
    ) -> Outcome<DataplaneContext> {
        context.set_transition_reason(Some(reason));
        self.terminate_process(context).await
    }

    async fn terminate_process(&self, mut context: DataplaneContext) -> Outcome<DataplaneContext> {
        if context.dataplane_process().inner.state == TransferState::Terminated {
            return Ok(context);
        }
        let dataplane_urn = Urn::from_str(&context.dataplane_process().inner.id)?;
        let new_state = TransferState::Terminated;
        let reason = context
            .take_transition_reason()
            .unwrap_or_else(|| format!("Transfer terminated (handler={})", self.handler_name()));
        let dataplane_process = self
            .dataplane_service()
            .edit(
                &context.engine_actor(),
                &dataplane_urn,
                &EditDataplaneTransferDto {
                    state: Some(new_state),
                    reason: Some(reason),
                    ..EditDataplaneTransferDto::default()
                },
            )
            .await?;
        context.set_dataplane_process(dataplane_process);
        if let Some(store) = self.secret_store() {
            let transfer_id = context.dataplane_process().inner.id.clone();
            let _ = RuntimeSecretVault::new(
                store.as_ref(),
                &context.dataplane_process().inner.user_id,
            )
            .cleanup(&transfer_id)
            .await;
        }
        Ok(context)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(handler = self.handler_name()))]
    async fn set_terminating(&self, context: DataplaneContext) -> Outcome<DataplaneContext> {
        self.terminate_process(context).await
    }
}
