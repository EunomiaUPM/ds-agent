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

//! Start-up sequence of an agent process, from config loading to graceful shutdown.

use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;

use tokio_util::sync::CancellationToken;
use ymir::config::traits::{ConnectionConfigTrait, HostsConfigTrait};
use ymir::config::types::HostType;
use ymir::errors::{Errors, Outcome};
use ymir::http::HealthRouter;
use ymir::services::vault::global::VaultService;
use ymir::services::vault::VaultTrait;

use crate::boot::migrations::SetupMigrator;
use crate::boot::seeders::{BootPhase, BootSeeder};
use crate::boot::servers::{GrpcServer, HttpServer};
use crate::boot::shutdown::ShutdownSignal;
use crate::boot::workers::{WorkerExit, WorkerSet};
use crate::boot::BootstrapServiceTrait;
use crate::config::types::min_known_config::MinKnownConfig;
use crate::config::types::traits::{CommonConfigTrait, ConfigLoader};
use crate::http_global_404::global_handler_404;
use crate::http_tracing::trace_layer;
use crate::module_loader::root_context::RootContext;
use crate::module_loader::service_composer::ServiceComposer;
use crate::utils::show_table;
use crate::vault_utils::vault;
use crate::well_known::WellKnownRoot;

/// Time workers get to drain once shutdown starts.
const SHUTDOWN_GRACE: Duration = Duration::from_secs(15);

pub struct Bootstrapper<S>(PhantomData<S>);

impl<S: BootstrapServiceTrait> Bootstrapper<S> {
    /// Serves the agent until SIGINT/SIGTERM, or fails when a worker stops on its own.
    pub async fn start(env_file: &str) -> Outcome<()> {
        let (config, vault) = Self::load(env_file)?;
        let root = RootContext::connect(config.common(), vault, S::validator).await?;
        let (before, after): (Vec<_>, Vec<_>) = S::seeders(&config, &root)
            .await?
            .into_iter()
            .partition(|s| s.phase() == BootPhase::BeforeServe);
        Self::seed(&before).await?;

        tracing::info!("Composing service graph...");
        let composer = S::compose(&config, &root).await?;
        let mut workers = WorkerSet::new(CancellationToken::new());
        workers.spawn(Box::new(
            Self::http_server(&config, &root.vault, &composer).await?,
        ));
        if let Some(grpc) = Self::grpc_server(&config, &composer).await? {
            workers.spawn(Box::new(grpc));
        }
        workers.spawn_all(composer.workers());

        if let Err(e) = Self::seed(&after).await {
            workers.shutdown(SHUTDOWN_GRACE).await;
            return Err(e);
        }

        tracing::info!("System is RUNNING. Waiting for termination signal...");
        let outcome = tokio::select! {
            _ = ShutdownSignal::received() => {
                tracing::info!("Termination signal received");
                Ok(())
            }
            exit = workers.wait_any() => Self::unexpected(exit),
        };
        tracing::info!("Initiating graceful shutdown...");
        workers.shutdown(SHUTDOWN_GRACE).await;
        tracing::info!("Terminating process.");
        outcome
    }

    /// Writes vault secrets and applies the agent's migrations.
    pub async fn setup(env_file: &str, reset: bool) -> Outcome<()> {
        let (config, vault) = Self::load(env_file)?;
        vault.write_all_secrets(None).await?;
        if S::migrations().is_empty() {
            tracing::info!("Agent owns no migrations, nothing to migrate");
            return Ok(());
        }
        let db = vault.get_db_connection(config.common()).await?;
        SetupMigrator::<S>::run(&db, reset).await
    }

    fn load(env_file: &str) -> Outcome<(S::Config, Arc<VaultService>)> {
        let config = S::Config::load(env_file)?;
        show_table(&config)?;
        let vault = Arc::new(vault(config.common())?);
        Ok((config, vault))
    }

    async fn seed(seeders: &[Box<dyn BootSeeder>]) -> Outcome<()> {
        for seeder in seeders {
            tracing::info!(seeder = seeder.name(), "Running boot seeder");
            seeder.seed().await?;
        }
        Ok(())
    }

    /// Composed HTTP plane plus the process-wide well-known, health and 404 surfaces.
    async fn http_server(
        config: &S::Config,
        vault: &VaultService,
        composer: &ServiceComposer,
    ) -> Outcome<HttpServer> {
        let common = config.common();
        let router = composer
            .http_router()
            .merge(WellKnownRoot::get_well_known_router(
                &MinKnownConfig::from(common),
            )?)
            .merge(HealthRouter::new().router())
            .fallback(global_handler_404)
            .layer(trace_layer());
        let tls = match common.is_prod() && !common.has_tls_proxy() {
            true => Some(HttpServer::tls_from_vault(vault).await?),
            false => None,
        };
        HttpServer::bind(common.get_internal_port(HostType::Http), router, tls).await
    }

    /// `None` when no gRPC host is configured or no module has a gRPC plane.
    async fn grpc_server(
        config: &S::Config,
        composer: &ServiceComposer,
    ) -> Outcome<Option<GrpcServer>> {
        let common = config.common();
        let descriptors = composer.grpc_descriptors();
        if common.grpc().is_none() || descriptors.is_empty() {
            tracing::info!("No gRPC host or gRPC services, skipping gRPC server");
            return Ok(None);
        }
        let port = common.get_internal_port(HostType::Grpc);
        Ok(Some(
            GrpcServer::bind(port, composer.grpc_routes(), descriptors).await?,
        ))
    }

    fn unexpected(exit: WorkerExit) -> Outcome<()> {
        let reason = match exit.result {
            Ok(()) => "stopped".to_string(),
            Err(e) => format!("failed: {e}"),
        };
        tracing::error!(worker = exit.name, "Worker {reason} unexpectedly");
        Err(Errors::crazy(
            format!("Worker '{}' {reason} unexpectedly", exit.name),
            None,
        ))
    }
}
