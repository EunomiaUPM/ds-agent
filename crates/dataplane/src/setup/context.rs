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

use std::sync::{Arc, Mutex};

use ymir::services::token_validator::OauthTokenValidatorTrait;
use common::config::services::traits::TransferConfigTrait;
use common::config::services::TransferConfig;
use common::config::types::traits::CacheConfigTrait;
use connector::{ConnectorInstanceFacadeTrait, KeystoreLookup};
use ymir::module_loader::root_context::RootContext;
use keystore::KeystoreModule;
use ymir::errors::{Errors, Outcome};

use crate::cache::cache_redis::dataplane_transfer_cache::DataplaneTransferCacheForRedis;
use crate::cache::L1DataplaneTransferCache;
use crate::data::factory_trait::DataplaneRepoTrait;
use crate::data::sea_orm::SeaOrmDataFactory;
use crate::engine::dataplane_drivers::keystore_lookup::KeystoreClientImpl;
use crate::engine::dataplane_manager::dataplane_driver_factory::DataplaneDriverFactory;
use crate::engine::dataplane_manager::dataplane_manager::DataplaneManager;
use crate::proxy::{DeliveryTracker, ProxyAuditSender, ProxyAuditWorker};
use crate::relay::{KafkaLeaseManager, KafkaRelaySupervisor, RdkafkaConsumerFactory};
use crate::services::dataplane_transfer_logs::service::DataplaneTransferLogsService;
use crate::services::dataplane_transfers::service::DataplaneTransferService;
use crate::services::transfer_events::service::TransferEventsService;
use crate::setup::ports::DataplanePorts;
use crate::signals::DataplaneSignalSender;
use events::EventSinkTrait;

#[derive(Clone)]
pub struct AppContext {
    pub repo: Arc<dyn DataplaneRepoTrait>,
    pub transfer_svc: Arc<DataplaneTransferService>,
    pub logs_svc: Arc<DataplaneTransferLogsService>,
    pub events_svc: Arc<TransferEventsService>,
    pub keystore_lookup: Arc<KeystoreClientImpl>,
    pub connector: Arc<dyn ConnectorInstanceFacadeTrait>,
    pub manager: Arc<DataplaneManager>,
    pub oauth_validator: Arc<dyn OauthTokenValidatorTrait>,
    pub signals: DataplaneSignalSender,
    pub tracker: Arc<DeliveryTracker>,
    pub kafka_supervisor: Arc<KafkaRelaySupervisor>,
    pub events: Arc<dyn EventSinkTrait>,
    pub audit_tx: ProxyAuditSender,
    audit_worker: Arc<Mutex<Option<ProxyAuditWorker>>>,
}

impl AppContext {
    pub fn take_audit_worker(&self) -> Option<ProxyAuditWorker> {
        self.audit_worker
            .lock()
            .ok()
            .and_then(|mut guard| guard.take())
    }

    pub async fn build(
        config: &TransferConfig,
        root: &RootContext,
        events: Arc<dyn EventSinkTrait>,
        ports: &DataplanePorts,
    ) -> Outcome<Self> {
        let repo: Arc<dyn DataplaneRepoTrait> =
            Arc::new(SeaOrmDataFactory::create_repo(root.db.clone()));
        let redis_conn = Self::redis_connection(config).await?;
        let tracker = Arc::new(DeliveryTracker::with_redis(redis_conn.clone()));
        let kafka_lease = Arc::new(KafkaLeaseManager::with_redis(redis_conn.clone()));
        let cache = Arc::new(L1DataplaneTransferCache::new(
            DataplaneTransferCacheForRedis::new(redis_conn),
        ));
        let (audit_tx, audit_worker) =
            ProxyAuditWorker::open(repo.clone(), ProxyAuditWorker::DEFAULT_CAPACITY);
        let audit_worker = audit_worker.with_events(events.clone());
        let transfer_svc =
            Arc::new(DataplaneTransferService::new(repo.clone(), cache).with_events(events.clone()));

        // The lookup feeds drivers and proxy; the secret store also goes to the manager.
        let (parameter_store, secret_store) = KeystoreModule::build_stores(root, events.clone());
        let keystore_lookup = Arc::new(KeystoreClientImpl::new(
            parameter_store,
            secret_store.clone(),
        ));
        let keystore_dyn: Arc<dyn KeystoreLookup> = keystore_lookup.clone();
        let kafka_supervisor = Arc::new(
            KafkaRelaySupervisor::new(kafka_lease, Arc::new(RdkafkaConsumerFactory::new()))
                .with_repo(repo.clone())
                .with_tracker(tracker.clone())
                .with_signals(ports.signals.clone())
                .with_events(events.clone())
                .with_accept_invalid_certs(config.proxy_accept_invalid_certs())
                .with_allow_private_egress(config.proxy_allow_private_egress()),
        );

        let manager = DataplaneManager::new(
            transfer_svc.clone(),
            ports.connector.clone(),
            Arc::new(config.clone()),
        )
        .with_driver_factory(Arc::new(
            DataplaneDriverFactory::new()
                .with_keystore(keystore_dyn)
                .with_kafka_supervisor(kafka_supervisor.clone()),
        ))
        .with_secret_store(secret_store)
        .with_signals(ports.signals.clone());

        Ok(Self {
            logs_svc: Arc::new(DataplaneTransferLogsService::new(repo.clone())),
            events_svc: Arc::new(TransferEventsService::new(repo.clone())),
            repo,
            transfer_svc,
            keystore_lookup,
            connector: ports.connector.clone(),
            manager: Arc::new(manager),
            oauth_validator: root.validator.clone(),
            signals: ports.signals.clone(),
            tracker,
            kafka_supervisor,
            events,
            audit_tx,
            audit_worker: Arc::new(Mutex::new(Some(audit_worker))),
        })
    }

    /// Opens the Redis connection behind the transfer cache.
    async fn redis_connection(
        config: &TransferConfig,
    ) -> Outcome<redis::aio::MultiplexedConnection> {
        let client = redis::Client::open(config.get_full_cache_url()?).map_err(|e| {
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
