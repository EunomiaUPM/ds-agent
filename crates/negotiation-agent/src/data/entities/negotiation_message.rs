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
#[sea_orm(table_name = "negotiation_agent_messages")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub negotiation_agent_process_id: String,
    pub created_at: DateTimeWithTimeZone,
    pub direction: String,
    pub protocol: String,
    pub message_type: String,
    pub state_transition_from: String,
    pub state_transition_to: String,
    pub payload: Json,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::offer::Entity")]
    Offers,
    #[sea_orm(has_many = "super::agreement::Entity")]
    Agreements,
    #[sea_orm(
        belongs_to = "super::negotiation_process::Entity",
        from = "Column::NegotiationAgentProcessId",
        to = "super::negotiation_process::Column::Id",
        on_delete = "Cascade"
    )]
    Process,
}

impl Related<super::negotiation_process::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Process.def()
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
pub struct NewNegotiationMessageModel {
    pub id: Option<Urn>,
    pub negotiation_agent_process_id: Urn,
    pub direction: String,
    pub protocol: String,
    pub message_type: String,
    pub state_transition_from: String,
    pub state_transition_to: String,
    pub payload: Json,
}

impl From<NewNegotiationMessageModel> for ActiveModel {
    fn from(dto: NewNegotiationMessageModel) -> Self {
        let new_urn =
            UrnBuilder::new("negotiation-message", uuid::Uuid::new_v4().to_string().as_str())
                .build()
                .expect("UrnBuilder failed");
        Self {
            id: ActiveValue::Set(dto.id.unwrap_or(new_urn).to_string()),
            negotiation_agent_process_id: ActiveValue::Set(
                dto.negotiation_agent_process_id.to_string(),
            ),
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
