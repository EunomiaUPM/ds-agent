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

pub(crate) mod m20260519_000001_parameters;
pub(crate) mod m20260519_000002_secrets;

use sea_orm_migration::{MigrationTrait, MigratorTrait};

#[allow(dead_code)]
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
        get_keystore_migrations()
    }
}

pub fn get_keystore_migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
        Box::new(m20260519_000001_parameters::Migration),
        Box::new(m20260519_000002_secrets::Migration),
    ]
}
