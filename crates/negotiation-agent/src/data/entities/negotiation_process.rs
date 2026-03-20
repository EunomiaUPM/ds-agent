/*
 *
 * * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
 * *
 * * This program is free software: you can redistribute it and/or modify
 * * it under the terms of the GNU General Public License as published by
 * * the Free Software Foundation, either version 3 of the License, or
 * * (at your option) any later version.
 * *
 * * This program is distributed in the hope that it will be useful,
 * * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * * GNU General Public License for more details.
 * *
 * * You should have received a copy of the GNU General Public License
 * * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use sea_orm::prelude::{DateTimeWithTimeZone, Json};
use sea_orm::{
    ActiveModelBehavior, ActiveValue, DeriveEntityModel, DerivePrimaryKey, DeriveRelation,
    EntityTrait, EnumIter, PrimaryKeyTrait, Related, RelationDef, RelationTrait,
};
use serde::{Deserialize, Serialize};
use urn::{Urn, UrnBuilder};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "negotiation_agent_process")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub state: String,
    pub state_attribute: Option<String>,
    pub associated_agent_peer: String,
    pub protocol: String,
    pub callback_address: Option<String>,
    pub role: String,
    pub properties: Json,
    pub error_details: Option<Json>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::negotiation_message::Entity")]
    Messages,
    #[sea_orm(has_many = "super::negotiation_process_identifier::Entity")]
    Identifiers,
    #[sea_orm(has_many = "super::offer::Entity")]
    Offers,
    #[sea_orm(has_many = "super::agreement::Entity")]
    Agreements,
}

impl Related<super::negotiation_message::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Messages.def()
    }
}

impl Related<super::negotiation_process_identifier::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Identifiers.def()
    }
}

impl Related<super::offer::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Offers.def()
    }
}

impl Related<super::agreement::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Agreements.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Clone)]
pub struct NewNegotiationProcessModel {
    pub(crate) id: Option<Urn>,
    pub(crate) state: String,
    pub(crate) state_attribute: Option<String>,
    pub(crate) associated_agent_peer: String,
    pub(crate) protocol: String,
    pub(crate) callback_address: Option<String>,
    pub(crate) role: String,
    pub(crate) properties: Json,
    pub(crate) error_details: Option<Json>,
}

impl Default for NewNegotiationProcessModel {
    fn default() -> Self {
        Self {
            id: None,
            state: "".to_string(),
            state_attribute: None,
            associated_agent_peer: "".to_string(),
            protocol: "DSP".to_string(),
            callback_address: None,
            role: "Provider".to_string(),
            properties: serde_json::json!({}),
            error_details: None,
        }
    }
}

impl From<NewNegotiationProcessModel> for ActiveModel {
    fn from(value: NewNegotiationProcessModel) -> Self {
        let new_urn =
            UrnBuilder::new("negotiation-process", uuid::Uuid::new_v4().to_string().as_str())
                .build()
                .expect("UrnBuilder failed");

        Self {
            id: ActiveValue::Set(value.id.unwrap_or(new_urn).to_string()),
            state: ActiveValue::Set(value.state),
            state_attribute: ActiveValue::Set(value.state_attribute),
            associated_agent_peer: ActiveValue::Set(value.associated_agent_peer),
            protocol: ActiveValue::Set(value.protocol),
            callback_address: ActiveValue::Set(value.callback_address),
            role: ActiveValue::Set(value.role),
            properties: ActiveValue::Set(value.properties),
            error_details: ActiveValue::Set(value.error_details),
            created_at: ActiveValue::Set(chrono::Utc::now().into()),
            updated_at: ActiveValue::Set(None),
        }
    }
}

impl From<&NewNegotiationProcessModel> for ActiveModel {
    fn from(value: &NewNegotiationProcessModel) -> Self {
        value.clone().into()
    }
}

pub struct EditNegotiationProcessModel {
    pub state: Option<String>,
    pub state_attribute: Option<String>,
    pub properties: Option<Json>,
    pub error_details: Option<Json>,
}

impl Default for EditNegotiationProcessModel {
    fn default() -> Self {
        Self { state: None, state_attribute: None, properties: None, error_details: None }
    }
}
