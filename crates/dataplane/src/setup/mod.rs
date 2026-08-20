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

use crate::cache::cache_redis::dataplane_transfer_cache::DataplaneTransferCacheForRedis;
use crate::data::factory_sql::DataplaneRepoForSql;
use crate::data::factory_trait::DataplaneRepoTrait;
use crate::entities::dataplane_drivers::keystore_lookup::KeystoreClientImpl;
use crate::entities::dataplane_manager::dataplane_driver_factory::DataplaneDriverFactory;
use crate::entities::dataplane_manager::dataplane_manager::DataplaneManager;
use crate::entities::dataplane_transfer_logs::dataplane_transfer_logs_entity::DataplaneTransferLogsEntityService;
use crate::entities::dataplane_transfers::dataplane_transfers_entity::DataplaneTransfersEntityService;
use crate::entities::transfer_events::transfer_event_entity::TransferEventEntityService;
use crate::http::dataplane_info::DataPlaneProcessesRouter;
use crate::http::dataplane_transfer_logs::DataplaneTransferLogsRouter;
use crate::http::transfer_events::TransferEventsRouter;
use crate::testing_proxy::http::http::TestingHTTPProxy;
use axum::Router;
use common::config::services::TransferConfig;
use common::config::types::traits::{CacheConfigTrait, CommonConfigTrait};
use common::http_client::HttpClient;
use connector::ConnectorInstanceTrait;
use keystore::KeystoreModule;
use keystore::SecretStore;
use std::sync::Arc;
use ymir::services::vault::global::VaultService;
use ymir::services::vault::VaultTrait;

/// Infrastructure shared by every entry point of the dataplane: the
/// Redis-backed cache and the SQL repository. Wiring it once here is what
/// lets the public builders below stay focused on their own concerns instead
/// of repeating the cache/repo bootstrap.
struct DataplaneInfra {
    cache: Arc<DataplaneTransferCacheForRedis>,
    repo: Arc<dyn DataplaneRepoTrait>,
}

/// Composition root for the dataplane: turns `config` + `vault` into the
/// concrete services, entities and routers the rest of the crate depends on.
pub struct DataplaneSetup {}

impl Default for DataplaneSetup {
    fn default() -> Self {
        Self::new()
    }
}

impl DataplaneSetup {
    pub fn new() -> Self {
        DataplaneSetup {}
    }

    // --- Shared building blocks ------------------------------------------

    /// Opens the Redis client from the cache URL declared in config.
    fn redis_client(&self, config: &TransferConfig) -> redis::Client {
        redis::Client::open(config.get_full_cache_url())
            .expect("dataplane setup: failed to open redis client")
    }

    /// Builds the SQL repository on top of the vault-provided DB connection.
    async fn build_repo(
        &self,
        config: &TransferConfig,
        vault: Arc<VaultService>,
    ) -> Arc<dyn DataplaneRepoTrait> {
        let db_connection = vault.get_db_connection(config.common()).await.unwrap();
        Arc::new(DataplaneRepoForSql::create_repo(db_connection))
    }

    /// Wires the cache + repository every builder needs. Single source of
    /// truth for the dataplane's infrastructure dependencies.
    async fn build_infra(
        &self,
        config: &TransferConfig,
        vault: Arc<VaultService>,
    ) -> DataplaneInfra {
        let redis_conn = self
            .redis_client(config)
            .get_multiplexed_async_connection()
            .await
            .expect("dataplane setup: failed to get redis connection");
        DataplaneInfra {
            cache: Arc::new(DataplaneTransferCacheForRedis::new(redis_conn)),
            repo: self.build_repo(config, vault).await,
        }
    }

    /// Builds the transfers entity service from the shared infrastructure.
    fn transfers_entity(&self, infra: &DataplaneInfra) -> Arc<DataplaneTransfersEntityService> {
        Arc::new(DataplaneTransfersEntityService::new(
            infra.repo.clone(),
            infra.cache.clone(),
        ))
    }

    /// Builds the keystore lookup client and returns its secret store too:
    /// the lookup is injected into drivers/proxy, while the secret store is
    /// also handed directly to the manager so it can resolve runtime secrets.
    async fn build_keystore(
        &self,
        config: &TransferConfig,
        vault: Arc<VaultService>,
    ) -> (Arc<KeystoreClientImpl>, Arc<dyn SecretStore>) {
        let (parameter_store, secret_store) = KeystoreModule::build_stores(config, vault).await;
        let lookup = Arc::new(KeystoreClientImpl::new(
            parameter_store,
            secret_store.clone(),
        ));
        (lookup, secret_store)
    }

    // --- Public composition roots ----------------------------------------

    /// Builds the dataplane manager: transfers entity + connector + a driver
    /// factory backed by the keystore, with the secret store wired in.
    pub async fn get_data_plane_manager(
        &self,
        config: Arc<TransferConfig>,
        vault: Arc<VaultService>,
        connector_entity: Arc<dyn ConnectorInstanceTrait>,
        _http_client: Arc<HttpClient>,
    ) -> DataplaneManager {
        let infra = self.build_infra(config.as_ref(), vault.clone()).await;
        let dataplane_process_entity = self.transfers_entity(&infra);
        let (keystore_lookup, secret_store) = self.build_keystore(config.as_ref(), vault).await;

        DataplaneManager::new(dataplane_process_entity, connector_entity, config.clone())
            .with_driver_factory(Arc::new(
                DataplaneDriverFactory::new().with_keystore(keystore_lookup),
            ))
            .with_secret_store(secret_store)
    }

    /// Builds the control-plane router exposing transfer processes, logs and
    /// events under their public mount points.
    pub async fn build_control_router(
        &self,
        config: &TransferConfig,
        vault: Arc<VaultService>,
    ) -> Router {
        let infra = self.build_infra(config, vault).await;

        // Events: one entity feeding both the per-process feed and the global lookup.
        let transfer_event_entity = Arc::new(TransferEventEntityService::new(infra.repo.clone()));
        let transfer_event_service = TransferEventsRouter::new(transfer_event_entity.clone());
        let dataplane_processes_events_router = transfer_event_service
            .clone()
            .dataplane_processes_sub_router();
        let events_lookup_router = transfer_event_service.events_sub_router();

        // Transfer logs.
        let logs_entity = Arc::new(DataplaneTransferLogsEntityService::new(infra.repo.clone()));
        let logs_router = DataplaneTransferLogsRouter::new(logs_entity).router();

        // Transfer processes (CRUD + event association).
        let dataplane_process_entity = self.transfers_entity(&infra);
        let dataplane_processes_router =
            DataPlaneProcessesRouter::new(dataplane_process_entity, transfer_event_entity.clone())
                .router();

        // Compose the process-scoped routers, then mount everything.
        let dataplane_processes_router = Router::new()
            .merge(dataplane_processes_router)
            .merge(logs_router)
            .merge(dataplane_processes_events_router);

        Router::new()
            .nest("/dataplane-processes", dataplane_processes_router)
            .nest("/transfer-events", events_lookup_router)
    }

    /// Builds the standalone testing HTTP proxy with keystore-backed lookup.
    pub async fn build_testing_proxy(
        &self,
        config: &TransferConfig,
        vault: Arc<VaultService>,
    ) -> Router {
        let infra = self.build_infra(config, vault.clone()).await;
        let dataplane_process_entity = self.transfers_entity(&infra);
        let (keystore_lookup, _secret_store) = self.build_keystore(config, vault).await;

        TestingHTTPProxy::new(dataplane_process_entity, infra.repo)
            .with_keystore(keystore_lookup)
            .router()
    }
}
