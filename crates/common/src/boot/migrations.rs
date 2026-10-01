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

//! Migrator over the static migration list of a bootstrapped agent.

use std::marker::PhantomData;

use sea_orm::sea_query::{Alias, DynIden, IntoIden};
use sea_orm::DatabaseConnection;
use sea_orm_migration::{MigrationTrait, MigratorTrait};
use ymir::errors::{Errors, Outcome};

use crate::boot::BootstrapServiceTrait;

pub struct SetupMigrator<S>(PhantomData<S>);

impl<S: BootstrapServiceTrait> MigratorTrait for SetupMigrator<S> {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        S::migrations()
    }

    fn migration_table_name() -> DynIden {
        Alias::new(S::MIGRATION_TABLE).into_iden()
    }
}

impl<S: BootstrapServiceTrait> SetupMigrator<S> {
    /// Applies pending migrations; `reset` first rolls back every applied one (destroys data).
    pub async fn run(db: &DatabaseConnection, reset: bool) -> Outcome<()> {
        let result = if reset {
            tracing::warn!("Resetting database: rolling back every migration");
            Self::refresh(db).await
        } else {
            Self::up(db, None).await
        };
        result.map_err(|e| Errors::db("Error running migrations", Some(Box::new(e))))
    }
}
