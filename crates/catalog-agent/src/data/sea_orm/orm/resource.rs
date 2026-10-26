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

//! `catalog_resources` rows.

use std::str::FromStr;

use chrono::Utc;
use common::oauth::Owner;
use common::utils::parse_urn;
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;
use ymir::errors::{Outcome, RepoIntoErrors};
use ymir::types::oauth::RolePath;
use ymir::types::participants::Visibility;

use crate::data::repo::resource::ResourceRepoErrors;
use crate::entities::graph::ResourceGraph;
use crate::entities::resource::{CatalogResource, CatalogResourceParts};
use crate::entities::resource_kind::ResourceKind;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "catalog_resources")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub kind: String,
    pub user_id: String,
    pub user_role: RolePath,
    pub visibility: Visibility,
    pub catalog_id: Option<String>,
    pub dataset_id: Option<String>,
    pub is_main: bool,
    pub title: Option<String>,
    pub issued: DateTimeWithTimeZone,
    pub modified: Option<DateTimeWithTimeZone>,
    pub version: i64,
    pub description: Json,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn into_domain(self) -> Outcome<CatalogResource> {
        let parse_parent = |id: Option<String>| id.as_deref().map(parse_urn).transpose();
        Ok(CatalogResource::rehydrate(CatalogResourceParts {
            id: parse_urn(&self.id)?,
            kind: ResourceKind::from_str(&self.kind)?,
            owner: Owner::new(self.user_id, self.user_role, self.visibility),
            catalog_id: parse_parent(self.catalog_id)?,
            dataset_id: parse_parent(self.dataset_id)?,
            main: self.is_main,
            description: ResourceGraph::from_node(self.description)
                .map_err(|e| ResourceRepoErrors::Decoding(e.to_string()).into_errors())?,
            issued: self.issued.with_timezone(&Utc),
            modified: self.modified.map(|at| at.with_timezone(&Utc)),
            version: self.version,
        }))
    }
}

impl ActiveModel {
    pub fn from_domain(resource: &CatalogResource) -> Self {
        let owner = resource.owner();
        Self {
            id: Set(resource.id().to_string()),
            kind: Set(resource.kind().as_str().to_string()),
            user_id: Set(owner.user_id.clone()),
            user_role: Set(owner.role.clone()),
            visibility: Set(owner.visibility.clone()),
            catalog_id: Set(resource.catalog_id().map(ToString::to_string)),
            dataset_id: Set(resource.dataset_id().map(ToString::to_string)),
            is_main: Set(resource.is_main()),
            title: Set(resource.title()),
            issued: Set(resource.issued().into()),
            modified: Set(resource.modified().map(Into::into)),
            version: Set(resource.version()),
            description: Set(resource.description().to_value()),
        }
    }
}
