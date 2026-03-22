/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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
use crate::{DataplaneInitCommandType, DataplaneManager};
use connector::{ConnectorInstanceDto, ConnectorInstanceTrait, InteractionConfig};
use std::sync::Arc;
use urn::{Urn, UrnBuilder};
use uuid::Uuid;
use ymir::errors::{Errors, Outcome};

impl DataplaneManager {
    // ─── Persistence helpers ───

    /// Logic for creating the Dataplane.
    /// Only input.command == DataplaneCommand::SetInit message is allowed
    /// Need to know InteractionMode and TransferRole
    /// Creates incremental config in ingress and egress
    pub(super) async fn handle_creation(
        &self,
        input: &DataplaneManagerInput,
        connector_urn: &Option<Urn>,
        connector_instance: &Option<ConnectorInstanceDto>,
    ) -> Outcome<DataplaneResponse> {
        if let DataplaneCommand::SetInit(role) = &input.command {
            // InteractionMode (PUSH or PULL)
            let interaction_mode = match role {
                // Provider has connector. PULL or PUSH mode are defined in ConnectorDto
                DataplaneInitCommandType::Provider { .. } => match connector_instance {
                    Some(c) => match &c.interaction {
                        InteractionConfig::Pull(_) => Ok(InteractionMode::Pull),
                        InteractionConfig::Push(_) => Ok(InteractionMode::Push),
                    },
                    None => Err(Errors::crazy("Missing Connector Instance", None)),
                },
                // But, Consumer has no connector. We need a hack here.
                // PUSH mode is signaled by a non-None data_address in DSP. (if any other protocol, this feature would be consistent
                // If DataAddress present in SetInit, is PUSH, otherwise is PULL
                DataplaneInitCommandType::Consumer { data_address } => match data_address {
                    Some(_) => Ok(InteractionMode::Push),
                    None => Ok(InteractionMode::Pull),
                },
            }?;
            // Id
            let new_id = UrnBuilder::new("dataplane-transfer", Uuid::new_v4().to_string().as_str())
                .build()?;
            // TransferRole (Consumer or Provider)
            let transfer_role = TransferRole::try_from(role)?;
            // Set up of configuration for egress and ingress
            let config = DataplaneConfigBuilder::from_connector(
                &transfer_role,
                connector_instance.as_ref(),
                &new_id.to_string(),
            );
            // persist
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
            // return
            Ok(DataplaneResponse::Ok)
        } else {
            Err(Errors::crazy(
                "Cannot execute command without an existing process (only SetInit creates)",
                None,
            ))
        }
    }

    pub(super) async fn update_state(
        &self,
        process_id: &Urn,
        new_state: TransferState,
    ) -> Outcome<DataplaneTransferDto> {
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

    pub(super) async fn update_state_and_config(
        &self,
        process_id: &Urn,
        new_state: TransferState,
        ingress: Option<serde_json::Value>,
        egress: Option<serde_json::Value>,
    ) -> Outcome<DataplaneTransferDto> {
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
    pub(super) async fn cmd_set_egress(
        &self,
        data_address: &DataplaneAddress,
        ctx: &CommandContext<'_>,
    ) -> Outcome<DataplaneResponse> {
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
}
