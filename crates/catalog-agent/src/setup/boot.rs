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

use crate::setup::grpc_worker::CatalogGrpcWorker;
use crate::setup::http_worker::CatalogHttpWorker;
use crate::setup::CatalogAgentModule;
use common::auth::ServiceHttpClient;
use common::boot::shutdown::shutdown_signal;
use common::boot::BootstrapServiceTrait;
use common::config::services::traits::CatalogConfigTrait;
use common::config::services::{CatalogConfig, ContractsConfig, TransferConfig};
use common::config::types::roles::RoleConfig;
use common::config::types::traits::{CommonConfigTrait, ConfigLoader, MinKnownConfigTrait};
use common::module_loader::service_composer::ServiceComposer;
use common::worker_utils::GrpcServer;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::{fs, signal};
use tokio_util::sync::CancellationToken;
use tracing::error;
use ymir::config::traits::{ApiConfigTrait, HostsConfigTrait};
use ymir::config::types::HostType;
use ymir::data::entities::shared::participant;
use ymir::errors::{Errors, Outcome};
use ymir::services::vault::global::VaultService;

pub struct CatalogAgentBoot;

impl CatalogAgentBoot {
    /// Boot goes through the same idempotent provisioning as any new tenant.
    async fn provision_admin_tenant(config: &CatalogConfig) -> Outcome<serde_json::Value> {
        let client = ServiceHttpClient::from_common(config.common(), 30);
        let tenant = &config.common().admin_seed.tenant_id;
        let url = format!(
            "{}{}/catalog-agent/tenants/{}/provision",
            config.common().get_host(HostType::Http),
            config.common().get_api_version(),
            tenant
        );
        client
            .post_json(&url, Some(tenant), &serde_json::json!({}))
            .await
    }

    fn string_at(value: &serde_json::Value, pointer: &str) -> Outcome<String> {
        value
            .pointer(pointer)
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .ok_or_else(|| Errors::parse(format!("provisioning response lacks {pointer}"), None))
    }
}

#[async_trait::async_trait]
impl BootstrapServiceTrait for CatalogAgentBoot {
    type Config = CatalogConfig;
    async fn load_config(env_file: String) -> Outcome<Self::Config> {
        let config = Self::Config::load(&*env_file)?;
        let table = json_to_table::json_to_table(&serde_json::to_value(&config)?)
            .collapse()
            .to_string();
        tracing::info!("Current Catalog Agent Config:\n{}", table);
        Ok(config)
    }
    async fn create_participant(config: &Self::Config) -> Outcome<String> {
        let client = ServiceHttpClient::from_common(config.common(), 30);
        let base_url = config.ssi_auth().get_host(HostType::Http);
        let api = config.ssi_auth().get_api_version();
        let url = format!("{}{}/mates/myself", base_url, api);
        let tenant = &config.common().admin_seed.tenant_id;
        let participant: participant::Model = client.get_json(&url, Some(tenant)).await?;
        Ok(participant.participant_id)
    }

    async fn load_catalog(
        _participant_id: &Option<String>,
        config: &Self::Config,
    ) -> Outcome<String> {
        let provisioned = Self::provision_admin_tenant(config).await?;
        Self::string_at(&provisioned, "/catalog/id")
    }

    async fn load_dataservice(
        _catalog_id: &Option<String>,
        config: &Self::Config,
    ) -> Outcome<String> {
        let provisioned = Self::provision_admin_tenant(config).await?;
        Self::string_at(&provisioned, "/dataService/id")
    }

    async fn load_policy_templates(config: &Self::Config) -> Outcome<()> {
        let client = ServiceHttpClient::from_common(config.common(), 3);
        let tenant = &config.common().admin_seed.tenant_id;
        let base_url = config.common().get_host(HostType::Http);
        let api = config.common().get_api_version();
        let url = format!("{}{}/catalog-agent/policy-templates", base_url, api);
        // load files
        let policies_folder = config.get_policy_templates_folder();
        let mut read_dir = match fs::read_dir(&policies_folder).await {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to read folder: {}", e.to_string());
                return Ok(());
            }
        };
        while let Ok(Some(entry)) = read_dir.next_entry().await {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                let content = match fs::read_to_string(&path).await {
                    Ok(c) => c,
                    Err(e) => {
                        error!("Failed to read file {:?}: {}", path, e);
                        continue;
                    }
                };
                let json_payload: serde_json::Value = match serde_json::from_str(&content) {
                    Ok(json) => json,
                    Err(e) => {
                        error!("Invalid JSON format in file {:?}: {}", path, e);
                        continue;
                    }
                };
                let _ = match client
                    .post_json::<serde_json::Value, serde_json::Value>(
                        url.as_str(),
                        Some(tenant),
                        &json_payload,
                    )
                    .await
                {
                    Ok(_) => {}
                    Err(e) => {
                        error!("Invalid request {:?}: {}", path, e);
                        continue;
                    }
                };
            }
        }
        Ok(())
    }

    async fn start_services(
        config: &Self::Config,
        vault: Arc<VaultService>,
    ) -> Outcome<broadcast::Sender<()>> {
        // thread control
        let (shutdown_tx, mut shutdown_rx) = broadcast::channel(1);
        let cancel_token = CancellationToken::new();

        // workers
        tracing::info!("Spawning HTTP subsystem...");
        let http_handle = CatalogHttpWorker::spawn(config, vault.clone(), &cancel_token).await?;

        tracing::info!("Spawning gRPC subsystem...");
        let composer =
            ServiceComposer::new().register(CatalogAgentModule::compose(config, &vault).await?);
        let grpc_handle = CatalogGrpcWorker::spawn(config, &composer, &cancel_token).await?;

        // non-blocking thread
        let token_clone = cancel_token.clone();
        tokio::spawn(async move {
            tokio::select! {
                // ctrl+c
                _ = shutdown_rx.recv() => {
                    tracing::info!("Shutdown command received from Main Pipeline.");
                }
                _ = async { http_handle.await } => {
                    tracing::error!("HTTP subsystem failed or stopped unexpectedly!");
                }
                _ = GrpcServer::supervise(grpc_handle) => {
                    tracing::error!("GRPC subsystem failed or stopped unexpectedly!");
                }
            }

            tracing::info!("Initiating internal graceful shutdown sequence...");
            token_clone.cancel();
            tracing::info!("Background services stopped.");
        });

        Ok(shutdown_tx)
    }
}
