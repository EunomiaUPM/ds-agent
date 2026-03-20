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

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "policy_templates")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub version: String,
    pub date: DateTimeWithTimeZone,
    pub author: String,
    pub title: Option<serde_json::Value>,
    pub description: Option<serde_json::Value>,
    pub content: serde_json::Value,
    pub parameters: serde_json::Value,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::odrl_offer::Entity")]
    OdrlOffers,
}

impl Related<super::odrl_offer::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OdrlOffers.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Clone)]
pub struct NewPolicyTemplateModel {
    pub id: Option<String>,
    pub version: Option<String>,
    pub date: Option<DateTimeWithTimeZone>,
    pub author: Option<String>,
    pub title: Option<serde_json::Value>,
    pub description: Option<serde_json::Value>,
    pub content: serde_json::Value,
    pub parameters: serde_json::Value,
}

impl From<NewPolicyTemplateModel> for ActiveModel {
    fn from(dto: NewPolicyTemplateModel) -> Self {
        let new_urn =
            UrnBuilder::new("policy-templates", uuid::Uuid::new_v4().to_string().as_str())
                .build()
                .expect("UrnBuilder failed");
        Self {
            id: ActiveValue::Set(dto.id.clone().unwrap_or(new_urn.clone().to_string()).to_string()),
            version: ActiveValue::Set(dto.version.unwrap_or("1.0".to_string())),
            date: ActiveValue::Set(dto.date.unwrap_or(chrono::Utc::now().into())),
            author: ActiveValue::Set(dto.author.unwrap_or("".to_string())),
            title: ActiveValue::Set(dto.title),
            description: ActiveValue::Set(dto.description),
            content: ActiveValue::Set(dto.content),
            parameters: ActiveValue::Set(dto.parameters),
        }
    }
}

impl From<&NewPolicyTemplateModel> for ActiveModel {
    fn from(dto: &NewPolicyTemplateModel) -> Self {
        dto.clone().into()
    }
}
