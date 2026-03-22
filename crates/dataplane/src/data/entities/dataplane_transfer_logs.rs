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

use super::dataplane_transfers::TransferState;
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue;
use serde::{Deserialize, Serialize};
use urn::UrnBuilder;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "dataplane_transfer_logs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub dataplane_process_id: String,
    pub previous_state: Option<TransferState>,
    pub new_state: TransferState,
    pub trigger: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub reason: Option<String>,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::dataplane_transfers::Entity",
        from = "Column::DataplaneProcessId",
        to = "super::dataplane_transfers::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    DataplaneTransfer,
}

impl Related<super::dataplane_transfers::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DataplaneTransfer.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Clone)]
pub struct NewTransferLog {
    pub dataplane_process_id: String,
    pub previous_state: Option<TransferState>,
    pub new_state: TransferState,
    pub trigger: String,
    pub reason: Option<String>,
}

impl From<NewTransferLog> for ActiveModel {
    fn from(value: NewTransferLog) -> Self {
        let new_urn = UrnBuilder::new(
            "dataplane-process-log",
            uuid::Uuid::new_v4().to_string().as_str(),
        )
        .build()
        .expect("UrnBuilder failed");
        Self {
            id: ActiveValue::Set(new_urn.to_string()),
            dataplane_process_id: ActiveValue::Set(value.dataplane_process_id),
            previous_state: ActiveValue::Set(value.previous_state),
            new_state: ActiveValue::Set(value.new_state),
            trigger: ActiveValue::Set(value.trigger),
            reason: ActiveValue::Set(value.reason),
            created_at: ActiveValue::Set(chrono::Utc::now().into()),
        }
    }
}
