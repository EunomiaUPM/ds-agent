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

//! Wires the dataplane's cache, repository, keystore lookup, services and manager once.

use std::sync::Arc;

use common::auth::OauthTokenValidator;
use common::config::services::TransferConfig;
use common::config::types::traits::CacheConfigTrait;
use common::module_loader::root_context::RootContext;
use keystore::KeystoreModule;
use ymir::errors::{Errors, Outcome};

use crate::cache::cache_redis::dataplane_transfer_cache::DataplaneTransferCacheForRedis;
use crate::data::factory_trait::DataplaneRepoTrait;
use crate::data::sea_orm::SeaOrmDataFactory;
use crate::engine::dataplane_drivers::keystore_lookup::KeystoreClientImpl;
use crate::engine::dataplane_manager::dataplane_driver_factory::DataplaneDriverFactory;
use crate::engine::dataplane_manager::dataplane_manager::DataplaneManager;
use crate::services::dataplane_transfer_logs::service::DataplaneTransferLogsService;
use crate::services::dataplane_transfers::service::DataplaneTransferService;
use crate::services::transfer_events::service::TransferEventsService;
use crate::setup::ports::DataplanePorts;

#[derive(Clone)]
pub(crate) struct AppContext {
    pub repo: Arc<dyn DataplaneRepoTrait>,
    pub transfer_svc: Arc<DataplaneTransferService>,
    pub logs_svc: Arc<DataplaneTransferLogsService>,
    pub events_svc: Arc<TransferEventsService>,
    pub keystore_lookup: Arc<KeystoreClientImpl>,
    pub manager: Arc<DataplaneManager>,
    pub oauth_validator: Arc<dyn OauthTokenValidator>,
}

impl AppContext {
    pub async fn build(
        config: &TransferConfig,
        root: &RootContext,
        ports: &DataplanePorts,
    ) -> Outcome<Self> {
        let repo: Arc<dyn DataplaneRepoTrait> =
            Arc::new(SeaOrmDataFactory::create_repo(root.db.clone()));
        let cache = Arc::new(DataplaneTransferCacheForRedis::new(
            Self::redis_connection(config).await?,
        ));
        let transfer_svc = Arc::new(DataplaneTransferService::new(repo.clone(), cache));

        // The lookup feeds drivers and proxy; the secret store also goes to the manager.
        let (parameter_store, secret_store) = KeystoreModule::build_stores(root, None);
        let keystore_lookup = Arc::new(KeystoreClientImpl::new(
            parameter_store,
            secret_store.clone(),
        ));
        let manager = DataplaneManager::new(
            transfer_svc.clone(),
            ports.connector.clone(),
            Arc::new(config.clone()),
        )
        .with_driver_factory(Arc::new(
            DataplaneDriverFactory::new().with_keystore(keystore_lookup.clone()),
        ))
        .with_secret_store(secret_store);

        Ok(Self {
            logs_svc: Arc::new(DataplaneTransferLogsService::new(repo.clone())),
            events_svc: Arc::new(TransferEventsService::new(repo.clone())),
            repo,
            transfer_svc,
            keystore_lookup,
            manager: Arc::new(manager),
            oauth_validator: root.validator.clone(),
        })
    }

    /// Opens the Redis connection behind the transfer cache.
    async fn redis_connection(
        config: &TransferConfig,
    ) -> Outcome<redis::aio::MultiplexedConnection> {
        let client = redis::Client::open(config.get_full_cache_url()).map_err(|e| {
            Errors::crazy(
                "dataplane setup: failed to open redis client",
                Some(Box::new(e)),
            )
        })?;
        client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| {
                Errors::crazy(
                    "dataplane setup: failed to get redis connection",
                    Some(Box::new(e)),
                )
            })
    }
}
