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

//! `catalog_policy_templates` rows.

use common::oauth::Owner;
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;
use ymir::errors::{Outcome, RepoIntoErrors};
use ymir::types::oauth::RolePath;
use ymir::types::participants::Visibility;

use crate::data::repo::policy_template::PolicyTemplateRepoErrors;
use crate::entities::commands::NewPolicyTemplateCommand;
use crate::entities::policy_templates::PolicyTemplateDto;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "catalog_policy_templates")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub version: String,
    pub kind: String,
    pub control_points: Json,
    pub user_id: String,
    pub user_role: RolePath,
    pub visibility: Visibility,
    pub date: DateTimeWithTimeZone,
    pub author: String,
    pub title: Option<Json>,
    pub description: Option<Json>,
    pub content: Json,
    pub parameters: Json,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn into_domain(self) -> Outcome<PolicyTemplateDto> {
        let decode = |e: serde_json::Error| PolicyTemplateRepoErrors::Decoding(e.to_string()).into_errors();
        Ok(PolicyTemplateDto {
            id: self.id,
            user_id: self.user_id,
            user_role: self.user_role,
            visibility: self.visibility,
            version: self.version,
            kind: self.kind.parse().map_err(|e: String| PolicyTemplateRepoErrors::Decoding(e).into_errors())?,
            control_points: serde_json::from_value(self.control_points).map_err(decode)?,
            date: self.date,
            title: self.title.map(serde_json::from_value).transpose().map_err(decode)?,
            description: self.description.map(serde_json::from_value).transpose().map_err(decode)?,
            author: self.author,
            content: self.content,
            parameters: serde_json::from_value(self.parameters).map_err(decode)?,
        })
    }
}

impl ActiveModel {
    /// The row a command stores: a fresh URN, version 1.0 and now when they are not given.
    pub fn from_command(command: &NewPolicyTemplateCommand) -> Self {
        let owner: &Owner = &command.owner;
        let id = command.id.clone().unwrap_or_else(|| {
            format!("urn:policy-templates:{}", uuid::Uuid::new_v4())
        });
        Self {
            id: Set(id),
            version: Set(command.version.clone().unwrap_or_else(|| "1.0".to_string())),
            kind: Set(command.kind.to_string()),
            control_points: Set(serde_json::json!(command.control_points)),
            user_id: Set(owner.user_id.clone()),
            user_role: Set(owner.role.clone()),
            visibility: Set(owner.visibility.clone()),
            date: Set(command.date.unwrap_or_else(|| chrono::Utc::now().into())),
            author: Set(command.author.clone().unwrap_or_default()),
            title: Set(command.title.clone()),
            description: Set(command.description.clone()),
            content: Set(command.content.clone()),
            parameters: Set(command.parameters.clone()),
        }
    }
}
