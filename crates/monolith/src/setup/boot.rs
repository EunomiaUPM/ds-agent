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

use std::str::FromStr;
use std::sync::Arc;

use catalog_agent::{CatalogDto, DataServiceDto, NewCatalogDto, NewDataServiceDto};
use common::boot::BootstrapServiceTrait;
use common::config::services::traits::CatalogConfigTrait;
use common::config::types::traits::{CacheConfigTrait, CommonConfigTrait};
use common::config::ApplicationConfig;
use common::http_client::{HttpClient, HttpClientError};
use common::module_loader::service_composer::ServiceComposer;
use common::utils::flush_redis_cache;
use tokio::fs;
use tokio::sync::broadcast;
use tokio::sync::broadcast::Sender;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};
use urn::Urn;
use ymir::config::traits::{ApiConfigTrait, HostsConfigTrait};
use ymir::config::types::HostType;
use ymir::data::entities::shared::participant;
use ymir::errors::{Errors, Outcome};
use ymir::services::vault::global::VaultService;
use ymir::services::vault::VaultTrait;

use crate::setup::composition::MonolithModule;
use crate::setup::CoreHttpWorker;

pub struct CoreBoot;

#[async_trait::async_trait]
impl BootstrapServiceTrait for CoreBoot {
    type Config = ApplicationConfig;
    async fn load_config(env_file: String) -> Outcome<Self::Config> {
        let config = Self::Config::load(&env_file)?;
        let config_value = serde_json::to_value(&config)?;
        let table = json_to_table::json_to_table(&config_value)
            .collapse()
            .into_table();
        info!("Current Monolith Dataspace Agent Config:\n{}", table);
        Ok(config)
    }

    async fn create_participant(config: &Self::Config) -> Outcome<String> {
        let client = HttpClient::new(1, 30);
        let base_url = config.ssi_auth().common().get_host(HostType::Http);
        let api = config.ssi_auth().common().get_api_version();

        // attempt first
        let url = format!("{}{}/mates/myself", base_url, api);
        let url = url.replace("host.docker.internal", "127.0.0.1");
        let participant = client.get_json::<participant::Model>(url.as_str()).await;

        // catch error
        if let Err(err) = participant {
            match err {
                // if mate not found
                HttpClientError::HttpError { status, .. } if status.as_u16() == 404 => {
                    // onboard mate with wallet
                    let url = format!("{}{}/wallet/link", base_url, api);
                    client.post_void::<()>(url.as_str()).await?;
                }
                _ => return Err(err.into()),
            }
            // attempt again
            let url = format!("{}{}/mates/myself", base_url, api);
            let participant = client.get_json::<participant::Model>(url.as_str()).await?;
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
    ) -> Outcome<String> {
        let participant_id = participant_id.clone().unwrap_or_default();
        let client = HttpClient::new(1, 3);
        let base_url = config.catalog().common().get_host(HostType::Http);
        let api = config.catalog().common().get_api_version();
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
    ) -> Outcome<String> {
        let catalog_id = catalog_id.clone().unwrap_or_default();
        let client = HttpClient::new(1, 3);
        let base_url = config.catalog().common().get_host(HostType::Http);
        let negotiation_url = config.contracts().common().get_host(HostType::Http);

        let api = config.catalog().common().get_api_version();
        let url = format!("{}{}/catalog-agent/data-services/main", base_url, api);
        let catalog = client
            .post_json::<NewDataServiceDto, DataServiceDto>(
                url.as_str(),
                &NewDataServiceDto {
                    dcat_endpoint_url: format!("{}/dsp/current", negotiation_url),
                    catalog_id: Urn::from_str(catalog_id.as_str()).map_err(|e| {
                        Errors::parse("Error parsing urn catalog_id", Some(Box::new(e)))
                    })?,
                    ..Default::default()
                },
            )
            .await?;
        Ok(catalog.inner.id)
    }

    async fn load_policy_templates(config: &Self::Config) -> Outcome<()> {
        let client = HttpClient::new(1, 3);
        let base_url = config.catalog().common().get_host(HostType::Http);
        let api = config.catalog().common().get_api_version();
        let url = format!(
            "{}{}/catalog-agent/policy-templates?silent=true",
            base_url, api
        );
        // load files
        let policies_folder = config.catalog().get_policy_templates_folder();
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
                        warn!("Invalid request {:?}: {}", path, e);
                        continue;
                    }
                };
            }
        }
        Ok(())
    }

    async fn cleanup_cache(config: &Self::Config) -> Outcome<()> {
        let url = config.monolith().get_full_cache_url();
        info!("Flushing Redis at {}...", url);
        if let Err(e) = flush_redis_cache(&url).await {
            warn!("Failed to flush Redis at {}: {}", url, e);
        } else {
            info!("Redis cache flushed successfully.");
        }
        Ok(())
    }

    fn enable_user_seed() -> bool {
        true
    }

    async fn seed_users(config: &Self::Config) -> Outcome<()> {
        let vault = common::vault_utils::vault(config)?;
        let db = vault.get_db_connection(config.transfer().common()).await?;
        let admin = config.transfer().admin_seed();
        oauth::services::seed_admin_user(db, &admin.tenant_id, &admin.email, &admin.password).await
    }

    async fn start_services(
        config: &Self::Config,
        vault: Arc<VaultService>,
    ) -> Outcome<Sender<()>> {
        // thread control
        let (shutdown_tx, mut shutdown_rx) = broadcast::channel::<()>(1);
        let cancel_token = CancellationToken::new();

        // compose the service graph: every hosted agent as a module behind one composer
        info!("Composing service graph...");
        let composer =
            ServiceComposer::new().register(MonolithModule::compose(config, vault.clone()).await?);

        // workers
        info!("Spawning HTTP subsystem...");
        let http_handle =
            CoreHttpWorker::spawn(config, vault.clone(), composer.http_router(), &cancel_token)
                .await?;

        // non-blocking thread supervisor
        let token_clone = cancel_token.clone();
        tokio::spawn(async move {
            tokio::select! {
                _msg = shutdown_rx.recv() => {
                    info!("Shutdown command received from Main Pipeline.");
                }
                res = http_handle => {
                    match res {
                        Ok(_) => error!("HTTP subsystem stopped unexpectedly (task finished)."),
                        Err(e) => error!("HTTP subsystem panicked: {}", e),
                    }
                }
            }
            info!("Initiating internal graceful shutdown sequence...");
            token_clone.cancel();
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            info!("Background services supervisor finished.");
        });

        Ok(shutdown_tx)
    }
}
