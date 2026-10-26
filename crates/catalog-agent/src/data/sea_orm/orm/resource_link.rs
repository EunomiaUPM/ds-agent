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

//! `catalog_resource_links` rows: one link from a resource to another.

use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;

use crate::entities::resource::CatalogResource;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "catalog_resource_links")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub subject: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub predicate: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub object: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl ActiveModel {
    /// The rows of every local link the resource's description makes.
    pub fn from_domain(resource: &CatalogResource) -> Vec<Self> {
        let subject = resource.id().to_string();
        resource
            .description()
            .local_links()
            .into_iter()
            .map(|(predicate, object)| Self {
                subject: Set(subject.clone()),
                predicate: Set(predicate.to_string()),
                object: Set(object),
            })
            .collect()
    }
}
