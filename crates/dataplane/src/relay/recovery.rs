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

//! Background reconciler (`KafkaRecoveryWorker`) ensuring active `Provider + Push + Started`
//! Kafka transfers automatically resume after agent restarts or horizontal replica failovers
//! within `<= 15 s`.

use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use connector::{
    ConnectorInstanceFacadeTrait, InteractionConfig, KeystoreLookup, ProtocolSpec,
    RuntimeParametersResolver,
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, warn};
use urn::Urn;
use ymir::boot::workers::BackgroundWorker;
use ymir::errors::Outcome;
use ymir::oauth::{OwnedTrait, OwnerScope};
use ymir::query::{Page, Sort};

use crate::data::factory_trait::DataplaneRepoTrait;
use crate::data::sea_orm::orm::dataplane_transfers::{
    self, InteractionMode, TransferRole, TransferState,
};
use crate::engine::dataplane_manager::dataplane_proxy::DataplaneProxyEgress;
use crate::engine::dataplane_manager::dataplane_runtime::{DataplaneRuntime, RuntimeSecretVault};
use crate::entities::filters::DataplaneTransferFilter;
use crate::relay::consumer::KafkaConsumerConfig;
use crate::relay::supervisor::KafkaRelaySupervisor;
use crate::relay::worker::KafkaRelayTaskSpec;

const DEFAULT_RECOVERY_SWEEP_INTERVAL: Duration = Duration::from_secs(5);
const RECOVERY_PAGE_SIZE: u32 = 500;

/// Periodically reconciles persisted `STARTED` Kafka push transfers with `KafkaRelaySupervisor`.
pub struct KafkaRecoveryWorker {
    repo: Arc<dyn DataplaneRepoTrait>,
    connector: Arc<dyn ConnectorInstanceFacadeTrait>,
    keystore: Option<Arc<dyn KeystoreLookup>>,
    supervisor: Arc<KafkaRelaySupervisor>,
    sweep_interval: Duration,
}

impl KafkaRecoveryWorker {
    pub fn new(
        repo: Arc<dyn DataplaneRepoTrait>,
        connector: Arc<dyn ConnectorInstanceFacadeTrait>,
        keystore: Option<Arc<dyn KeystoreLookup>>,
        supervisor: Arc<KafkaRelaySupervisor>,
    ) -> Self {
        Self {
            repo,
            connector,
            keystore,
            supervisor,
            sweep_interval: DEFAULT_RECOVERY_SWEEP_INTERVAL,
        }
    }

    pub fn with_sweep_interval(mut self, interval: Duration) -> Self {
        self.sweep_interval = interval;
        self
    }

    /// Runs a single reconciliation pass across all `Provider + Push + Started` transfers.
    pub async fn sweep_once(&self) {
        let filter = DataplaneTransferFilter {
            role: Some(TransferRole::Provider),
            interaction_mode: Some(InteractionMode::Push),
            state: Some(TransferState::Started),
            ..Default::default()
        };
        let page = Page {
            limit: RECOVERY_PAGE_SIZE,
            page: None,
            cursor: None,
        };

        let models = match self
            .repo
            .get_dataplane_transfers_repo()
            .get_all_dataplane_transfers(&OwnerScope::All, &filter, &page, &Sort::default())
            .await
        {
            Ok(rows) => rows,
            Err(e) => {
                warn!(error = %e, "KafkaRecoveryWorker failed to query started transfers");
                return;
            }
        };

        for model in models {
            if self.supervisor.is_running(&model.id) {
                continue;
            }
            self.reconcile_model(model).await;
        }
    }

    async fn reconcile_model(&self, model: dataplane_transfers::Model) {
        let Some(connector_id_str) = model.connector_instance_id.as_deref() else {
            return;
        };
        let Ok(connector_urn) = Urn::from_str(connector_id_str) else {
            return;
        };
        let Ok(Some(connector)) = self.connector.get_instance_by_id(&connector_urn).await else {
            return;
        };

        // Fast check before resolving runtime secrets: skip non-Kafka push transfers.
        match &connector.interaction {
            InteractionConfig::Push(lc) if matches!(lc.subscribe, ProtocolSpec::Kafka(_)) => {}
            _ => return,
        }

        let Ok(transfer_process_id) = Urn::from_str(&model.transfer_process_id) else {
            return;
        };
        let Ok(egress) = serde_json::from_value::<DataplaneProxyEgress>(model.egress_config.clone())
        else {
            return;
        };

        let raw_runtime = model
            .flow_control
            .as_ref()
            .and_then(|v| serde_json::from_value::<DataplaneRuntime>(v.clone()).ok())
            .unwrap_or_default();
        let resolved_rt = match &self.keystore {
            Some(ks) => {
                RuntimeSecretVault::resolve_with_lookup(raw_runtime, ks, &connector.user_id).await
            }
            None => raw_runtime,
        };

        let Ok(rt_value) = serde_json::to_value(&resolved_rt) else {
            return;
        };
        let mut resolver = RuntimeParametersResolver::new(&connector, &rt_value);
        if let Some(ks) = &self.keystore {
            resolver = resolver.with_keystore(ks.clone(), &connector.user_id);
        }
        let Ok(resolved_connector) = resolver.resolve().await else {
            return;
        };

        let InteractionConfig::Push(push_lc) = &resolved_connector.interaction else {
            return;
        };
        let ProtocolSpec::Kafka(kafka_spec) = &push_lc.subscribe else {
            return;
        };

        let Ok(consumer_config) =
            KafkaConsumerConfig::from_spec(kafka_spec, &model.id, resolved_rt.auth.clone())
        else {
            return;
        };

        let spec = KafkaRelayTaskSpec {
            dataplane_id: model.id.clone(),
            transfer_process_id,
            owner: model.owner(),
            consumer_config,
            egress,
            delivery_policy: resolved_rt.delivery_policy,
        };

        match self.supervisor.ensure_running(spec).await {
            Ok(true) => {
                debug!(
                    dataplane_id = %model.id,
                    "KafkaRecoveryWorker resumed Kafka relay worker"
                );
            }
            Ok(false) => {}
            Err(e) => {
                warn!(
                    dataplane_id = %model.id,
                    error = %e,
                    "KafkaRecoveryWorker failed to resume Kafka relay worker"
                );
            }
        }
    }
}

#[async_trait::async_trait]
impl BackgroundWorker for KafkaRecoveryWorker {
    fn name(&self) -> &'static str {
        "dataplane-kafka-recovery"
    }

    async fn run(self: Box<Self>, cancel_token: CancellationToken) -> Outcome<()> {
        let mut ticker = tokio::time::interval(self.sweep_interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    self.supervisor.shutdown_all().await;
                    break;
                }
                _ = ticker.tick() => {
                    self.sweep_once().await;
                }
            }
        }
        Ok(())
    }
}
