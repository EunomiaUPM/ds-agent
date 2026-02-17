use crate::cache::cache_redis::dataplane_transfer_cache::DataplaneTransferCacheForRedis;
use crate::coordinator::data_source_connector::data_source_connector::DataSourceConnector;
use crate::coordinator::dataplane_access_controller::dataplane_access_controller::DataPlaneAccessControllerService;
use crate::coordinator::dataplane_access_controller::DataPlaneAccessControllerTrait;
use crate::data::factory_sql::DataplaneRepoForSql;
use crate::data::factory_trait::DataplaneRepoTrait;
use crate::entities::dataplane_transfer_logs::dataplane_transfer_logs_entity::DataplaneTransferLogsEntityService;
use crate::entities::dataplane_transfers::dataplane_transfers_entity::DataplaneTransfersEntityService;
use crate::entities::transfer_events::transfer_event_entity::TransferEventEntityService;
use crate::http::dataplane_info::DataPlaneRouter;
use crate::http::dataplane_transfer_logs::DataplaneTransferLogsRouter;
use crate::http::transfer_events::TransferEventsRouter;
use crate::testing_proxy::http::http::TestingHTTPProxy;
use axum::Router;
use rainbow_common::config::services::TransferConfig;
use rainbow_common::config::traits::{CacheConfigTrait, CommonConfigTrait};
use sea_orm::Database;
use std::ops::Deref;
use std::sync::Arc;
use ymir::services::vault::vault_rs::VaultService;
use ymir::services::vault::VaultTrait;

pub struct DataplaneSetup {}
impl DataplaneSetup {
    pub fn new() -> Self {
        DataplaneSetup {}
    }

    pub async fn get_redis_client(&self, config: &TransferConfig) -> redis::Client {
        redis::Client::open(config.get_full_cache_url()).expect("Failed to open redis client")
    }

    pub async fn get_data_plane_repo(
        &self,
        config: &TransferConfig,
        vault: Arc<VaultService>,
    ) -> Arc<dyn DataplaneRepoTrait> {
        let db_connection = vault.get_db_connection(config.common()).await;
        let dataplane_repo = Arc::new(DataplaneRepoForSql::create_repo(db_connection.clone()));
        dataplane_repo
    }
    pub async fn get_data_plane_controller(
        &self,
        config: Arc<TransferConfig>,
        vault: Arc<VaultService>,
    ) -> Arc<dyn DataPlaneAccessControllerTrait> {
        let db_connection = vault.get_db_connection(config.deref().common()).await;
        let dataplane_repo = self.get_data_plane_repo(config.as_ref(), vault.clone()).await;

        let redis_client = self.get_redis_client(&config).await;
        let redis_conn = redis_client
            .get_multiplexed_async_connection()
            .await
            .expect("Failed to get redis connection");
        let cache = Arc::new(DataplaneTransferCacheForRedis::new(redis_conn));

        let dataplane_process_entity =
            Arc::new(DataplaneTransfersEntityService::new(dataplane_repo.clone(), cache));

        let dataplane_source_connector = Arc::new(DataSourceConnector::new());
        let controller = Arc::new(DataPlaneAccessControllerService::new(
            dataplane_source_connector.clone(),
            dataplane_process_entity.clone(),
            config.clone(),
        ));
        controller
    }
    pub async fn build_control_router(
        &self,
        config: &TransferConfig,
        vault: Arc<VaultService>,
    ) -> Router {
        let dataplane_repo = self.get_data_plane_repo(config, vault.clone()).await;

        let redis_client = self.get_redis_client(config).await;
        let redis_conn = redis_client
            .get_multiplexed_async_connection()
            .await
            .expect("Failed to get redis connection");
        let cache = Arc::new(DataplaneTransferCacheForRedis::new(redis_conn));

        let dataplane_process_entity =
            Arc::new(DataplaneTransfersEntityService::new(dataplane_repo.clone(), cache));

        let transfer_event_entity = Arc::new(TransferEventEntityService::new(&dataplane_repo));

        let logs_entity = Arc::new(DataplaneTransferLogsEntityService::new(dataplane_repo.clone()));

        let dataplane_router =
            DataPlaneRouter::new(dataplane_process_entity.clone(), transfer_event_entity.clone())
                .router();

        let transfer_event_service = TransferEventsRouter::new(transfer_event_entity.clone());

        let transfers_events_router = transfer_event_service.clone().transfers_sub_router();
        let events_lookup_router = transfer_event_service.events_sub_router();

        let logs_router = DataplaneTransferLogsRouter::new(logs_entity).router();

        let transfers_router =
            Router::new().merge(dataplane_router).merge(logs_router).merge(transfers_events_router);

        Router::new()
            .nest("/dataplane-process", transfers_router)
            .nest("/transfer-events", events_lookup_router)
    }
    pub async fn build_testing_proxy(
        &self,
        config: &TransferConfig,
        vault: Arc<VaultService>,
    ) -> Router {
        let dataplane_repo = self.get_data_plane_repo(config, vault.clone()).await;

        let redis_client = self.get_redis_client(config).await;
        let redis_conn = redis_client
            .get_multiplexed_async_connection()
            .await
            .expect("Failed to get redis connection");
        let cache = Arc::new(DataplaneTransferCacheForRedis::new(redis_conn));

        let dataplane_process_entity =
            Arc::new(DataplaneTransfersEntityService::new(dataplane_repo.clone(), cache));

        TestingHTTPProxy::new(dataplane_process_entity.clone()).router()
    }
}
