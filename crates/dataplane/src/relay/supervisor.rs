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

//! In-process registry and lifecycle manager for active `KafkaRelayWorker` Tokio tasks.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{debug, warn};
use ymir::errors::Outcome;
use ymir::services::client::ClientService;

use crate::data::factory_trait::DataplaneRepoTrait;
use crate::proxy::DeliveryTracker;
use crate::relay::consumer::{KafkaConsumerConfig, KafkaConsumerFactory, RdkafkaConsumerFactory};
use crate::relay::lease::KafkaLeaseManager;
use crate::relay::worker::{
    KafkaRelayTaskSpec, KafkaRelayWorker, DEFAULT_KAFKA_RELAY_BACKOFF_MS,
};
use crate::signals::DataplaneSignalSender;

const STOP_JOIN_TIMEOUT: Duration = Duration::from_secs(5);

struct ActiveRelay {
    cancel: CancellationToken,
    handle: JoinHandle<()>,
}

/// Supervises active per-transfer `KafkaRelayWorker` tasks within a dataplane replica.
pub struct KafkaRelaySupervisor {
    lease: Arc<KafkaLeaseManager>,
    consumer_factory: Arc<dyn KafkaConsumerFactory>,
    client: Arc<ClientService>,
    repo: Option<Arc<dyn DataplaneRepoTrait>>,
    tracker: Arc<DeliveryTracker>,
    signals: DataplaneSignalSender,
    allow_private_egress: bool,
    backoff_ms: Vec<u64>,
    active: Mutex<HashMap<String, ActiveRelay>>,
}

impl Default for KafkaRelaySupervisor {
    fn default() -> Self {
        Self::new(
            Arc::new(KafkaLeaseManager::in_memory()),
            Arc::new(RdkafkaConsumerFactory::new()),
        )
    }
}

impl KafkaRelaySupervisor {
    pub fn new(
        lease: Arc<KafkaLeaseManager>,
        consumer_factory: Arc<dyn KafkaConsumerFactory>,
    ) -> Self {
        Self {
            lease,
            consumer_factory,
            client: Arc::new(KafkaRelayWorker::build_http_client(false)),
            repo: None,
            tracker: Arc::new(DeliveryTracker::in_memory()),
            signals: DataplaneSignalSender::noop(),
            allow_private_egress: false,
            backoff_ms: DEFAULT_KAFKA_RELAY_BACKOFF_MS.to_vec(),
            active: Mutex::new(HashMap::new()),
        }
    }

    pub fn with_repo(mut self, repo: Arc<dyn DataplaneRepoTrait>) -> Self {
        self.repo = Some(repo);
        self
    }

    pub fn with_tracker(mut self, tracker: Arc<DeliveryTracker>) -> Self {
        self.tracker = tracker;
        self
    }

    pub fn with_signals(mut self, signals: DataplaneSignalSender) -> Self {
        self.signals = signals;
        self
    }

    pub fn with_accept_invalid_certs(mut self, accept_invalid_certs: bool) -> Self {
        self.client = Arc::new(KafkaRelayWorker::build_http_client(accept_invalid_certs));
        self
    }

    pub fn with_allow_private_egress(mut self, allow_private_egress: bool) -> Self {
        self.allow_private_egress = allow_private_egress;
        self
    }

    pub fn with_backoff_ms(mut self, backoff_ms: Vec<u64>) -> Self {
        self.backoff_ms = backoff_ms;
        self
    }

    /// Runs broker & topic metadata preflight (used synchronously during `TransferStart`).
    pub async fn preflight(&self, cfg: &KafkaConsumerConfig) -> Outcome<()> {
        self.consumer_factory.preflight(cfg).await
    }

    /// Returns `true` if a live relay task for `dataplane_id` is currently running locally.
    pub fn is_running(&self, dataplane_id: &str) -> bool {
        let mut guard = self.active.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(entry) = guard.get(dataplane_id) {
            if !entry.handle.is_finished() {
                return true;
            }
            guard.remove(dataplane_id);
        }
        false
    }

    /// Idempotently ensures a `KafkaRelayWorker` is running for `spec.dataplane_id` if no other
    /// replica holds the distributed lease.
    pub async fn ensure_running(&self, spec: KafkaRelayTaskSpec) -> Outcome<bool> {
        let dp_id = spec.dataplane_id.clone();
        if self.is_running(&dp_id) {
            return Ok(true);
        }

        if !self.lease.acquire(&dp_id).await {
            debug!(
                dataplane_id = %dp_id,
                "Another dataplane replica holds the Kafka relay lease; skipping local spawn"
            );
            return Ok(false);
        }

        let session = match self
            .consumer_factory
            .create_session(&spec.consumer_config)
            .await
        {
            Ok(s) => s,
            Err(e) => {
                self.lease.release(&dp_id).await;
                return Err(e);
            }
        };

        let cancel = CancellationToken::new();
        let child_cancel = cancel.child_token();
        let worker = KafkaRelayWorker {
            spec,
            session,
            client: self.client.clone(),
            lease: self.lease.clone(),
            repo: self.repo.clone(),
            tracker: self.tracker.clone(),
            signals: self.signals.clone(),
            allow_private_egress: self.allow_private_egress,
            backoff_ms: self.backoff_ms.clone(),
        };

        let handle = tokio::spawn(async move {
            worker.run(child_cancel).await;
        });

        let mut guard = self.active.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(prev) = guard.insert(dp_id, ActiveRelay { cancel, handle }) {
            prev.cancel.cancel();
        }
        Ok(true)
    }

    /// Cancels the local relay task for `dataplane_id`, awaits clean drain up to 5 s, and
    /// releases its distributed lease (`Suspend` / `Terminate`).
    pub async fn stop(&self, dataplane_id: &str) {
        let entry = {
            let mut guard = self.active.lock().unwrap_or_else(|e| e.into_inner());
            guard.remove(dataplane_id)
        };
        if let Some(active) = entry {
            active.cancel.cancel();
            if tokio::time::timeout(STOP_JOIN_TIMEOUT, active.handle)
                .await
                .is_err()
            {
                warn!(
                    dataplane_id,
                    "Timed out waiting for KafkaRelayWorker to drain on stop"
                );
            }
        }
        self.lease.release(dataplane_id).await;
    }

    /// Cancels and drains all active relay tasks on agent shutdown.
    pub async fn shutdown_all(&self) {
        let entries: Vec<(String, ActiveRelay)> = {
            let mut guard = self.active.lock().unwrap_or_else(|e| e.into_inner());
            guard.drain().collect()
        };
        for (_, entry) in &entries {
            entry.cancel.cancel();
        }
        for (dp_id, entry) in entries {
            let _ = tokio::time::timeout(STOP_JOIN_TIMEOUT, entry.handle).await;
            self.lease.release(&dp_id).await;
        }
    }
}
