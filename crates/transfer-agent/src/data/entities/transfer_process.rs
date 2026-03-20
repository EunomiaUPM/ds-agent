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
use sea_orm::sqlx::types::uuid;
use sea_orm::ActiveValue;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use urn::{Urn, UrnBuilder};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "transfer_agent_process")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub state: String,
    pub state_attribute: Option<String>,
    pub associated_agent_peer: String,
    pub protocol: String,
    pub transfer_direction: String,
    pub agreement_id: String,
    pub callback_address: Option<String>,
    pub role: String,
    pub properties: Json,
    pub error_details: Option<Json>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::transfer_message::Entity")]
    Messages,
    #[sea_orm(has_many = "super::transfer_process_identifier::Entity")]
    Identifiers,
}

impl Related<super::transfer_message::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Messages.def()
    }
}

impl Related<super::transfer_process_identifier::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Identifiers.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Clone)]
pub struct NewTransferProcessModel {
    pub(crate) id: Option<Urn>,
    pub(crate) state: String,
    pub(crate) state_attribute: Option<String>,
    pub(crate) associated_agent_peer: String,
    pub(crate) protocol: String,
    pub(crate) transfer_direction: String,
    pub(crate) agreement_id: Urn,
    pub(crate) callback_address: Option<String>,
    pub(crate) role: String,
    pub(crate) properties: Json,
    pub(crate) error_details: Option<Json>,
}

impl Default for NewTransferProcessModel {
    fn default() -> Self {
        Self {
            id: None,
            state: "".to_string(),
            state_attribute: None,
            associated_agent_peer: "".to_owned(),
            protocol: "dsp".to_owned(), // TODO display enum
            transfer_direction: "push".to_owned(),
            agreement_id: Urn::from_str(format!("urn:uuid:{}", uuid::Uuid::default()).as_str())
                .unwrap(),
            callback_address: None,
            role: "".to_string(),
            properties: serde_json::json!({}),
            error_details: None,
        }
    }
}

impl From<NewTransferProcessModel> for ActiveModel {
    fn from(dto: NewTransferProcessModel) -> Self {
        let new_urn =
            UrnBuilder::new("transfer-process", uuid::Uuid::new_v4().to_string().as_str())
                .build()
                .expect("UrnBuilder failed");
        Self {
            id: ActiveValue::Set(dto.id.unwrap_or(new_urn).to_string()),
            state: ActiveValue::Set(dto.state),
            state_attribute: ActiveValue::Set(dto.state_attribute),
            associated_agent_peer: ActiveValue::Set(dto.associated_agent_peer),
            protocol: ActiveValue::Set(dto.protocol),
            transfer_direction: ActiveValue::Set(dto.transfer_direction),
            agreement_id: ActiveValue::Set(dto.agreement_id.to_string()),
            callback_address: ActiveValue::Set(dto.callback_address),
            role: ActiveValue::Set(dto.role),
            properties: ActiveValue::Set(dto.properties),
            error_details: ActiveValue::Set(dto.error_details),
            created_at: ActiveValue::Set(chrono::Utc::now().into()),
            updated_at: ActiveValue::Set(None),
        }
    }
}

pub struct EditTransferProcessModel {
    pub state: Option<String>,
    pub state_attribute: Option<String>,
    pub properties: Option<Json>,
    pub error_details: Option<Json>,
}

impl Default for EditTransferProcessModel {
    fn default() -> Self {
        Self { state: None, state_attribute: None, properties: None, error_details: None }
    }
}
