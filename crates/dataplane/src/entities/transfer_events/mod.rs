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

use crate::data::sea_orm::orm::transfer_event;
use crate::data::sea_orm::orm::transfer_event::{LogLevel, NewTransferEvent};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use urn::Urn;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferEventDto {
    #[serde(flatten)]
    pub inner: transfer_event::Model,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NewTransferEventDto {
    pub tenant_id: String,
    pub transfer_id: Urn,
    pub level: LogLevel,
    pub component: String,
    pub message: String,
    pub data: Option<Value>,
}

impl From<NewTransferEventDto> for NewTransferEvent {
    fn from(value: NewTransferEventDto) -> Self {
        Self {
            tenant_id: value.tenant_id,
            transfer_id: value.transfer_id.to_string(),
            level: value.level,
            component: value.component,
            message: value.message,
            data: value.data,
        }
    }
}
