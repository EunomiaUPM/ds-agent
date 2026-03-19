use crate::entities::dataplane_manager::config_builder::{
    DataplaneConfigBuilder, EgressConfig, IngressConfig,
};
use crate::entities::dataplane_manager::dataplane_commands::CommandContext;
use crate::entities::dataplane_manager::driver_factory::{DataplaneDriver, DataplaneDriverFactory};
use crate::entities::dataplane_manager::{
    DataplaneAddress, DataplaneCommand, DataplaneManagerInput, DataplaneResponse,
};
use crate::entities::dataplane_transfers::{
    DataplaneTransferDto, DataplaneTransfersEntitiesTrait, EditDataplaneTransferDto,
    InteractionMode, NewDataplaneTransferDto, TransferRole, TransferState,
};
use crate::DataplaneInitCommandType;
use anyhow::anyhow;
use rainbow_connector::{ConnectorInstanceDto, ConnectorInstanceTrait, InteractionConfig};
use std::sync::Arc;
use urn::{Urn, UrnBuilder};
use uuid::Uuid;

// ─── DataplaneManager ───

pub struct DataplaneManager {
    pub(super) dataplane_entity: Arc<dyn DataplaneTransfersEntitiesTrait>,
    pub(super) connector_entity: Arc<dyn ConnectorInstanceTrait>,
    pub(super) driver_factory: Arc<DataplaneDriverFactory>,
}

impl DataplaneManager {
    pub fn new(
        dataplane_entity: Arc<dyn DataplaneTransfersEntitiesTrait>,
        connector_entity: Arc<dyn ConnectorInstanceTrait>,
        driver_factory: Arc<DataplaneDriverFactory>,
    ) -> Self {
        Self { dataplane_entity, connector_entity, driver_factory }
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
        // If still no process -> If setInit, connectorUrn in message,
        // If process -> process has connectorUrn in tuple
        let connector_urn = self.resolve_connector_urn(&dataplane_process_opt, &input.command)?;

        // 3. Fetch connector instance (only if URN is present)
        let connector_instance = self.fetch_connector(&connector_urn).await?;

        // 4. CREATION GUARD: If no process exists, only SetInit is allowed → create it
        if dataplane_process_opt.is_none() {
            self.handle_creation(&input, &connector_urn, &connector_instance).await?;
            // Re-execute: now the process exists, so cmd_init runs (Init → Configuring)
            // and trigger_autonomous_transition chains (Configuring → Auth → Ready)
            return Box::pin(self.execute_command(input)).await;
        }

        // 5. Process is guaranteed to exist from here
        let process = dataplane_process_opt
            .ok_or_else(|| anyhow!("Dataplane process not found after creation check"))?;

        // 6. Build driver (connector is optional)
        let driver = self.driver_factory.create_driver(&process, connector_instance.as_ref())?;

        // 7. Creates context for avoiding prop drilling
        let ctx = CommandContext::from_connector(&process, &driver, connector_instance.as_ref())?;

        // 8. Execute the command (state transition + side effects)
        let response = self.handle_command(&input.command, &process, &ctx).await?;

        // 9. Fire autonomous transitions (recursive chain)
        self.trigger_autonomous_transition(&input.transfer_process_id).await?;

        Ok(response)
    }

    // ─── Resolver helpers ───

    /// ConnectorUrn is saved in Dataplane Process, or comes in command on SetInit
    /// We resolve both options here and return Urn
    fn resolve_connector_urn(
        &self,
        process_opt: &Option<DataplaneTransferDto>,
        command: &DataplaneCommand,
    ) -> anyhow::Result<Option<Urn>> {
        match process_opt {
            None => match command {
                DataplaneCommand::SetInit(role) => match role {
                    DataplaneInitCommandType::Provider { connector_instance, .. } => {
                        Ok(Some(connector_instance.clone()))
                    }
                    DataplaneInitCommandType::Consumer { .. } => {
                        Err(anyhow!("Consumer role shouldn't ever have a Connector instance"))
                    }
                },

                _ => Err(anyhow!(
                    "Cannot execute command without an existing dataplane process"
                )),
            },
            Some(process) => match &process.inner.connector_instance_id {
                Some(id_str) => Ok(Some(id_str.parse::<Urn>()?)),
                None => Ok(None),
            },
        }
    }

    /// Knowing ConnectorUrn, we fetch the whole spec of the connector to hand it over to
    /// the driver factory, so Dataplane knows how to connect to final data systems
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

    // ─── Command dispatch ───

    async fn handle_command(
        &self,
        cmd: &DataplaneCommand,
        process: &DataplaneTransferDto,
        ctx: &CommandContext<'_>,
    ) -> anyhow::Result<DataplaneResponse> {
        match cmd {
            DataplaneCommand::SetInit { .. } => self.cmd_init(process, ctx).await,
            DataplaneCommand::SetConfiguring => Ok(DataplaneResponse::Ok),
            DataplaneCommand::SetAuth => self.cmd_auth(ctx).await,
            DataplaneCommand::SetReady => self.cmd_ready(ctx).await,
            DataplaneCommand::SetSubscribing => self.cmd_subscribing(ctx).await,
            DataplaneCommand::SetStarted => self.cmd_started(ctx).await,
            DataplaneCommand::SetUnsubscribing => self.cmd_unsubscribing(ctx).await,
            DataplaneCommand::SetStopped => self.cmd_stopped(ctx).await,
            DataplaneCommand::SetTerminated => self.cmd_terminated(ctx).await,
            DataplaneCommand::SetEgress { data_address } => {
                self.cmd_set_egress(data_address, ctx).await
            }
        }
    }

    // ─── Autonomous transitions (recursive chain) ───

    async fn trigger_autonomous_transition(&self, transfer_process_id: &Urn) -> anyhow::Result<()> {
        let current =
            self.dataplane_entity.get_dataplane_transfer_by_process_id(transfer_process_id).await?;

        let process = match current {
            Some(p) => p,
            None => return Ok(()),
        };

        let next_command = match process.inner.state {
            TransferState::Configuring => Some(DataplaneCommand::SetAuth),
            TransferState::Auth => Some(DataplaneCommand::SetReady),
            TransferState::Subscribing => Some(DataplaneCommand::SetStarted),
            TransferState::Unsubscribing => Some(DataplaneCommand::SetStopped),
            _ => None,
        };

        if let Some(command) = next_command {
            let next_input =
                DataplaneManagerInput { transfer_process_id: transfer_process_id.clone(), command };
            Box::pin(self.execute_command(&next_input)).await?;
        }

        Ok(())
    }

    // ─── Public query helpers ───

    /// Returns the proxy listener path stored in `ingress_config` for a dataplane process.
    /// Returns `None` if the process does not exist or the ingress is not an HttpListener
    /// (e.g. PUSH Provider whose ingress is a connector callback).
    pub async fn get_ingress_address(
        &self,
        transfer_id: &Urn,
    ) -> anyhow::Result<Option<DataplaneAddress>> {
        let Some(process) =
            self.dataplane_entity.get_dataplane_transfer_by_process_id(transfer_id).await?
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
        if let Some(process) =
            self.dataplane_entity.get_dataplane_transfer_by_process_id(transfer_id).await?
        {
            Ok(process.inner.interaction_mode == InteractionMode::Pull)
        } else {
            Ok(false)
        }
    }
}
