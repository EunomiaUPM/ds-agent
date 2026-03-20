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

use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue;
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;
use urn::{Urn, UrnBuilder};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "connector_instances")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub template_name: String,
    pub template_version: String,
    pub distribution_id: String,
    pub created_at: DateTimeWithTimeZone,
    pub metadata: Json,
    pub configuration_parameters: Json,
    pub authentication: Json,
    pub interaction: Json,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::connector_templates::Entity",
        from = "(Column::TemplateName, Column::TemplateVersion)",
        to = "(super::connector_templates::Column::Name, super::connector_templates::Column::Version)"
    )]
    ConnectorTemplate,
}

impl Related<super::connector_templates::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ConnectorTemplate.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Clone)]
pub struct NewConnectorInstanceModel {
    pub id: Option<Urn>,
    pub template_name: String,
    pub template_version: String,
    pub distribution_id: String,
    pub metadata: Json,
    pub configuration_parameters: Json,
    pub authentication: Json,
    pub interaction: Json,
}

impl From<NewConnectorInstanceModel> for ActiveModel {
    fn from(dto: NewConnectorInstanceModel) -> Self {
        let new_urn =
            UrnBuilder::new("connector-instance", uuid::Uuid::new_v4().to_string().as_str())
                .build()
                .expect("UrnBuilder failed");
        let now = chrono::Utc::now().into();

        Self {
            id: ActiveValue::Set(dto.id.clone().unwrap_or(new_urn).to_string()),
            template_name: ActiveValue::Set(dto.template_name),
            template_version: ActiveValue::Set(dto.template_version),
            distribution_id: ActiveValue::Set(dto.distribution_id),
            created_at: ActiveValue::Set(now),
            metadata: ActiveValue::Set(dto.metadata),
            configuration_parameters: ActiveValue::Set(dto.configuration_parameters),
            authentication: ActiveValue::Set(dto.authentication),
            interaction: ActiveValue::Set(dto.interaction),
        }
    }
}
