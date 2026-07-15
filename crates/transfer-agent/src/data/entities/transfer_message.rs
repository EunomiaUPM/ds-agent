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

use common::utils::get_urn;
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue;
use serde::{Deserialize, Serialize};
use urn::Urn;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "transfer_agent_messages")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub(crate) id: String,
    pub(crate) transfer_agent_process_id: String,
    pub(crate) created_at: DateTimeWithTimeZone,
    pub(crate) direction: String,
    pub(crate) protocol: String,
    pub(crate) message_type: String,
    pub(crate) state_transition_from: String,
    pub(crate) state_transition_to: String,
    pub(crate) payload: Option<Json>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::transfer_process::Entity",
        from = "Column::TransferAgentProcessId",
        to = "super::transfer_process::Column::Id",
        on_delete = "Cascade"
    )]
    Process,
}

impl Related<super::transfer_process::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Process.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Clone)]
pub struct NewTransferMessageModel {
    pub id: Option<Urn>,
    pub transfer_agent_process_id: Urn,
    pub direction: String,
    pub protocol: String,
    pub message_type: String,
    pub state_transition_from: String,
    pub state_transition_to: String,
    pub payload: Option<Json>,
}

impl From<NewTransferMessageModel> for ActiveModel {
    fn from(dto: NewTransferMessageModel) -> Self {
        Self {
            id: ActiveValue::Set(dto.id.unwrap_or(get_urn(None)).to_string()),
            transfer_agent_process_id: ActiveValue::Set(dto.transfer_agent_process_id.to_string()),
            direction: ActiveValue::Set(dto.direction),
            protocol: ActiveValue::Set(dto.protocol),
            message_type: ActiveValue::Set(dto.message_type),
            state_transition_from: ActiveValue::Set(dto.state_transition_from),
            state_transition_to: ActiveValue::Set(dto.state_transition_to),
            payload: ActiveValue::Set(dto.payload),
            created_at: ActiveValue::Set(chrono::Utc::now().into()),
        }
    }
}
