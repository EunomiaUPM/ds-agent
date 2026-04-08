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
use crate::entities::dataplane_manager::dataplane_manager::DataplaneManager;
use crate::entities::dataplane_manager::driver_factory::DataplaneDriverFactory;
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
use sea_orm::Database;
use std::ops::Deref;
use std::sync::Arc;
use ymir::config::traits::HostsConfigTrait;
use ymir::services::vault::global::VaultService;
use ymir::services::vault::VaultTrait;

pub struct DataplaneSetup {}
impl DataplaneSetup {
    pub fn new() -> Self {
        DataplaneSetup {}
    }

    async fn get_redis_client(&self, config: &TransferConfig) -> redis::Client {
        redis::Client::open(config.get_full_cache_url()).expect("Failed to open redis client")
    }

    async fn get_data_plane_repo(
        &self,
        config: &TransferConfig,
        vault: Arc<VaultService>,
    ) -> Arc<dyn DataplaneRepoTrait> {
        let db_connection = vault.get_db_connection(config.common()).await;
        let dataplane_repo = Arc::new(DataplaneRepoForSql::create_repo(db_connection.clone()));
        dataplane_repo
    }

    pub async fn get_data_plane_manager(
        &self,
        config: Arc<TransferConfig>,
        vault: Arc<VaultService>,
        connector_entity: Arc<dyn ConnectorInstanceTrait>,
        http_client: Arc<HttpClient>,
    ) -> DataplaneManager {
        let db_connection = vault.get_db_connection(config.deref().common()).await;
        let redis_client = self.get_redis_client(&config).await;
        let redis_conn = redis_client
            .get_multiplexed_async_connection()
            .await
            .expect("Failed to get redis connection");

        // cache
        let cache = Arc::new(DataplaneTransferCacheForRedis::new(redis_conn));
        // repo
        let dataplane_repo = self
            .get_data_plane_repo(config.as_ref(), vault.clone())
            .await;

        // entity
        let dataplane_process_entity = Arc::new(DataplaneTransfersEntityService::new(
            dataplane_repo.clone(),
            cache,
        ));

        // driver factory
        let driver_factory = Arc::new(DataplaneDriverFactory::new(config.clone()));

        DataplaneManager::new(dataplane_process_entity, connector_entity, driver_factory)
    }

    pub async fn build_control_router(
        &self,
        config: &TransferConfig,
        vault: Arc<VaultService>,
    ) -> Router {
        let redis_client = self.get_redis_client(config).await;
        let redis_conn = redis_client
            .get_multiplexed_async_connection()
            .await
            .expect("Failed to get redis connection");

        // cache
        let cache = Arc::new(DataplaneTransferCacheForRedis::new(redis_conn));
        // repo
        let dataplane_repo = self.get_data_plane_repo(config, vault.clone()).await;

        // entities and routres
        let transfer_event_entity = Arc::new(TransferEventEntityService::new(&dataplane_repo));
        let transfer_event_service = TransferEventsRouter::new(transfer_event_entity.clone());
        let dataplane_processes_events_router = transfer_event_service
            .clone()
            .dataplane_processes_sub_router();
        let events_lookup_router = transfer_event_service.events_sub_router();
        let logs_entity = Arc::new(DataplaneTransferLogsEntityService::new(
            dataplane_repo.clone(),
        ));
        let logs_router = DataplaneTransferLogsRouter::new(logs_entity).router();
        let dataplane_process_entity = Arc::new(DataplaneTransfersEntityService::new(
            dataplane_repo.clone(),
            cache,
        ));
        let dataplane_processes_router = DataPlaneProcessesRouter::new(
            dataplane_process_entity.clone(),
            transfer_event_entity.clone(),
        )
        .router();

        let dataplane_processes_router = Router::new()
            .merge(dataplane_processes_router)
            .merge(logs_router)
            .merge(dataplane_processes_events_router);

        // merge router
        Router::new()
            .nest("/dataplane-processes", dataplane_processes_router)
            .nest("/transfer-events", events_lookup_router)
    }

    pub async fn build_testing_proxy(
        &self,
        config: &TransferConfig,
        vault: Arc<VaultService>,
    ) -> Router {
        let redis_client = self.get_redis_client(config).await;
        let redis_conn = redis_client
            .get_multiplexed_async_connection()
            .await
            .expect("Failed to get redis connection");

        let cache = Arc::new(DataplaneTransferCacheForRedis::new(redis_conn));
        let dataplane_repo = self.get_data_plane_repo(config, vault.clone()).await;

        let dataplane_process_entity = Arc::new(DataplaneTransfersEntityService::new(
            dataplane_repo.clone(),
            cache,
        ));
        TestingHTTPProxy::new(dataplane_process_entity.clone(), dataplane_repo).router()
    }
}
