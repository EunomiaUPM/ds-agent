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

//! Diagnostic events.

use crate::data::sea_orm::orm::transfer_event;
use crate::data::sea_orm::orm::transfer_event::{LogLevel, NewTransferEvent};
use common::oauth::Owner;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use urn::Urn;

/// Diagnostic event as returned by the API.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferEventDto {
    #[serde(flatten)]
    pub inner: transfer_event::Model,
}

/// New diagnostic event.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NewTransferEventDto {
    /// Owner of the event; only honoured for the root (the engine, for the transfer's owner).
    #[serde(default)]
    pub owner: Option<Owner>,
    pub transfer_id: Urn,
    pub level: LogLevel,
    pub component: String,
    pub message: String,
    pub data: Option<Value>,
}

impl NewTransferEventDto {
    /// The row to store, owned by `owner`.
    pub fn into_model(self, owner: Owner) -> NewTransferEvent {
        let value = self;
        NewTransferEvent {
            owner,
            transfer_id: value.transfer_id.to_string(),
            level: value.level,
            component: value.component,
            message: value.message,
            data: value.data,
        }
    }
}
