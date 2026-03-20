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
#[sea_orm(table_name = "negotiation_agent_agreements")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub negotiation_agent_process_id: String,
    pub negotiation_agent_message_id: String,
    pub consumer_participant_id: String,
    pub provider_participant_id: String,
    pub agreement_content: Json,
    pub target: String,
    pub state: String,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::negotiation_process::Entity",
        from = "Column::NegotiationAgentProcessId",
        to = "super::negotiation_process::Column::Id",
        on_delete = "Cascade"
    )]
    Process,
    #[sea_orm(
        belongs_to = "super::negotiation_message::Entity",
        from = "Column::NegotiationAgentMessageId",
        to = "super::negotiation_message::Column::Id",
        on_delete = "Cascade"
    )]
    Message,
}

impl Related<super::negotiation_process::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Process.def()
    }
}

impl Related<super::negotiation_message::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Message.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Clone)]
pub struct NewAgreementModel {
    pub id: Option<Urn>,
    pub negotiation_agent_process_id: Urn,
    pub negotiation_agent_message_id: Urn,
    pub consumer_participant_id: String,
    pub provider_participant_id: String,
    pub agreement_content: Json,
    pub target: Urn,
}

impl From<NewAgreementModel> for ActiveModel {
    fn from(value: NewAgreementModel) -> Self {
        let new_urn =
            UrnBuilder::new("negotiation-agreement", uuid::Uuid::new_v4().to_string().as_str())
                .build()
                .expect("UrnBuilder failed");
        Self {
            id: ActiveValue::Set(value.id.unwrap_or(new_urn).to_string()),
            negotiation_agent_process_id: ActiveValue::Set(
                value.negotiation_agent_process_id.to_string(),
            ),
            negotiation_agent_message_id: ActiveValue::Set(
                value.negotiation_agent_message_id.to_string(),
            ),
            consumer_participant_id: ActiveValue::Set(value.consumer_participant_id),
            provider_participant_id: ActiveValue::Set(value.provider_participant_id),
            agreement_content: ActiveValue::Set(value.agreement_content),
            target: ActiveValue::Set(value.target.to_string()),
            state: ActiveValue::Set("REQUESTED".to_string()),
            created_at: ActiveValue::Set(chrono::Utc::now().into()),
            updated_at: ActiveValue::Set(None),
        }
    }
}

impl From<&NewAgreementModel> for ActiveModel {
    fn from(value: &NewAgreementModel) -> Self {
        value.clone().into()
    }
}

pub struct EditAgreementModel {
    pub state: Option<String>,
}

impl Default for EditAgreementModel {
    fn default() -> Self {
        Self { state: None }
    }
}
