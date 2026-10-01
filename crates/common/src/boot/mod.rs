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

//! Process bootstrap shared by every agent binary: CLI, migrations, workers and boot sequence.

pub mod bootstrapper;
pub mod cli;
pub mod migrations;
pub mod seeders;
pub mod servers;
pub mod shutdown;
pub mod workers;

use std::sync::Arc;

use sea_orm::DatabaseConnection;
use sea_orm_migration::MigrationTrait;
use serde::Serialize;
use ymir::errors::Outcome;

use crate::auth::OauthTokenValidator;
use crate::boot::seeders::BootSeeder;
use crate::config::services::CommonConfig;
use crate::config::types::traits::{CommonConfigTrait, ConfigLoader};
use crate::module_loader::root_context::RootContext;
use crate::module_loader::service_composer::ServiceComposer;

/// What makes a binary an agent: its config, schema, module graph and boot tasks.
#[async_trait::async_trait]
pub trait BootstrapServiceTrait: Send + Sync + 'static {
    type Config: ConfigLoader + CommonConfigTrait + Serialize + Clone + Send + Sync + 'static;

    /// Table recording applied migrations.
    const MIGRATION_TABLE: &'static str = "seaql_migrations";

    /// Every migration the agent owns, in FK order; static because `MigratorTrait` is.
    fn migrations() -> Vec<Box<dyn MigrationTrait>>;

    /// The token validator every module guards its routes with, built once into the root.
    fn validator(common: &CommonConfig, db: DatabaseConnection) -> Arc<dyn OauthTokenValidator>;

    /// Wires every module the process hosts; workers and planes are read from the result.
    async fn compose(config: &Self::Config, root: &RootContext) -> Outcome<ServiceComposer>;

    /// Boot tasks, run in order around worker start-up (see `BootPhase`).
    async fn seeders(
        _config: &Self::Config,
        _root: &RootContext,
    ) -> Outcome<Vec<Box<dyn BootSeeder>>> {
        Ok(vec![])
    }
}
