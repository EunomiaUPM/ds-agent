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
use crate::setup::grpc_worker::CatalogGrpcWorker;
use crate::setup::http_worker::CatalogHttpWorker;
use crate::{CatalogDto, DataServiceDto, NewCatalogDto, NewDataServiceDto};
use rainbow_common::boot::shutdown::shutdown_signal;
use rainbow_common::boot::BootstrapServiceTrait;
use rainbow_common::config::services::{CatalogConfig, ContractsConfig, TransferConfig};
use rainbow_common::config::types::traits::{CommonConfigTrait, ConfigLoader};
use rainbow_common::config::types::roles::RoleConfig;
use rainbow_common::http_client::{HttpClient, HttpClientError};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::{fs, signal};
use tokio_util::sync::CancellationToken;
use tracing::error;
use urn::Urn;
use ymir::config::traits::{ApiConfigTrait, HostsConfigTrait};
use ymir::config::types::HostType;
use ymir::data::entities::mates;
use ymir::services::vault::global::VaultService;

pub struct CatalogAgentBoot;

#[async_trait::async_trait]
impl BootstrapServiceTrait for CatalogAgentBoot {
    type Config = CatalogConfig;
    async fn load_config(env_file: String) -> anyhow::Result<Self::Config> {
        let config = Self::Config::load(env_file);
        let table =
            json_to_table::json_to_table(&serde_json::to_value(&config)?).collapse().to_string();
        tracing::info!("Current Catalog Agent Config:\n{}", table);
        Ok(config)
    }
    async fn create_participant(config: &Self::Config) -> anyhow::Result<String> {
        let client = HttpClient::new(1, 30);
        let base_url = config.ssi_auth().get_host(HostType::Http);
        let api = config.ssi_auth().get_api_version();

        // attempt first
        let url = format!("{}{}/mates/myself", base_url, api);
        let participant = client.get_json::<mates::Model>(url.as_str()).await;

        // catch error
        if let Err(err) = participant {
            match err {
                // if mate not found
                HttpClientError::HttpError { status, .. } if status.as_u16() == 404 => {
                    // onboard mate with wallet
                    let url = format!("{}{}/wallet/onboard", base_url, api);
                    client.post_void::<()>(url.as_str()).await?;
                }
                _ => anyhow::bail!(err),
            }
            // attempt again
            let url = format!("{}{}/mates/myself", base_url, api);
            let participant = client.get_json::<mates::Model>(url.as_str()).await?;
            // and return id
            Ok(participant.participant_id)
        } else {
            // if mate exists, just return id
            let participant = participant?;
            Ok(participant.participant_id)
        }
    }

    async fn load_catalog(
        participant_id: &Option<String>,
        config: &Self::Config,
    ) -> anyhow::Result<String> {
        let participant_id = participant_id.clone().unwrap_or_default();
        let client = HttpClient::new(1, 3);
        let base_url = config.common().get_host(HostType::Http);
        let api = config.common().get_api_version();
        let url = format!("{}{}/catalog-agent/catalogs/main", base_url, api);
        let catalog = client
            .post_json::<NewCatalogDto, CatalogDto>(
                url.as_str(),
                &NewCatalogDto {
                    dspace_participant_id: Some(participant_id),
                    ..NewCatalogDto::default()
                },
            )
            .await?;
        Ok(catalog.inner.id)
    }

    async fn load_dataservice(
        catalog_id: &Option<String>,
        config: &Self::Config,
    ) -> anyhow::Result<String> {
        let catalog_id = catalog_id.clone().unwrap_or_default();
        let client = HttpClient::new(1, 3);
        let base_url = config.common().get_host(HostType::Http);
        let negotiation_url = config.contracts().get_host(HostType::Http);
        let api = config.common().get_api_version();
        let url = format!("{}{}/catalog-agent/data-services/main", base_url, api);
        let catalog = client
            .post_json::<NewDataServiceDto, DataServiceDto>(
                url.as_str(),
                &NewDataServiceDto {
                    dcat_endpoint_url: format!("{}/dsp/current", negotiation_url),
                    catalog_id: Urn::from_str(catalog_id.as_str())?,
                    ..Default::default()
                },
            )
            .await?;
        Ok(catalog.inner.id)
    }

    async fn load_policy_templates(config: &Self::Config) -> anyhow::Result<()> {
        let client = HttpClient::new(1, 3);
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
                    .post_json::<serde_json::Value, serde_json::Value>(url.as_str(), &json_payload)
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

    async fn start_services_background(
        config: &Self::Config,
        vault: Arc<VaultService>,
    ) -> anyhow::Result<broadcast::Sender<()>> {
        // thread control
        let (shutdown_tx, mut shutdown_rx) = broadcast::channel(1);
        let cancel_token = CancellationToken::new();

        // workers
        tracing::info!("Spawning HTTP subsystem...");
        let http_handle = CatalogHttpWorker::spawn(config, vault.clone(), &cancel_token).await?;

        tracing::info!("Spawning gRPC subsystem...");
        let grpc_handle = CatalogGrpcWorker::spawn(config, vault.clone(), &cancel_token).await?;

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
                _ = async { grpc_handle.await } => {
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
