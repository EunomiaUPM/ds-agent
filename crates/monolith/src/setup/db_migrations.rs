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

use sea_orm::sea_query::{Alias, DynIden, IntoIden};
use sea_orm::DatabaseConnection;
use sea_orm_migration::{MigrationTrait, MigratorTrait};
use ymir::errors::{Errors, Outcome};

use crate::setup::composition::MonolithModule;

pub struct CoreProviderMigration;

impl MigratorTrait for CoreProviderMigration {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        MonolithModule::migrations()
    }

    fn migration_table_name() -> DynIden {
        Alias::new("seaql_ds_agent_migrations").into_iden()
    }
}

impl CoreProviderMigration {
    pub async fn run(db_connection: &DatabaseConnection) -> Outcome<()> {
        Self::up(db_connection, None)
            .await
            .map_err(|e| Errors::db("Error migrating data", Some(Box::new(e))))
    }
}
