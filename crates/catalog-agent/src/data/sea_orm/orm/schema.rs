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

//! `catalog_schemas` rows.

use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;
use ymir::types::oauth::RolePath;
use ymir::types::participants::Visibility;

use crate::entities::commands::NewSchemaVersionCommand;
use crate::entities::schemas::DataSchema;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "catalog_schemas")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub version: i32,
    pub user_id: String,
    pub user_role: RolePath,
    pub visibility: Visibility,
    pub created_at: DateTimeWithTimeZone,
    pub title: String,
    pub description: Option<String>,
    pub content: Json,
    pub sha256: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn into_domain(self) -> DataSchema {
        DataSchema {
            id: self.id,
            version: self.version,
            user_id: self.user_id,
            user_role: self.user_role,
            visibility: self.visibility,
            created_at: self.created_at,
            title: self.title,
            description: self.description,
            content: self.content,
            sha256: self.sha256,
        }
    }
}

impl ActiveModel {
    pub fn from_command(command: &NewSchemaVersionCommand) -> Self {
        Self {
            id: Set(command.id.clone()),
            version: Set(command.version),
            user_id: Set(command.owner.user_id.clone()),
            user_role: Set(command.owner.role.clone()),
            visibility: Set(command.owner.visibility.clone()),
            created_at: Set(chrono::Utc::now().into()),
            title: Set(command.title.clone()),
            description: Set(command.description.clone()),
            content: Set(command.content.clone()),
            sha256: Set(command.sha256.clone()),
        }
    }
}
