/*
 *
 *  * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
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
use crate::setup::grpc_worker::TransferGrpcWorker;
use crate::setup::http_worker::TransferHttpWorker;
use common::boot::BootstrapServiceTrait;
use common::config::services::TransferConfig;
use common::config::types::traits::ConfigLoader;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::broadcast::Sender;
use tokio_util::sync::CancellationToken;
use ymir::config::traits::ConnectionConfigTrait;
use ymir::errors::Outcome;
use ymir::services::vault::VaultTrait;
use ymir::services::vault::fake_vault::FakeVaultService;
use ymir::services::vault::global::VaultService;
use ymir::services::vault::vault_rs::RealVaultService;

pub struct TransferBoot;

#[async_trait::async_trait]
impl BootstrapServiceTrait for TransferBoot {
    type Config = TransferConfig;

    async fn load_config(env_file: String) -> Outcome<Self::Config> {
        let config = Self::Config::load(&env_file)?;
        let table = json_to_table::json_to_table(&serde_json::to_value(&config)?)
            .collapse()
            .to_string();
        tracing::info!("Current Transfer Agent Ref Config:\n{}", table);
        Ok(config)
    }

    fn enable_participant() -> bool {
        false
    }
    fn enable_catalog() -> bool {
        false
    }
    fn enable_dataservice() -> bool {
        false
    }
    fn enable_policy_templates() -> bool {
        false
    }

    fn enable_user_seed() -> bool {
        true
    }

    async fn seed_users(config: &Self::Config) -> Outcome<()> {
        use common::config::types::traits::CommonConfigTrait;
        let vault = if config.is_vault_real() {
            VaultService::Real(RealVaultService::new()?)
        } else {
            VaultService::Fake(FakeVaultService::new()?)
        };
        let db = vault.get_db_connection(config.common()).await?;
        oauth::services::seed_admin_user(db, "admin", "admin@admin.local", "admin").await
    }

    async fn start_services_background(
        config: &Self::Config,
        vault: Arc<VaultService>,
    ) -> Outcome<Sender<()>> {
        let (shutdown_tx, mut shutdown_rx) = broadcast::channel(1);
        let cancel_token = CancellationToken::new();

        tracing::info!("Spawning HTTP subsystem...");
        let http_handle = TransferHttpWorker::spawn(config, vault.clone(), &cancel_token).await?;

        tracing::info!("Spawning gRPC subsystem...");
        let grpc_handle = TransferGrpcWorker::spawn(config, vault.clone(), &cancel_token).await?;

        let token_clone = cancel_token.clone();
        tokio::spawn(async move {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    tracing::info!("Shutdown command received from Main Pipeline.");
                }
                _ = http_handle => {
                    tracing::error!("HTTP subsystem failed or stopped unexpectedly!");
                }
                _ = grpc_handle => {
                    tracing::error!("gRPC subsystem failed or stopped unexpectedly!");
                }
            }

            tracing::info!("Initiating internal graceful shutdown sequence...");
            token_clone.cancel();
            tracing::info!("Background services stopped.");
        });

        Ok(shutdown_tx)
    }
}
