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

//! `DriverPubSubTrait` implementation for `ProtocolSpec::Kafka` (`KafkaPubSubscriber`).
//!
//! - `subscribe`: resolves runtime secrets & placeholders, verifies broker + topic metadata via a
//!   5 s preflight check, starts the supervised `KafkaRelayWorker`, and records subscription
//!   metadata in `DataplaneRuntime`.
//! - `unsubscribe`: cancels the supervised worker, drains any in-flight HTTP sink delivery /
//!   offset commit, and releases the distributed lease (`Suspend` / `Terminate`).

use std::str::FromStr;
use std::sync::Arc;

use connector::{
    InteractionConfig, KeystoreLookup, ProtocolSpec, RuntimeParametersResolver,
};
use serde_json::json;
use urn::Urn;
use ymir::errors::Outcome;
use ymir::oauth::OwnedTrait;

use crate::engine::dataplane_drivers::DriverPubSubTrait;
use crate::engine::dataplane_manager::dataplane_context::DataplaneContext;
use crate::engine::dataplane_manager::dataplane_proxy::DataplaneProxyEgress;
use crate::engine::dataplane_manager::dataplane_runtime::{
    DataplaneRuntime, DeliveryPolicy, RuntimeSecretVault,
};
use crate::errors::DataplaneError;
use crate::relay::{KafkaConsumerConfig, KafkaRelaySupervisor, KafkaRelayTaskSpec};

pub struct KafkaPubSubscriber {
    supervisor: Arc<KafkaRelaySupervisor>,
    keystore: Option<Arc<dyn KeystoreLookup>>,
}

impl std::fmt::Debug for KafkaPubSubscriber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KafkaPubSubscriber")
            .field("keystore", &self.keystore.is_some())
            .finish()
    }
}

impl KafkaPubSubscriber {
    pub fn new(
        supervisor: Arc<KafkaRelaySupervisor>,
        keystore: Option<Arc<dyn KeystoreLookup>>,
    ) -> Self {
        Self {
            supervisor,
            keystore,
        }
    }

    async fn resolved_runtime(
        &self,
        context: &DataplaneContext,
        user_id: &str,
    ) -> DataplaneRuntime {
        let rt = context.runtime().cloned().unwrap_or_default();
        match &self.keystore {
            Some(ks) => RuntimeSecretVault::resolve_with_lookup(rt, ks, user_id).await,
            None => rt,
        }
    }

    fn resolve_egress(context: &DataplaneContext) -> Outcome<DataplaneProxyEgress> {
        if let Some(proxy) = context.proxy() {
            return Ok(proxy.egress().clone());
        }
        if let Ok(egress) = serde_json::from_value::<DataplaneProxyEgress>(
            context.dataplane_process().inner.egress_config.clone(),
        ) {
            if !matches!(egress, DataplaneProxyEgress::NoOp) {
                return Ok(egress);
            }
        }
        if let Some(addr) = context.forward_dataplane_address() {
            return Ok(DataplaneProxyEgress::HttpProxy {
                path: addr.endpoint.clone(),
                token_type: addr.authorization_type.clone(),
                token: addr.authorization.clone(),
            });
        }
        Err(DataplaneError::MissingTransferContext {
            detail: "consumer HTTP sink egress address".to_string(),
        }
        .into())
    }
}

#[async_trait::async_trait]
impl DriverPubSubTrait for KafkaPubSubscriber {
    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "kafka-broker"))]
    async fn subscribe(&self, context: &DataplaneContext) -> Outcome<DataplaneContext> {
        let connector = context.connector_instance().ok_or_else(|| {
            DataplaneError::PubSubConnectorNotAvailable {
                operation: "subscribe".to_string(),
            }
        })?;

        let dp_proc = context.dataplane_process();
        let dataplane_id = dp_proc.inner.id.clone();
        let transfer_process_id = Urn::from_str(&dp_proc.inner.transfer_process_id)?;

        let resolved_rt = self.resolved_runtime(context, &connector.user_id).await;
        let runtime_value = serde_json::to_value(&resolved_rt)?;
        let mut resolver = RuntimeParametersResolver::new(connector, &runtime_value);
        if let Some(ks) = &self.keystore {
            resolver = resolver.with_keystore(ks.clone(), &connector.user_id);
        }
        let resolved_connector = resolver.resolve().await?;

        let push_lifecycle = match &resolved_connector.interaction {
            InteractionConfig::Push(p) => p,
            _ => {
                return Err(DataplaneError::WrongInteractionType {
                    expected: "Push".to_string(),
                    found: "other".to_string(),
                }
                .into())
            }
        };

        let kafka_spec = match &push_lifecycle.subscribe {
            ProtocolSpec::Kafka(spec) => spec,
            _ => {
                return Err(DataplaneError::UnsupportedProtocol {
                    protocol: "non-KAFKA subscribe".to_string(),
                }
                .into())
            }
        };

        let consumer_config = KafkaConsumerConfig::from_spec(
            kafka_spec,
            &dataplane_id,
            resolved_rt.auth.clone(),
        )?;
        let egress = Self::resolve_egress(context)?;

        // Fail fast before transitioning to STARTED if broker or topic metadata is unreachable.
        self.supervisor.preflight(&consumer_config).await?;

        let delivery_policy = match context.runtime() {
            Some(rt) => rt.delivery_policy,
            None => DeliveryPolicy {
                lifetime: push_lifecycle.lifetime,
                max_retries: push_lifecycle.max_retries.unwrap_or(3),
            },
        };

        let subscription_meta = json!({
            "protocol": "KAFKA",
            "brokers": consumer_config.brokers,
            "topic": consumer_config.topic,
            "groupId": consumer_config.group_id,
            "securityProtocol": consumer_config.security_protocol,
            "autoOffsetReset": consumer_config.auto_offset_reset,
        });

        let spec = KafkaRelayTaskSpec {
            dataplane_id,
            transfer_process_id,
            owner: dp_proc.inner.owner(),
            consumer_config,
            egress,
            delivery_policy,
        };
        self.supervisor.ensure_running(spec).await?;

        let mut ctx = context.clone();
        let mut runtime = ctx.runtime().cloned().unwrap_or_default();
        runtime.subscription = subscription_meta;
        ctx.set_runtime(runtime);
        Ok(ctx)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "kafka-broker"))]
    async fn unsubscribe(&self, context: &DataplaneContext) -> Outcome<DataplaneContext> {
        let dataplane_id = &context.dataplane_process().inner.id;
        self.supervisor.stop(dataplane_id).await;

        let mut ctx = context.clone();
        let mut runtime = ctx.runtime().cloned().unwrap_or_default();
        runtime.unsubscription = json!({
            "protocol": "KAFKA",
            "stopped": true,
        });
        ctx.set_runtime(runtime);
        Ok(ctx)
    }
}
