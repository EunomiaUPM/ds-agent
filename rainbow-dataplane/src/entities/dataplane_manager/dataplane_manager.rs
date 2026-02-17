use crate::entities::dataplane_manager::driver_factory::{DataplaneDriver, DataplaneDriverFactory};
use crate::entities::dataplane_manager::{
    DataplaneCommand, DataplaneManagerInput, DataplaneResponse,
};
use crate::entities::dataplane_transfers::{
    DataplaneTransferDto, DataplaneTransfersEntitiesTrait, EditDataplaneTransferDto,
    InteractionMode, NewDataplaneTransferDto, TransferRole, TransferState,
};
use anyhow::anyhow;
use rainbow_connector::{ConnectorInstanceDto, ConnectorInstanceTrait, InteractionConfig};
use std::sync::Arc;
use urn::Urn;
use crate::entities::dataplane_manager::config_builder::DataplaneConfigBuilder;

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
        let connector_urn: Option<Urn> = match &dataplane_process_opt {
            None => match &input.command {
                DataplaneCommand::SetInit { connector_instance, .. } => connector_instance.clone(),
                _ => {
                    return Err(anyhow!(
                        "Cannot execute {:?} without an existing dataplane process",
                        "command"
                    ))
                }
            },
            Some(process) => match &process.inner.connector_instance_id {
                Some(id_str) => Some(id_str.parse::<Urn>()?),
                None => None,
            },
        };

        // 3. Fetch connector instance (only if URN is present)
        let connector_instance: Option<ConnectorInstanceDto> = match &connector_urn {
            Some(urn) => {
                let instance = self
                    .connector_entity
                    .get_instance_by_id(urn)
                    .await?
                    .ok_or_else(|| anyhow!("Connector instance not found for URN: {}", urn))?;
                Some(instance)
            }
            None => None,
        };

        // 4. CREATION GUARD: If no process exists, only SetInit is allowed → create it
        if dataplane_process_opt.is_none() {
            if let DataplaneCommand::SetInit {
                role,
                connector_instance: _,
                data_address: _,
            } = &input.command
            {
                let interaction_mode = match &connector_instance {
                    Some(ci) => match &ci.interaction {
                        InteractionConfig::Pull(_) => InteractionMode::Pull,
                        InteractionConfig::Push(_) => InteractionMode::Push,
                    },
                    None => InteractionMode::Pull,
                };

                let _new = self
                    .dataplane_entity
                    .create_dataplane_transfer(&NewDataplaneTransferDto {
                        id: None,
                        transfer_process_id: input.transfer_process_id.to_string(),
                        role: TransferRole::try_from(role.clone())?,
                        interaction_mode,
                        state: TransferState::Init,
                        connector_instance_id: connector_urn.clone(),
                        ingress_config: serde_json::Value::Null,
                        egress_config: serde_json::Value::Null,
                    })
                    .await?;

                return Ok(DataplaneResponse::Ok);
            }
        }

        // 5. Process is guaranteed to exist from here
        let process = dataplane_process_opt
            .ok_or_else(|| anyhow!("Dataplane process not found after creation check"))?;

        // 6. Build driver (connector is optional)
        let driver = self
            .driver_factory
            .create_driver(&process, connector_instance.as_ref())?;

        // 7. Execute the command (state transition + side effects)
        let response = self
            .handle_command(&input.command, &process, &driver, connector_instance.as_ref())
            .await?;

        // 8. Fire autonomous transitions (recursive chain)
        self.trigger_autonomous_transition(&process, &input.transfer_process_id)
            .await?;

        Ok(response)
    }

    // ─── handle_command: one command → one state transition + action ───

    async fn handle_command(
        &self,
        cmd: &DataplaneCommand,
        process: &DataplaneTransferDto,
        driver: &DataplaneDriver,
        connector: Option<&ConnectorInstanceDto>,
    ) -> anyhow::Result<DataplaneResponse> {
        let state = &process.inner.state;
        let mode = &process.inner.interaction_mode;
        let process_id: Urn = process.inner.id.parse()?;

        match cmd {
            // ── SetInit on existing process: INIT → CONFIGURING ──
            DataplaneCommand::SetInit { .. } => {
                if *state == TransferState::Init {
                    let builder = DataplaneConfigBuilder::from_process(process);
                    self.update_state_and_config(
                        &process_id,
                        TransferState::Configuring,
                        Some(builder.ingress),
                        Some(builder.egress),
                    ).await?;
                }
                Ok(DataplaneResponse::Ok)
            }

            // ── SetConfiguring: explicit (reserved for future use) ──
            DataplaneCommand::SetConfiguring => {
                Ok(DataplaneResponse::Ok)
            }

            // ── SetAuth: CONFIGURING → AUTH (driver authenticates) ──
            DataplaneCommand::SetAuth => {
                if *state == TransferState::Configuring {
                    driver.auth_driver.perform_auth(connector).await?;
                    self.update_state(&process_id, TransferState::Auth).await?;
                }
                Ok(DataplaneResponse::Ok)
            }

            // ── SetReady: AUTH → READY ──
            DataplaneCommand::SetReady => {
                if *state == TransferState::Auth {
                    self.update_state(&process_id, TransferState::Ready).await?;
                }
                Ok(DataplaneResponse::Ok)
            }

            // ── SetSubscribing: READY → SUBSCRIBING (PUSH only, driver subscribes) ──
            DataplaneCommand::SetSubscribing => {
                if *state == TransferState::Ready && *mode == InteractionMode::Push {
                    driver.lifecycle_driver.perform_subscribe(connector).await?;
                    self.update_state(&process_id, TransferState::Subscribing).await?;
                }
                Ok(DataplaneResponse::Ok)
            }

            // ── SetStarted: opens data relay ──
            DataplaneCommand::SetStarted => {
                let valid = match mode {
                    InteractionMode::Pull => *state == TransferState::Ready,
                    InteractionMode::Push => *state == TransferState::Subscribing,
                };
                if valid {
                    self.update_state(&process_id, TransferState::Started).await?;
                }
                Ok(DataplaneResponse::Ok)
            }

            // ── SetUnsubscribing: STARTED → UNSUBSCRIBING (PUSH only, driver unsubscribes) ──
            DataplaneCommand::SetUnsubscribing => {
                if *state == TransferState::Started && *mode == InteractionMode::Push {
                    driver.lifecycle_driver.perform_unsubscribe(connector).await?;
                    self.update_state(&process_id, TransferState::Unsubscribing).await?;
                }
                Ok(DataplaneResponse::Ok)
            }

            // ── SetStopped: pauses data relay ──
            DataplaneCommand::SetStopped => {
                let valid = match mode {
                    InteractionMode::Pull => *state == TransferState::Started,
                    InteractionMode::Push => *state == TransferState::Unsubscribing,
                };
                if valid {
                    self.update_state(&process_id, TransferState::Stopped).await?;
                }
                Ok(DataplaneResponse::Ok)
            }

            // ── SetTerminated: cleanup and final state ──
            DataplaneCommand::SetTerminated => {
                // If PUSH and in an active state, unsubscribe first
                if *mode == InteractionMode::Push {
                    match state {
                        TransferState::Started | TransferState::Subscribing => {
                            let _ = driver.lifecycle_driver.perform_unsubscribe(connector).await;
                        }
                        _ => {}
                    }
                }
                self.update_state(&process_id, TransferState::Terminated).await?;
                Ok(DataplaneResponse::Ok)
            }
        }
    }

    // ─── trigger_autonomous_transition: chains automatic steps ───

    async fn trigger_autonomous_transition(
        &self,
        old_process: &DataplaneTransferDto,
        transfer_process_id: &Urn,
    ) -> anyhow::Result<()> {
        // Reload the process from DB to see the NEW state after handle_command
        let current = self
            .dataplane_entity
            .get_dataplane_transfer_by_process_id(transfer_process_id)
            .await?;

        let process = match current {
            Some(p) => p,
            None => return Ok(()), // Process was deleted or not found
        };

        let next_command = match process.inner.state {
            // CONFIGURING → AUTH (autonomous)
            TransferState::Configuring => Some(DataplaneCommand::SetAuth),
            // AUTH → READY (autonomous)
            TransferState::Auth => Some(DataplaneCommand::SetReady),
            // SUBSCRIBING → STARTED (autonomous, PUSH only)
            TransferState::Subscribing => Some(DataplaneCommand::SetStarted),
            // UNSUBSCRIBING → STOPPED (autonomous, PUSH only)
            TransferState::Unsubscribing => Some(DataplaneCommand::SetStopped),
            // All other states: wait for manual command from control plane
            _ => None,
        };

        if let Some(command) = next_command {
            let next_input = DataplaneManagerInput {
                transfer_process_id: transfer_process_id.clone(),
                command,
            };
            // Recursive call via Box::pin to avoid infinite type size
            Box::pin(self.execute_command(&next_input)).await?;
        }

        Ok(())
    }

    // ─── helpers ───

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
}
