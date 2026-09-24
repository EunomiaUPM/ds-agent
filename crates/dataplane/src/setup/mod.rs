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
use crate::data::factory_trait::DataplaneRepoTrait;
use crate::data::sea_orm::SeaOrmDataFactory;
use crate::engine::dataplane_drivers::keystore_lookup::KeystoreClientImpl;
use crate::engine::dataplane_manager::dataplane_driver_factory::DataplaneDriverFactory;
use crate::engine::dataplane_manager::dataplane_manager::DataplaneManager;
use crate::http::dataplane_info::DataPlaneProcessesRouter;
use crate::http::dataplane_transfer_logs::DataplaneTransferLogsRouter;
use crate::http::transfer_events::TransferEventsRouter;
use crate::services::dataplane_transfer_logs::DataplaneTransferLogsService;
use crate::services::dataplane_transfers::DataplaneTransferService;
use crate::services::transfer_events::TransferEventsService;
use crate::testing_proxy::http::http::TestingHTTPProxy;
use axum::Router;
use common::config::services::TransferConfig;
use common::config::types::traits::CacheConfigTrait;
use common::module_loader::root_context::RootContext;
use connector::ConnectorInstanceServiceTrait;
use keystore::KeystoreModule;
use keystore::SecretStore;
use std::sync::Arc;

/// Infrastructure shared by every entry point of the dataplane: the
/// Redis-backed cache and the SQL repository. Wiring it once here is what
/// lets the public builders below stay focused on their own concerns instead
/// of repeating the cache/repo bootstrap.
struct DataplaneInfra {
    cache: Arc<DataplaneTransferCacheForRedis>,
    repo: Arc<dyn DataplaneRepoTrait>,
}

/// Composition root for the dataplane: turns `config` + the root context into the
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

    /// Builds the SQL repository on top of the shared DB pool.
    fn build_repo(&self, root: &RootContext) -> Arc<dyn DataplaneRepoTrait> {
        Arc::new(SeaOrmDataFactory::create_repo(root.db.clone()))
    }

    /// Wires the cache + repository every builder needs. Single source of
    /// truth for the dataplane's infrastructure dependencies.
    async fn build_infra(&self, config: &TransferConfig, root: &RootContext) -> DataplaneInfra {
        let redis_conn = self
            .redis_client(config)
            .get_multiplexed_async_connection()
            .await
            .expect("dataplane setup: failed to get redis connection");
        DataplaneInfra {
            cache: Arc::new(DataplaneTransferCacheForRedis::new(redis_conn)),
            repo: self.build_repo(root),
        }
    }

    /// Builds the transfers service from the shared infrastructure.
    fn transfers_service(&self, infra: &DataplaneInfra) -> Arc<DataplaneTransferService> {
        Arc::new(DataplaneTransferService::new(
            infra.repo.clone(),
            infra.cache.clone(),
        ))
    }

    /// Builds the keystore lookup client and returns its secret store too:
    /// the lookup is injected into drivers/proxy, while the secret store is
    /// also handed directly to the manager so it can resolve runtime secrets.
    fn build_keystore(
        &self,
        root: &RootContext,
    ) -> (Arc<KeystoreClientImpl>, Arc<dyn SecretStore>) {
        let (parameter_store, secret_store) = KeystoreModule::build_stores(root, None);
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
        root: &RootContext,
        connector_service: Arc<dyn ConnectorInstanceServiceTrait>,
    ) -> DataplaneManager {
        let infra = self.build_infra(config.as_ref(), root).await;
        let transfer_service = self.transfers_service(&infra);
        let (keystore_lookup, secret_store) = self.build_keystore(root);

        DataplaneManager::new(transfer_service, connector_service, config.clone())
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
        root: &RootContext,
    ) -> Router {
        let infra = self.build_infra(config, root).await;
        let validator = root.validator.clone();

        // Events: service feeding both the per-process feed and the global lookup.
        let transfer_event_service = Arc::new(TransferEventsService::new(infra.repo.clone()));
        let transfer_events_router = TransferEventsRouter::new(transfer_event_service);
        let dataplane_processes_events_router = transfer_events_router
            .clone()
            .dataplane_processes_sub_router();
        let events_lookup_router = transfer_events_router.events_sub_router();

        // Transfer logs.
        let logs_service = Arc::new(DataplaneTransferLogsService::new(infra.repo.clone()));
        let logs_router = DataplaneTransferLogsRouter::new(logs_service).router();

        // Transfer processes (CRUD + process endpoints).
        let dataplane_transfer_service = Arc::new(DataplaneTransferService::new(
            infra.repo.clone(),
            infra.cache.clone(),
        ));
        let dataplane_processes_router =
            DataPlaneProcessesRouter::new(dataplane_transfer_service).router();

        // Compose the process-scoped routers, then mount everything.
        let dataplane_processes_router = Router::new()
            .merge(dataplane_processes_router)
            .merge(logs_router)
            .merge(dataplane_processes_events_router);

        Router::new()
            .nest("/dataplane-processes", dataplane_processes_router)
            .nest("/transfer-events", events_lookup_router)
            .route_layer(axum::middleware::from_fn_with_state(
                validator,
                common::auth::http::AuthHttpMiddleware::run,
            ))
    }

    /// Builds the standalone testing HTTP proxy with keystore-backed lookup.
    pub async fn build_testing_proxy(&self, config: &TransferConfig, root: &RootContext) -> Router {
        let infra = self.build_infra(config, root).await;
        let transfer_service = self.transfers_service(&infra);
        let (keystore_lookup, _secret_store) = self.build_keystore(root);

        TestingHTTPProxy::new(transfer_service, infra.repo)
            .with_keystore(keystore_lookup)
            .router()
    }
}
