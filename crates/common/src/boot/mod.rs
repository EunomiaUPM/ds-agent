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
//!
//! A binary describes itself by implementing [`BootstrapServiceTrait`]: its config type, its
//! migrations, the token validator and the module graph it hosts. [`AgentCli`] turns that into
//! the `start` and `setup` commands, and [`Bootstrapper`] runs the boot: it loads the config,
//! builds the [`RootContext`], composes the modules, runs the seeders and serves HTTP, gRPC and
//! every [`BackgroundWorker`] until SIGINT or SIGTERM.
//!
//! [`AgentCli`]: cli::AgentCli
//! [`Bootstrapper`]: bootstrapper::Bootstrapper
//! [`BackgroundWorker`]: workers::BackgroundWorker
//! [`RootContext`]: crate::module_loader::root_context::RootContext
//!
//! ## 1. An agent's `main`
//!
//! The whole `main` is one call. Telemetry, the banner and argument parsing happen inside.
//!
//! ```rust,ignore
//! use common::boot::cli::AgentCli;
//! use transfer_agent::setup::TransferBoot;
//! use transfer_agent::{SERVICE_BIG_NAME, SERVICE_NAME};
//!
//! #[tokio::main]
//! async fn main() -> Outcome<()> {
//!     AgentCli::<TransferBoot>::run(SERVICE_NAME, SERVICE_BIG_NAME).await
//! }
//! ```
//!
//! ```text
//! transfer-agent setup -e config.yaml            write vault secrets, apply migrations
//! transfer-agent setup -e config.yaml --reset    roll everything back first (destroys data)
//! transfer-agent start -e config.yaml            serve until SIGINT/SIGTERM
//! ```
//!
//! ## 2. Describing the agent
//!
//! Implement [`BootstrapServiceTrait`] on a unit struct in the binary's `setup/boot.rs`.
//! `migrations` is static and lists every module's migrations in foreign-key order;
//! `compose` builds the modules on the shared root context.
//!
//! ```rust,ignore
//! use common::boot::BootstrapServiceTrait;
//! use common::module_loader::service_composer::ServiceComposer;
//!
//! pub struct TransferBoot;
//!
//! #[async_trait::async_trait]
//! impl BootstrapServiceTrait for TransferBoot {
//!     type Config = TransferConfig;
//!
//!     fn migrations() -> Vec<Box<dyn MigrationTrait>> {
//!         [OAuthModule::migrations(), TransferAgentModule::migrations()]
//!             .into_iter()
//!             .flatten()
//!             .collect()
//!     }
//!
//!     fn validator(
//!         common: &CommonConfig,
//!         db: DatabaseConnection,
//!     ) -> Arc<dyn OauthTokenValidator> {
//!         OAuthModule::validator(common, db)
//!     }
//!
//!     async fn compose(config: &TransferConfig, root: &RootContext) -> Outcome<ServiceComposer> {
//!         let ports = TransferPorts::remote(config, root).await?;
//!         Ok(ServiceComposer::new()
//!             .register(OAuthModule::compose(config.common(), root, None))
//!             .register(TransferAgentModule::compose(config, root, None, &ports))
//!             .with_auth_ports(ports.auth.clone()))
//!     }
//! }
//! ```
//!
//! ## 3. Seeders
//!
//! A [`BootSeeder`] is a one-off task run at every boot: seeding users, provisioning a tenant,
//! flushing a cache. `BeforeServe` seeders run before anything listens, so they must write
//! straight to the database. `AfterServe` seeders, the default, run once the servers are up
//! and may call the agent's own API.
//!
//! Seeders come from two places. The boot's `seeders()` returns infrastructure seeders, which
//! run before the module graph exists. A module's `seeders()` returns seeders built on its own
//! services, which run right after composing.
//!
//! ```rust,ignore
//! use common::boot::seeders::{BootPhase, BootSeeder};
//!
//! #[async_trait::async_trait]
//! impl BootSeeder for AdminSeeder {
//!     fn name(&self) -> &'static str {
//!         "oauth-admin"
//!     }
//!
//!     // Other seeders call the API as this admin, so it must exist first.
//!     fn phase(&self) -> BootPhase {
//!         BootPhase::BeforeServe
//!     }
//!
//!     async fn seed(&self) -> Outcome<()> {
//!         seed_admin_user(self.db.clone(), &self.admin.tenant_id, /* ... */).await
//!     }
//! }
//! ```
//!
//! [`RedisCacheFlush`] is a ready-made `BeforeServe` seeder that empties Redis on start.
//!
//! [`BootSeeder`]: seeders::BootSeeder
//! [`RedisCacheFlush`]: seeders::RedisCacheFlush
//!
//! ## 4. Background workers
//!
//! Anything that runs for the life of the process (pollers, bus listeners, the servers
//! themselves) is a [`BackgroundWorker`] returned from its module's `workers()`. The boot runs
//! them all in one [`WorkerSet`] with a shared cancellation token. A worker must return when
//! the token is cancelled; returning earlier counts as a failure and stops the process.
//!
//! ```rust,ignore
//! use common::boot::workers::BackgroundWorker;
//! use tokio_util::sync::CancellationToken;
//!
//! #[async_trait::async_trait]
//! impl BackgroundWorker for TenantProvisioningListener {
//!     fn name(&self) -> &'static str {
//!         "tenant-provisioning"
//!     }
//!
//!     async fn run(self: Box<Self>, token: CancellationToken) -> Outcome<()> {
//!         let mut receiver = self.bus.subscribe();
//!         loop {
//!             let envelope = tokio::select! {
//!                 _ = token.cancelled() => return Ok(()),
//!                 received = receiver.recv() => received,
//!             };
//!             // handle the envelope ...
//!         }
//!     }
//! }
//! ```
//!
//! On shutdown, workers get 15 seconds to drain before they are aborted.
//!
//! [`WorkerSet`]: workers::WorkerSet
//!
//! ## 5. What the boot serves
//!
//! Besides the composed modules, the HTTP plane always carries the `.well-known` routes, the
//! health check, a 404 fallback in the common error format and the tracing layers. TLS is read
//! from the vault in production unless a TLS proxy sits in front. The gRPC server only starts
//! when a gRPC host is configured and some module registers a descriptor set, and it adds
//! reflection.

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
    /// Config type the binary loads from its YAML file.
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
