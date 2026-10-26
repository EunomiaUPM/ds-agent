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

//! The catalog schema, written from scratch: resources, their links, templates and offers.

use sea_orm_migration::prelude::*;

pub mod m20261026_000001_catalog_resources;
pub mod m20261026_000002_catalog_resource_links;
pub mod m20261026_000003_catalog_policy_templates;
pub mod m20261026_000004_catalog_offers;

/// The catalog migrations, in creation order.
pub struct CatalogMigrations;

impl CatalogMigrations {
    pub fn all() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20261026_000001_catalog_resources::Migration),
            Box::new(m20261026_000002_catalog_resource_links::Migration),
            Box::new(m20261026_000003_catalog_policy_templates::Migration),
            Box::new(m20261026_000004_catalog_offers::Migration),
        ]
    }
}
