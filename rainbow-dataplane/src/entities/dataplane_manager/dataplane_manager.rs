use crate::entities::dataplane_manager::driver_factory::{DataplaneDriver, DataplaneDriverFactory};
use crate::entities::dataplane_manager::{
    DataplaneAddress, DataplaneCommand, DataplaneManagerInput, DataplaneResponse,
};
use crate::entities::dataplane_transfers::{
    DataplaneTransferDto, DataplaneTransfersEntitiesTrait, EditDataplaneTransferDto,
    InteractionMode, NewDataplaneTransferDto, TransferRole, TransferState,
};
use anyhow::anyhow;
use rainbow_connector::{ConnectorInstanceDto, ConnectorInstanceTrait, InteractionConfig};
use std::sync::Arc;
use urn::{Urn, UrnBuilder};
use uuid::Uuid;
use crate::entities::dataplane_manager::config_builder::{DataplaneConfigBuilder, EgressConfig, IngressConfig};

// ─── Context passed to each command handler ───

struct CommandContext<'a> {
    process_id: Urn,
    state: &'a TransferState,
    mode: &'a InteractionMode,
    driver: &'a DataplaneDriver,
    connector: Option<&'a ConnectorInstanceDto>,
}

impl<'a> CommandContext<'a> {
    fn from(
        process: &'a DataplaneTransferDto,
        driver: &'a DataplaneDriver,
        connector: Option<&'a ConnectorInstanceDto>,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            process_id: process.inner.id.parse()?,
            state: &process.inner.state,
            mode: &process.inner.interaction_mode,
            driver,
            connector,
        })
    }

    fn is_state(&self, expected: TransferState) -> bool {
        *self.state == expected
    }

    fn is_push(&self) -> bool {
        *self.mode == InteractionMode::Push
    }
}

// ─── DataplaneManager ───

pub struct DataplaneManager {
    dataplane_entity: Arc<dyn DataplaneTransfersEntitiesTrait>,
    connector_entity: Arc<dyn ConnectorInstanceTrait>,
    driver_factory: Arc<DataplaneDriverFactory>,
}

impl DataplaneManager {
    pub fn new(
        dataplane_entity: Arc<dyn DataplaneTransfersEntitiesTrait>,
        connector_entity: Arc<dyn ConnectorInstanceTrait>,
        driver_factory: Arc<DataplaneDriverFactory>,
    ) -> Self {
        Self {
            dataplane_entity,
            connector_entity,
            driver_factory,
        }
    }

    /// Main entry point: receives a command from the control plane and executes it.
    pub async fn execute_command(
        &self,
        input: &DataplaneManagerInput,
    ) -> anyhow::Result<DataplaneResponse> {
        // 1. Load existing dataplane process (if any)
        let dataplane_process_opt = self
            .dataplane_entity
            .get_dataplane_transfer_by_process_id(&input.transfer_process_id)
            .await
            .map_err(anyhow::Error::from)?;

        // 2. Resolve connector URN → Option<Urn>
        let connector_urn = self.resolve_connector_urn(&dataplane_process_opt, &input.command)?;

        // 3. Fetch connector instance (only if URN is present)
        let connector_instance = self.fetch_connector(&connector_urn).await?;

        // 4. CREATION GUARD: If no process exists, only SetInit is allowed → create it
        if dataplane_process_opt.is_none() {
            self.handle_creation(&input, &connector_urn, &connector_instance)
                .await?;
            // Re-execute: now the process exists, so cmd_init runs (Init → Configuring)
            // and trigger_autonomous_transition chains (Configuring → Auth → Ready)
            return Box::pin(self.execute_command(input)).await;
        }

        // 5. Process is guaranteed to exist from here
        let process = dataplane_process_opt
            .ok_or_else(|| anyhow!("Dataplane process not found after creation check"))?;

        // 6. Build driver (connector is optional)
        let driver = self
            .driver_factory
            .create_driver(&process, connector_instance.as_ref())?;

        let ctx = CommandContext::from(&process, &driver, connector_instance.as_ref())?;

        // 7. Execute the command (state transition + side effects)
        let response = self.handle_command(&input.command, &process, &ctx).await?;

        // 8. Fire autonomous transitions (recursive chain)
        self.trigger_autonomous_transition(&input.transfer_process_id)
            .await?;

        Ok(response)
    }

    // ─── Resolver helpers ───

    fn resolve_connector_urn(
        &self,
        process_opt: &Option<DataplaneTransferDto>,
        command: &DataplaneCommand,
    ) -> anyhow::Result<Option<Urn>> {
        match process_opt {
            None => match command {
                DataplaneCommand::SetInit { connector_instance, .. } => Ok(connector_instance.clone()),
                _ => Err(anyhow!("Cannot execute command without an existing dataplane process")),
            },
            Some(process) => match &process.inner.connector_instance_id {
                Some(id_str) => Ok(Some(id_str.parse::<Urn>()?)),
                None => Ok(None),
            },
        }
    }

    async fn fetch_connector(
        &self,
        connector_urn: &Option<Urn>,
    ) -> anyhow::Result<Option<ConnectorInstanceDto>> {
        match connector_urn {
            Some(urn) => {
                let instance = self
                    .connector_entity
                    .get_instance_by_id(urn)
                    .await?
                    .ok_or_else(|| anyhow!("Connector instance not found for URN: {}", urn))?;
                Ok(Some(instance))
            }
            None => Ok(None),
        }
    }

    // ─── Creation guard ───

    async fn handle_creation(
        &self,
        input: &DataplaneManagerInput,
        connector_urn: &Option<Urn>,
        connector_instance: &Option<ConnectorInstanceDto>,
    ) -> anyhow::Result<DataplaneResponse> {
        if let DataplaneCommand::SetInit { role, data_address, .. } = &input.command {
            let interaction_mode = match connector_instance {
                Some(ci) => match &ci.interaction {
                    InteractionConfig::Pull(_) => InteractionMode::Pull,
                    InteractionConfig::Push(_) => InteractionMode::Push,
                },
                // Consumer has no connector. PUSH mode is signalled by a non-None data_address
                // in the SetInit command (the consumer's ingest address sent to the provider).
                None => match data_address {
                    Some(_) => InteractionMode::Push,
                    None => InteractionMode::Pull,
                },
            };

            let new_id = UrnBuilder::new("dataplane-transfer", Uuid::new_v4().to_string().as_str())
                .build()?;

            let transfer_role = TransferRole::try_from(role.clone())?;
            let config = DataplaneConfigBuilder::from_connector(
                &transfer_role,
                connector_instance.as_ref(),
                &new_id.to_string(),
            );

            self.dataplane_entity
                .create_dataplane_transfer(&NewDataplaneTransferDto {
                    id: Some(new_id),
                    transfer_process_id: input.transfer_process_id.to_string(),
                    role: transfer_role,
                    interaction_mode,
                    state: TransferState::Init,
                    connector_instance_id: connector_urn.clone(),
                    ingress_config: config.ingress,
                    egress_config: config.egress,
                })
                .await?;

            Ok(DataplaneResponse::Ok)
        } else {
            Err(anyhow!("Cannot execute command without an existing process (only SetInit creates)"))
        }
    }

    // ─── Command dispatch ───

    async fn handle_command(
        &self,
        cmd: &DataplaneCommand,
        process: &DataplaneTransferDto,
        ctx: &CommandContext<'_>,
    ) -> anyhow::Result<DataplaneResponse> {
        match cmd {
            DataplaneCommand::SetInit { .. } => self.cmd_init(process, ctx).await,
            DataplaneCommand::SetConfiguring    => Ok(DataplaneResponse::Ok),
            DataplaneCommand::SetAuth           => self.cmd_auth(ctx).await,
            DataplaneCommand::SetReady          => self.cmd_ready(ctx).await,
            DataplaneCommand::SetSubscribing    => self.cmd_subscribing(ctx).await,
            DataplaneCommand::SetStarted        => self.cmd_started(ctx).await,
            DataplaneCommand::SetUnsubscribing  => self.cmd_unsubscribing(ctx).await,
            DataplaneCommand::SetStopped        => self.cmd_stopped(ctx).await,
            DataplaneCommand::SetTerminated     => self.cmd_terminated(ctx).await,
            DataplaneCommand::SetEgress { data_address } => self.cmd_set_egress(data_address, ctx).await,
        }
    }

    // ─── Individual command handlers ───

    /// INIT → CONFIGURING: freeze initial ingress/egress config from connector
    async fn cmd_init(
        &self,
        process: &DataplaneTransferDto,
        ctx: &CommandContext<'_>,
    ) -> anyhow::Result<DataplaneResponse> {
        if ctx.is_state(TransferState::Init) {
            // Config was already computed in handle_creation, just persist and transition
            let builder = DataplaneConfigBuilder::from_process(process);
            self.update_state_and_config(
                &ctx.process_id,
                TransferState::Configuring,
                Some(builder.ingress),
                Some(builder.egress),
            ).await?;
        }
        Ok(DataplaneResponse::Ok)
    }

    /// CONFIGURING → AUTH: driver authenticates
    async fn cmd_auth(&self, ctx: &CommandContext<'_>) -> anyhow::Result<DataplaneResponse> {
        if ctx.is_state(TransferState::Configuring) {
            ctx.driver.auth_driver.perform_auth(ctx.connector).await?;
            self.update_state(&ctx.process_id, TransferState::Auth).await?;
        }
        Ok(DataplaneResponse::Ok)
    }

    /// AUTH → READY
    async fn cmd_ready(&self, ctx: &CommandContext<'_>) -> anyhow::Result<DataplaneResponse> {
        if ctx.is_state(TransferState::Auth) {
            self.update_state(&ctx.process_id, TransferState::Ready).await?;
        }
        Ok(DataplaneResponse::Ok)
    }

    /// READY → SUBSCRIBING (PUSH only): driver subscribes to upstream
    async fn cmd_subscribing(&self, ctx: &CommandContext<'_>) -> anyhow::Result<DataplaneResponse> {
        if ctx.is_state(TransferState::Ready) && ctx.is_push() {
            ctx.driver.lifecycle_driver.perform_subscribe(ctx.connector).await?;
            self.update_state(&ctx.process_id, TransferState::Subscribing).await?;
        }
        Ok(DataplaneResponse::Ok)
    }

    /// READY or STOPPED → STARTED (PULL: direct) or via SUBSCRIBING (PUSH: autonomous)
    async fn cmd_started(&self, ctx: &CommandContext<'_>) -> anyhow::Result<DataplaneResponse> {
        match (ctx.mode, ctx.state) {
            // PULL: Ready → Started directly
            (InteractionMode::Pull, &TransferState::Ready) => {
                self.update_state(&ctx.process_id, TransferState::Started).await?;
            }
            // PULL: Stopped → Started (resume after suspension)
            (InteractionMode::Pull, &TransferState::Stopped) => {
                self.update_state(&ctx.process_id, TransferState::Started).await?;
            }
            // PUSH: Ready → subscribe → Subscribing (autonomous will chain to Started)
            (InteractionMode::Push, &TransferState::Ready) => {
                ctx.driver.lifecycle_driver.perform_subscribe(ctx.connector).await?;
                self.update_state(&ctx.process_id, TransferState::Subscribing).await?;
            }
            // PUSH: Stopped → re-subscribe → Subscribing (resume after suspension)
            (InteractionMode::Push, &TransferState::Stopped) => {
                ctx.driver.lifecycle_driver.perform_subscribe(ctx.connector).await?;
                self.update_state(&ctx.process_id, TransferState::Subscribing).await?;
            }
            // PUSH: Subscribing → Started (autonomous transition target)
            (InteractionMode::Push, &TransferState::Subscribing) => {
                self.update_state(&ctx.process_id, TransferState::Started).await?;
            }
            _ => {}
        }
        Ok(DataplaneResponse::Ok)
    }

    /// STARTED → UNSUBSCRIBING (PUSH only): driver unsubscribes
    async fn cmd_unsubscribing(&self, ctx: &CommandContext<'_>) -> anyhow::Result<DataplaneResponse> {
        if ctx.is_state(TransferState::Started) && ctx.is_push() {
            ctx.driver.lifecycle_driver.perform_unsubscribe(ctx.connector).await?;
            self.update_state(&ctx.process_id, TransferState::Unsubscribing).await?;
        }
        Ok(DataplaneResponse::Ok)
    }

    /// STARTED → STOPPED (PULL: direct) or STARTED → UNSUBSCRIBING → STOPPED (PUSH: via autonomous)
    async fn cmd_stopped(&self, ctx: &CommandContext<'_>) -> anyhow::Result<DataplaneResponse> {
        match (ctx.mode, ctx.state) {
            // PULL: Started → Stopped directly
            (InteractionMode::Pull, &TransferState::Started) => {
                self.update_state(&ctx.process_id, TransferState::Stopped).await?;
            }
            // PUSH: Started → unsubscribe → Unsubscribing (autonomous will chain to Stopped)
            (InteractionMode::Push, &TransferState::Started) => {
                ctx.driver.lifecycle_driver.perform_unsubscribe(ctx.connector).await?;
                self.update_state(&ctx.process_id, TransferState::Unsubscribing).await?;
            }
            // PUSH: Unsubscribing → Stopped (autonomous transition target)
            (InteractionMode::Push, &TransferState::Unsubscribing) => {
                self.update_state(&ctx.process_id, TransferState::Stopped).await?;
            }
            _ => {}
        }
        Ok(DataplaneResponse::Ok)
    }

    /// ANY → TERMINATED: cleanup (emergency unsubscribe if PUSH active)
    async fn cmd_terminated(&self, ctx: &CommandContext<'_>) -> anyhow::Result<DataplaneResponse> {
        if ctx.is_push() {
            match ctx.state {
                TransferState::Started | TransferState::Subscribing => {
                    let _ = ctx.driver.lifecycle_driver.perform_unsubscribe(ctx.connector).await;
                }
                _ => {}
            }
        }
        self.update_state(&ctx.process_id, TransferState::Terminated).await?;
        Ok(DataplaneResponse::Ok)
    }

    // ─── Autonomous transitions (recursive chain) ───

    async fn trigger_autonomous_transition(
        &self,
        transfer_process_id: &Urn,
    ) -> anyhow::Result<()> {
        let current = self
            .dataplane_entity
            .get_dataplane_transfer_by_process_id(transfer_process_id)
            .await?;

        let process = match current {
            Some(p) => p,
            None => return Ok(()),
        };

        let next_command = match process.inner.state {
            TransferState::Configuring   => Some(DataplaneCommand::SetAuth),
            TransferState::Auth          => Some(DataplaneCommand::SetReady),
            TransferState::Subscribing   => Some(DataplaneCommand::SetStarted),
            TransferState::Unsubscribing => Some(DataplaneCommand::SetStopped),
            _ => None,
        };

        if let Some(command) = next_command {
            let next_input = DataplaneManagerInput {
                transfer_process_id: transfer_process_id.clone(),
                command,
            };
            Box::pin(self.execute_command(&next_input)).await?;
        }

        Ok(())
    }

    // ─── Persistence helpers ───

    async fn update_state(
        &self,
        process_id: &Urn,
        new_state: TransferState,
    ) -> anyhow::Result<DataplaneTransferDto> {
        self.dataplane_entity
            .put_dataplane_transfer_by_id(
                process_id,
                &EditDataplaneTransferDto {
                    state: Some(new_state),
                    ..Default::default()
                },
            )
            .await
    }

    async fn update_state_and_config(
        &self,
        process_id: &Urn,
        new_state: TransferState,
        ingress: Option<serde_json::Value>,
        egress: Option<serde_json::Value>,
    ) -> anyhow::Result<DataplaneTransferDto> {
        self.dataplane_entity
            .put_dataplane_transfer_by_id(
                process_id,
                &EditDataplaneTransferDto {
                    state: Some(new_state),
                    ingress_config: ingress,
                    egress_config: egress,
                    ..Default::default()
                },
            )
            .await
    }

    /// SetEgress: update egress config from a DataplaneAddress.
    async fn cmd_set_egress(
        &self,
        data_address: &DataplaneAddress,
        ctx: &CommandContext<'_>,
    ) -> anyhow::Result<DataplaneResponse> {
        let egress = EgressConfig::HttpProxy {
            url: data_address.endpoint.clone(),
        };
        self.dataplane_entity
            .put_dataplane_transfer_by_id(
                &ctx.process_id,
                &EditDataplaneTransferDto {
                    egress_config: Some(serde_json::to_value(egress).unwrap_or_default()),
                    ..Default::default()
                },
            )
            .await?;
        Ok(DataplaneResponse::Ok)
    }

    // ─── Public query helpers ───

    /// Returns the proxy listener path stored in `ingress_config` for a dataplane process.
    /// Returns `None` if the process does not exist or the ingress is not an HttpListener
    /// (e.g. PUSH Provider whose ingress is a connector callback).
    pub async fn get_ingress_address(
        &self,
        transfer_id: &Urn,
    ) -> anyhow::Result<Option<DataplaneAddress>> {
        let Some(process) = self
            .dataplane_entity
            .get_dataplane_transfer_by_process_id(transfer_id)
            .await?
        else {
            return Ok(None);
        };
        let ingress: IngressConfig = serde_json::from_value(process.inner.ingress_config)
            .map_err(|e| anyhow!("Failed to parse ingress_config: {}", e))?;
        match ingress {
            IngressConfig::HttpListener { path } => Ok(Some(DataplaneAddress {
                endpoint_type: "HttpProxy".to_string(),
                endpoint: path,
                authorization_type: None,
                authorization: None,
            })),
            IngressConfig::Connector { .. } => Ok(None),
        }
    }

    /// Check if the transfer is in PULL mode.
    pub async fn is_pull(&self, transfer_id: &Urn) -> anyhow::Result<bool> {
        if let Some(process) = self
            .dataplane_entity
            .get_dataplane_transfer_by_process_id(transfer_id)
            .await?
        {
            Ok(process.inner.interaction_mode == InteractionMode::Pull)
        } else {
            Ok(false)
        }
    }
}
