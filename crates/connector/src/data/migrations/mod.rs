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

use sea_orm_migration::prelude::*;

mod m20251111_000001_connector_template;
mod m20251111_000002_connector_instance;
mod m20251111_000003_connector_distribution_relation;

pub fn get_connector_migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
        Box::new(m20251111_000001_connector_template::Migration),
        Box::new(m20251111_000002_connector_instance::Migration),
        Box::new(m20251111_000003_connector_distribution_relation::Migration),
    ]
}
