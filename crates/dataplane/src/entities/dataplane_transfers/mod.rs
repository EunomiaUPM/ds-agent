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

use crate::data::sea_orm::orm::dataplane_transfer_logs;
use crate::data::sea_orm::orm::dataplane_transfers;
pub use crate::data::sea_orm::orm::dataplane_transfers::{
    InteractionMode, NewDataplaneTransfer, TransferRole, TransferState,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use urn::Urn;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DataplaneTransferDto {
    #[serde(flatten)]
    pub inner: dataplane_transfers::Model,
    pub fields: HashMap<String, String>,
    pub logs: Vec<dataplane_transfer_logs::Model>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NewDataplaneTransferDto {
    pub id: Option<Urn>,
    pub tenant_id: String,
    pub transfer_process_id: String,
    pub role: TransferRole,
    pub interaction_mode: InteractionMode,
    pub state: TransferState,
    pub connector_instance_id: Option<Urn>,
    pub ingress_config: Value,
    pub egress_config: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct EditDataplaneTransferDto {
    pub state: Option<TransferState>,
    pub connector_instance_id: Option<Urn>,
    pub ingress_config: Option<Value>,
    pub egress_config: Option<Value>,
    pub flow_control: Option<Value>,
    pub fields: Option<HashMap<String, String>>,
}

impl From<NewDataplaneTransferDto> for NewDataplaneTransfer {
    fn from(value: NewDataplaneTransferDto) -> Self {
        Self {
            id: value.id,
            tenant_id: value.tenant_id,
            transfer_process_id: value.transfer_process_id,
            role: value.role,
            interaction_mode: value.interaction_mode,
            state: value.state,
            connector_instance_id: value.connector_instance_id,
            ingress_config: value.ingress_config,
            egress_config: value.egress_config,
        }
    }
}
