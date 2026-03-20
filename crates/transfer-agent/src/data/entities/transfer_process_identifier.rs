/*
 *
 *  * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
 *  *
 *  * This program is free software: you can redistribute it and/or modify
 *  * it under the terms of the GNU General Public License as published by
 *  * the Free Software Foundation, either version 3 of the License, or
 *  * (at your option) any later version.
 *  *
 *  * This program is distributed in the hope that it will be useful,
 *  * but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  * GNU General Public License for more details.
 *  *
 *  * You should have received a copy of the GNU General Public License
 *  * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use common::utils::get_urn;
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue;
use serde::{Deserialize, Serialize};
use urn::Urn;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "transfer_agent_identifiers")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub transfer_agent_process_id: String,
    pub id_key: String,
    pub id_value: Option<String>,
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
pub struct NewTransferIdentifierModel {
    pub(crate) id: Option<Urn>,
    pub(crate) transfer_agent_process_id: Urn,
    pub(crate) id_key: String,
    pub(crate) id_value: Option<String>,
}

impl From<NewTransferIdentifierModel> for ActiveModel {
    fn from(dto: NewTransferIdentifierModel) -> Self {
        Self {
            id: ActiveValue::Set(dto.id.unwrap_or(get_urn(None)).to_string()),
            transfer_agent_process_id: ActiveValue::Set(dto.transfer_agent_process_id.to_string()),
            id_key: ActiveValue::Set(dto.id_key),
            id_value: ActiveValue::Set(dto.id_value),
        }
    }
}

pub struct EditTransferIdentifierModel {
    pub id_key: Option<String>,
    pub id_value: Option<String>,
}

impl Default for EditTransferIdentifierModel {
    fn default() -> Self {
        Self {
            id_key: None,
            id_value: None,
        }
    }
}
