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

use chrono::Utc;
use compact_str::CompactString;
use sea_orm::ActiveValue::Set;
use sea_orm::entity::prelude::*;
use ymir::errors::Outcome;

use crate::data::sea_orm::orm::helpers::{deser_enum, deser_json, parse_urn, ser_enum, ser_json};
use crate::entities::commands::NewTransferProcessCommand;
use crate::entities::ids::{TenantId, TransferProcessId};
use crate::entities::protocol::{
    ProtocolId, ProtocolState, StateMetadata, TransferCorrelation, TransferRole,
};
use crate::entities::transfer_process::TransferProcess;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "transfer_processes")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub tenant_id: String,
    pub role: String,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub version: i32,
    pub protocol: String,
    pub protocol_state: String,
    pub state_metadata: Json,
    /// Denormalized from correlation.agreement_id for efficient filtering.
    pub agreement_id: Option<String>,
    /// Denormalized from correlation.peer_participant_id for efficient filtering.
    pub peer_participant_id: Option<String>,
    pub correlation: Json,
    pub properties: Json,
    pub error_details: Option<Json>,
    pub last_inbound_envelope: Option<Json>,
    pub last_outbound_envelope: Option<Json>,
}

impl Model {
    pub(crate) fn into_domain(self) -> Outcome<TransferProcess> {
        use crate::entities::message_envelope::MessageEnvelopeRef;

        let id = TransferProcessId::new(parse_urn(&self.id, "transfer_process.id")?);
        let tenant_id = TenantId::new(self.tenant_id);
        let role = deser_enum::<TransferRole>(&self.role)?;
        let created_at = self.created_at.with_timezone(&Utc);
        let updated_at = self.updated_at.with_timezone(&Utc);
        let version = self.version as u32;
        let protocol = deser_enum::<ProtocolId>(&self.protocol)?;
        let protocol_state = ProtocolState(CompactString::from(self.protocol_state));
        let state_metadata =
            deser_json::<StateMetadata>(self.state_metadata, "transfer_process.state_metadata")?;
        let correlation =
            deser_json::<TransferCorrelation>(self.correlation, "transfer_process.correlation")?;
        let last_inbound_envelope = self
            .last_inbound_envelope
            .map(|v| deser_json::<MessageEnvelopeRef>(v, "transfer_process.last_inbound_envelope"))
            .transpose()?;
        let last_outbound_envelope = self
            .last_outbound_envelope
            .map(|v| deser_json::<MessageEnvelopeRef>(v, "transfer_process.last_outbound_envelope"))
            .transpose()?;

        Ok(TransferProcess::rehydrate(
            id,
            tenant_id,
            role,
            created_at,
            updated_at,
            version,
            protocol,
            protocol_state,
            state_metadata,
            correlation,
            self.properties,
            self.error_details,
            last_inbound_envelope,
            last_outbound_envelope,
        ))
    }
}

impl ActiveModel {
    pub(crate) fn from_cmd(cmd: &NewTransferProcessCommand) -> Outcome<Self> {
        let id = cmd.id.clone().unwrap_or_else(TransferProcessId::generate);
        let tenant_id = cmd
            .tenant_id
            .clone()
            .ok_or_else(|| ymir::errors::Errors::crazy("tenant_id must be resolved before reaching the repo", None))?;
        let now = chrono::Utc::now();
        let correlation = TransferCorrelation {
            identifiers: std::collections::HashMap::new(),
            consumer_pid: None,
            provider_pid: None,
            agreement_id: Some(cmd.agreement_id.clone()),
            callback_address: cmd.callback_address.clone(),
            peer_participant_id: Some(cmd.peer_participant_id.clone()),
        };
        let process = TransferProcess::rehydrate(
            id,
            tenant_id,
            cmd.role,
            now,
            now,
            0,
            cmd.protocol.clone(),
            cmd.initial_state.clone(),
            cmd.initial_state_metadata.clone(),
            correlation,
            cmd.properties.clone().unwrap_or(serde_json::json!({})),
            None,
            None,
            None,
        );
        Ok(Self::from_domain(&process))
    }

    pub(crate) fn from_domain(process: &TransferProcess) -> Self {
        let correlation = process.correlation();
        Self {
            id: Set(process.id().to_string()),
            tenant_id: Set(process.tenant_id().as_str().to_string()),
            role: Set(ser_enum(&process.role())),
            created_at: Set(process.created_at().into()),
            updated_at: Set(process.updated_at().into()),
            version: Set(process.version() as i32),
            protocol: Set(ser_enum(process.protocol())),
            protocol_state: Set(process.state().0.to_string()),
            state_metadata: Set(ser_json(process.state_metadata())),
            agreement_id: Set(correlation.agreement_id.as_ref().map(|u| u.to_string())),
            peer_participant_id: Set(correlation
                .peer_participant_id
                .as_ref()
                .map(|p| p.to_string())),
            correlation: Set(ser_json(correlation)),
            properties: Set(process.properties().clone()),
            error_details: Set(process.error_details().cloned()),
            last_inbound_envelope: Set(process.last_inbound_envelope().map(ser_json)),
            last_outbound_envelope: Set(process.last_outbound_envelope().map(ser_json)),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
