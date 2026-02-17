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
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue;
use serde::{Deserialize, Serialize};
use urn::UrnBuilder;

#[derive(Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Text")]
pub enum LogLevel {
    #[sea_orm(string_value = "DEBUG")]
    Debug,
    #[sea_orm(string_value = "INFO")]
    Info,
    #[sea_orm(string_value = "WARN")]
    Warn,
    #[sea_orm(string_value = "ERROR")]
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "transfer_events")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub transfer_id: String,
    pub level: LogLevel,
    pub component: String,
    #[sea_orm(column_type = "Text")]
    pub message: String,
    pub data: Option<Json>,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::dataplane_transfers::Entity",
        from = "Column::TransferId",
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
pub struct NewTransferEvent {
    pub transfer_id: String,
    pub level: LogLevel,
    pub component: String,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

impl From<NewTransferEvent> for ActiveModel {
    fn from(value: NewTransferEvent) -> Self {
        let new_urn = UrnBuilder::new("transfer-event", uuid::Uuid::new_v4().to_string().as_str())
            .build()
            .expect("UrnBuilder failed");
        Self {
            id: ActiveValue::Set(new_urn.to_string()),
            transfer_id: ActiveValue::Set(value.transfer_id),
            level: ActiveValue::Set(value.level),
            component: ActiveValue::Set(value.component),
            message: ActiveValue::Set(value.message),
            data: ActiveValue::Set(value.data),
            created_at: ActiveValue::Set(chrono::Utc::now().into()),
            ..Default::default()
        }
    }
}
