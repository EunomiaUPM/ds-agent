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

//! Kafka consumer abstraction and `librdkafka` (`rdkafka`) implementation with preflight
//! metadata verification, manual offset commit, allowlisted client properties, and SASL
//! (`PLAIN`, `SCRAM-SHA-256`, `SCRAM-SHA-512`, `OAUTHBEARER`) authentication.

use std::collections::HashMap;
use std::time::Duration;

use connector::entities::parameters::{TemplateMapString, TemplateVecString};
use connector::{
    KafkaOffsetReset, KafkaSaslMechanism, KafkaSecurityProtocol, KafkaSpec,
    ALLOWED_KAFKA_CLIENT_PROPERTIES,
};
use rdkafka::client::{ClientContext, OAuthToken};
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{
    BaseConsumer, CommitMode, Consumer, ConsumerContext, DefaultConsumerContext, StreamConsumer,
};
use rdkafka::message::Message;
use rdkafka::topic_partition_list::{Offset, TopicPartitionList};
use ymir::errors::Outcome;

use crate::engine::dataplane_manager::dataplane_runtime::ResolvedAuthCredentials;
use crate::errors::DataplaneError;

/// Timeout applied to broker metadata preflight during `TransferStart`.
const PREFLIGHT_TIMEOUT: Duration = Duration::from_secs(5);

/// Concrete, fully resolved consumer configuration derived from `KafkaSpec` + `DataplaneRuntime`.
#[derive(Debug, Clone)]
pub struct KafkaConsumerConfig {
    pub brokers: Vec<String>,
    pub topic: String,
    pub group_id: String,
    pub security_protocol: KafkaSecurityProtocol,
    pub sasl_mechanism: Option<KafkaSaslMechanism>,
    pub auto_offset_reset: KafkaOffsetReset,
    pub client_properties: HashMap<String, String>,
    pub auth: ResolvedAuthCredentials,
}

impl KafkaConsumerConfig {
    /// Builds a resolved [`KafkaConsumerConfig`] from a resolved [`KafkaSpec`], falling back to
    /// deterministic consumer group `ds-{dataplane_id}` when `groupId` is omitted.
    pub fn from_spec(
        spec: &KafkaSpec,
        dataplane_id: &str,
        auth: ResolvedAuthCredentials,
    ) -> Outcome<Self> {
        spec.validate_resolved()?;

        let brokers = match &spec.brokers {
            TemplateVecString::Value(list) if !list.is_empty() => list.clone(),
            _ => {
                return Err(DataplaneError::MissingTransferContext {
                    detail: "resolved Kafka brokers list".to_string(),
                }
                .into())
            }
        };

        let group_id = spec
            .group_id
            .as_ref()
            .map(|g| g.trim().to_string())
            .filter(|g| !g.is_empty())
            .unwrap_or_else(|| {
                let tail = dataplane_id.rsplit(':').next().unwrap_or(dataplane_id);
                format!("ds-{tail}")
            });

        let mut client_properties = HashMap::new();
        if let Some(TemplateMapString::Value(props)) = &spec.client_properties {
            for (k, v) in props {
                if ALLOWED_KAFKA_CLIENT_PROPERTIES.contains(&k.as_str()) {
                    client_properties.insert(k.clone(), v.clone());
                } else {
                    return Err(DataplaneError::UnsupportedProtocol {
                        protocol: format!("disallowed Kafka clientProperty '{k}'"),
                    }
                    .into());
                }
            }
        }

        Ok(Self {
            brokers,
            topic: spec.topic.clone(),
            group_id,
            security_protocol: spec.security_protocol,
            sasl_mechanism: spec.sasl_mechanism,
            auto_offset_reset: spec.auto_offset_reset,
            client_properties,
            auth,
        })
    }

    /// Formats `brokers` as a comma-separated `bootstrap.servers` list.
    pub fn bootstrap_servers(&self) -> String {
        self.brokers.join(",")
    }
}

/// Single Kafka record consumed by a relay session.
#[derive(Debug, Clone)]
pub struct KafkaRecord {
    pub topic: String,
    pub partition: i32,
    pub offset: i64,
    pub timestamp_ms: Option<i64>,
    pub key: Option<Vec<u8>>,
    pub payload: Vec<u8>,
}

/// Active consumer stream for a single `KafkaRelayWorker`.
#[async_trait::async_trait]
pub trait KafkaConsumerSession: Send + Sync + 'static {
    /// Awaits the next message from the subscribed topic.
    async fn recv(&mut self) -> Result<KafkaRecord, String>;

    /// Commits `offset + 1` for `(topic, partition)` after `2xx` sink acknowledgement.
    async fn commit(&mut self, topic: &str, partition: i32, offset: i64) -> Result<(), String>;
}

/// Factory creating broker preflight checks and streaming consumer sessions.
#[async_trait::async_trait]
pub trait KafkaConsumerFactory: Send + Sync + 'static {
    /// Verifies broker reachability, SASL handshake, and topic metadata within 5 s before
    /// transitioning the dataplane transfer to `STARTED`.
    async fn preflight(&self, cfg: &KafkaConsumerConfig) -> Outcome<()>;

    /// Opens a subscribed consumer session with `enable.auto.commit=false`.
    async fn create_session(
        &self,
        cfg: &KafkaConsumerConfig,
    ) -> Outcome<Box<dyn KafkaConsumerSession>>;
}

/// `librdkafka` `ClientContext` supplying `OAUTHBEARER` tokens from `BearerToken` / `OAuth2`.
struct OAuthBearerConsumerContext {
    token: String,
    expires_at_ms: Option<i64>,
}

impl ClientContext for OAuthBearerConsumerContext {
    const ENABLE_REFRESH_OAUTH_TOKEN: bool = true;

    fn generate_oauth_token(
        &self,
        _oauthbearer_config: Option<&str>,
    ) -> Result<OAuthToken, Box<dyn std::error::Error>> {
        let lifetime_ms = self
            .expires_at_ms
            .unwrap_or_else(|| chrono::Utc::now().timestamp_millis().saturating_add(3_600_000));
        Ok(OAuthToken {
            token: self.token.clone(),
            principal_name: "eunomia-dataplane".to_string(),
            lifetime_ms,
        })
    }
}

impl ConsumerContext for OAuthBearerConsumerContext {}

/// Production [`KafkaConsumerFactory`] backed by `rdkafka` (`librdkafka`).
#[derive(Debug, Default, Clone)]
pub struct RdkafkaConsumerFactory;

impl RdkafkaConsumerFactory {
    pub fn new() -> Self {
        Self
    }

    fn build_client_config(cfg: &KafkaConsumerConfig) -> Outcome<(ClientConfig, Option<(String, Option<i64>)>)> {
        let mut client_cfg = ClientConfig::new();
        client_cfg
            .set("bootstrap.servers", cfg.bootstrap_servers())
            .set("group.id", &cfg.group_id)
            .set("enable.auto.commit", "false")
            .set("enable.auto.offset.store", "false")
            .set("auto.offset.reset", cfg.auto_offset_reset.as_rdkafka_str())
            .set("security.protocol", cfg.security_protocol.as_rdkafka_str());

        if let Some(mech) = cfg.sasl_mechanism {
            client_cfg.set("sasl.mechanism", mech.as_rdkafka_str());
        }

        for (k, v) in &cfg.client_properties {
            if ALLOWED_KAFKA_CLIENT_PROPERTIES.contains(&k.as_str()) {
                client_cfg.set(k, v);
            }
        }

        let mut oauth_token: Option<(String, Option<i64>)> = None;
        match &cfg.auth {
            ResolvedAuthCredentials::NoAuth => {}
            ResolvedAuthCredentials::BasicAuth { username, password } => {
                client_cfg
                    .set("sasl.username", username)
                    .set("sasl.password", password);
            }
            ResolvedAuthCredentials::BearerToken { token } => {
                oauth_token = Some((token.clone(), None));
            }
            ResolvedAuthCredentials::OAuth2 {
                access_token,
                expires_at,
                ..
            } => {
                let exp_ms = expires_at.map(|secs| (secs as i64).saturating_mul(1000));
                oauth_token = Some((access_token.clone(), exp_ms));
            }
            ResolvedAuthCredentials::ApiKey { .. } => {
                return Err(DataplaneError::UnsupportedProtocol {
                    protocol: "API_KEY authentication is not supported for KAFKA".to_string(),
                }
                .into());
            }
        }

        Ok((client_cfg, oauth_token))
    }
}

enum RdkafkaStreamSession {
    Standard(StreamConsumer<DefaultConsumerContext>),
    OAuth(StreamConsumer<OAuthBearerConsumerContext>),
}

#[async_trait::async_trait]
impl KafkaConsumerSession for RdkafkaStreamSession {
    async fn recv(&mut self) -> Result<KafkaRecord, String> {
        let extract = |msg: &rdkafka::message::BorrowedMessage<'_>| KafkaRecord {
            topic: msg.topic().to_string(),
            partition: msg.partition(),
            offset: msg.offset(),
            timestamp_ms: msg.timestamp().to_millis(),
            key: msg.key().map(|k| k.to_vec()),
            payload: msg.payload().map(|p| p.to_vec()).unwrap_or_default(),
        };

        match self {
            Self::Standard(c) => c.recv().await.map(|m| extract(&m)).map_err(|e| e.to_string()),
            Self::OAuth(c) => c.recv().await.map(|m| extract(&m)).map_err(|e| e.to_string()),
        }
    }

    async fn commit(&mut self, topic: &str, partition: i32, offset: i64) -> Result<(), String> {
        let mut tpl = TopicPartitionList::new();
        tpl.add_partition_offset(topic, partition, Offset::Offset(offset + 1))
            .map_err(|e| e.to_string())?;
        match self {
            Self::Standard(c) => c
                .commit(&tpl, CommitMode::Async)
                .map_err(|e| e.to_string()),
            Self::OAuth(c) => c
                .commit(&tpl, CommitMode::Async)
                .map_err(|e| e.to_string()),
        }
    }
}

#[async_trait::async_trait]
impl KafkaConsumerFactory for RdkafkaConsumerFactory {
    async fn preflight(&self, cfg: &KafkaConsumerConfig) -> Outcome<()> {
        let (client_cfg, oauth) = Self::build_client_config(cfg)?;
        let topic = cfg.topic.clone();
        let brokers_label = cfg.bootstrap_servers();

        tokio::task::spawn_blocking(move || -> Result<(), String> {
            if let Some((token, expires_at_ms)) = oauth {
                let ctx = OAuthBearerConsumerContext {
                    token,
                    expires_at_ms,
                };
                let consumer: BaseConsumer<OAuthBearerConsumerContext> = client_cfg
                    .create_with_context(ctx)
                    .map_err(|e| format!("failed to build Kafka client: {e}"))?;
                let md = consumer
                    .fetch_metadata(Some(&topic), PREFLIGHT_TIMEOUT)
                    .map_err(|e| format!("broker metadata fetch failed: {e}"))?;
                if let Some(t) = md.topics().iter().find(|t| t.name() == topic) {
                    if let Some(err) = t.error() {
                        return Err(format!("topic '{topic}' metadata error: {err:?}"));
                    }
                }
            } else {
                let consumer: BaseConsumer = client_cfg
                    .create()
                    .map_err(|e| format!("failed to build Kafka client: {e}"))?;
                let md = consumer
                    .fetch_metadata(Some(&topic), PREFLIGHT_TIMEOUT)
                    .map_err(|e| format!("broker metadata fetch failed: {e}"))?;
                if let Some(t) = md.topics().iter().find(|t| t.name() == topic) {
                    if let Some(err) = t.error() {
                        return Err(format!("topic '{topic}' metadata error: {err:?}"));
                    }
                }
            }
            Ok(())
        })
        .await
        .map_err(|e| DataplaneError::PubSubRequestFailed {
            method: "KAFKA_METADATA".to_string(),
            url: brokers_label.clone(),
            reason: format!("preflight join error: {e}"),
        })?
        .map_err(|reason| {
            DataplaneError::PubSubRequestFailed {
                method: "KAFKA_METADATA".to_string(),
                url: brokers_label,
                reason,
            }
            .into()
        })
    }

    async fn create_session(
        &self,
        cfg: &KafkaConsumerConfig,
    ) -> Outcome<Box<dyn KafkaConsumerSession>> {
        let (client_cfg, oauth) = Self::build_client_config(cfg)?;
        let brokers_label = cfg.bootstrap_servers();

        let session = if let Some((token, expires_at_ms)) = oauth {
            let ctx = OAuthBearerConsumerContext {
                token,
                expires_at_ms,
            };
            let consumer: StreamConsumer<OAuthBearerConsumerContext> =
                client_cfg.create_with_context(ctx).map_err(|e| {
                    DataplaneError::PubSubRequestFailed {
                        method: "KAFKA_SUBSCRIBE".to_string(),
                        url: brokers_label.clone(),
                        reason: e.to_string(),
                    }
                })?;
            consumer
                .subscribe(&[&cfg.topic])
                .map_err(|e| DataplaneError::PubSubRequestFailed {
                    method: "KAFKA_SUBSCRIBE".to_string(),
                    url: brokers_label,
                    reason: e.to_string(),
                })?;
            RdkafkaStreamSession::OAuth(consumer)
        } else {
            let consumer: StreamConsumer =
                client_cfg
                    .create()
                    .map_err(|e| DataplaneError::PubSubRequestFailed {
                        method: "KAFKA_SUBSCRIBE".to_string(),
                        url: brokers_label.clone(),
                        reason: e.to_string(),
                    })?;
            consumer
                .subscribe(&[&cfg.topic])
                .map_err(|e| DataplaneError::PubSubRequestFailed {
                    method: "KAFKA_SUBSCRIBE".to_string(),
                    url: brokers_label,
                    reason: e.to_string(),
                })?;
            RdkafkaStreamSession::Standard(consumer)
        };

        Ok(Box::new(session))
    }
}
