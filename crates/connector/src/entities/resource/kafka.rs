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

//! Kafka resource specification, security/SASL enums, and static/resolved validation.

use crate::entities::auth_config::AuthenticationConfig;
use crate::entities::parameters::{TemplateMapString, TemplateString};
use crate::TemplateVecString;
use serde::{Deserialize, Serialize};
use ymir::errors::{Errors, Outcome};

/// Safe allowlist of optional `librdkafka` consumer properties permitted in `KafkaSpec.clientProperties`.
/// Credentials and dynamic library hooks (`plugin.library.paths`, `sasl.*`) are strictly forbidden here.
pub const ALLOWED_KAFKA_CLIENT_PROPERTIES: &[&str] = &[
    "session.timeout.ms",
    "max.poll.interval.ms",
    "fetch.min.bytes",
    "fetch.max.bytes",
    "ssl.ca.location",
];

/// Transport security protocol between the dataplane consumer and the Kafka cluster.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum KafkaSecurityProtocol {
    #[default]
    Plaintext,
    Ssl,
    SaslPlaintext,
    SaslSsl,
}

impl KafkaSecurityProtocol {
    pub fn is_sasl(self) -> bool {
        matches!(self, Self::SaslPlaintext | Self::SaslSsl)
    }

    pub fn as_rdkafka_str(self) -> &'static str {
        match self {
            Self::Plaintext => "PLAINTEXT",
            Self::Ssl => "SSL",
            Self::SaslPlaintext => "SASL_PLAINTEXT",
            Self::SaslSsl => "SASL_SSL",
        }
    }
}

/// SASL authentication mechanism paired with `SASL_PLAINTEXT` or `SASL_SSL`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KafkaSaslMechanism {
    #[serde(rename = "PLAIN")]
    Plain,
    #[serde(rename = "SCRAM_SHA_256")]
    ScramSha256,
    #[serde(rename = "SCRAM_SHA_512")]
    ScramSha512,
    #[serde(rename = "OAUTHBEARER")]
    Oauthbearer,
}

impl KafkaSaslMechanism {
    pub fn as_rdkafka_str(self) -> &'static str {
        match self {
            Self::Plain => "PLAIN",
            Self::ScramSha256 => "SCRAM-SHA-256",
            Self::ScramSha512 => "SCRAM-SHA-512",
            Self::Oauthbearer => "OAUTHBEARER",
        }
    }
}

/// Starting offset policy when a transfer's consumer group has no committed offset yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum KafkaOffsetReset {
    /// Stream new records published after `TransferStart` (default).
    #[default]
    Latest,
    /// Replay retained records from the beginning of the topic.
    Earliest,
}

impl KafkaOffsetReset {
    pub fn as_rdkafka_str(self) -> &'static str {
        match self {
            Self::Latest => "latest",
            Self::Earliest => "earliest",
        }
    }
}

/// Kafka protocol specification for provider `PUSH` streaming relays (`Kafka -> HTTP sink`).
///
/// All string, list, and map fields support `{{__PARAM__}}` and `{{__RUNTIME_*__}}` placeholders.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KafkaSpec {
    /// Broker bootstrap addresses (e.g. `["broker-1:9092"]` or `"{{__BROKERS__}}"`).
    pub brokers: TemplateVecString,
    /// Source topic name (supports `{{__TOPIC__}}`).
    pub topic: TemplateString,
    /// Optional consumer group ID; defaults at runtime to `ds-{dataplane_id}` per transfer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_id: Option<TemplateString>,
    /// Wire security mode (`PLAINTEXT` by default).
    #[serde(default)]
    pub security_protocol: KafkaSecurityProtocol,
    /// SASL mechanism (required if and only if `securityProtocol` is `SASL_*`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sasl_mechanism: Option<KafkaSaslMechanism>,
    /// Initial offset behaviour (`LATEST` by default).
    #[serde(default)]
    pub auto_offset_reset: KafkaOffsetReset,
    /// Optional allowlisted `librdkafka` tuning properties.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_properties: Option<TemplateMapString>,
}

impl Default for KafkaSpec {
    fn default() -> Self {
        Self {
            brokers: TemplateVecString::Value(Vec::new()),
            topic: String::new(),
            group_id: None,
            security_protocol: KafkaSecurityProtocol::Plaintext,
            sasl_mechanism: None,
            auto_offset_reset: KafkaOffsetReset::Latest,
            client_properties: None,
        }
    }
}

impl KafkaSpec {
    /// Validates template-time compatibility between `KafkaSpec` security settings, allowlisted
    /// `clientProperties`, and the enclosing `AuthenticationConfig`.
    pub fn validate_with_auth(&self, auth: &AuthenticationConfig) -> Outcome<()> {
        if self.security_protocol.is_sasl() && self.sasl_mechanism.is_none() {
            return Err(Errors::validation(
                format!(
                    "KAFKA securityProtocol {:?} requires saslMechanism to be set",
                    self.security_protocol
                ),
                None,
            ));
        }
        if !self.security_protocol.is_sasl() && self.sasl_mechanism.is_some() {
            return Err(Errors::validation(
                format!(
                    "KAFKA saslMechanism must be omitted when securityProtocol is {:?}",
                    self.security_protocol
                ),
                None,
            ));
        }

        match auth {
            AuthenticationConfig::NoAuth => {
                if self.security_protocol.is_sasl() {
                    return Err(Errors::validation(
                        "KAFKA with NO_AUTH requires securityProtocol PLAINTEXT or SSL",
                        None,
                    ));
                }
            }
            AuthenticationConfig::BasicAuth(_) => {
                if !self.security_protocol.is_sasl()
                    || !matches!(
                        self.sasl_mechanism,
                        Some(
                            KafkaSaslMechanism::Plain
                                | KafkaSaslMechanism::ScramSha256
                                | KafkaSaslMechanism::ScramSha512
                        )
                    )
                {
                    return Err(Errors::validation(
                        "KAFKA with BASIC_AUTH requires SASL_PLAINTEXT or SASL_SSL and saslMechanism PLAIN, SCRAM_SHA_256, or SCRAM_SHA_512",
                        None,
                    ));
                }
            }
            AuthenticationConfig::BearerToken { .. } | AuthenticationConfig::OAuth2 { .. } => {
                if !self.security_protocol.is_sasl()
                    || self.sasl_mechanism != Some(KafkaSaslMechanism::Oauthbearer)
                {
                    return Err(Errors::validation(
                        "KAFKA with BEARER_TOKEN or OAUTH2 requires SASL_PLAINTEXT or SASL_SSL and saslMechanism OAUTHBEARER",
                        None,
                    ));
                }
            }
            AuthenticationConfig::ApiKey { .. } => {
                return Err(Errors::validation(
                    "API_KEY authentication is not supported for KAFKA protocol; use NO_AUTH, BASIC_AUTH, BEARER_TOKEN, or OAUTH2",
                    None,
                ));
            }
        }

        if let Some(TemplateMapString::Value(props)) = &self.client_properties {
            for key in props.keys() {
                if key != "__EXTRA__" && !ALLOWED_KAFKA_CLIENT_PROPERTIES.contains(&key.as_str()) {
                    return Err(Errors::validation(
                        format!(
                            "KAFKA clientProperties key '{key}' is not allowed; permitted keys: {}",
                            ALLOWED_KAFKA_CLIENT_PROPERTIES.join(", ")
                        ),
                        None,
                    ));
                }
            }
        }

        Ok(())
    }

    /// Validates concrete broker addresses, topic naming rules, and `clientProperties` after
    /// instance-time placeholders have been substituted.
    pub fn validate_resolved(&self) -> Outcome<()> {
        match &self.brokers {
            TemplateVecString::Value(brokers) if !brokers.is_empty() => {
                for broker in brokers {
                    if !Self::has_runtime_placeholder(broker) && !Self::is_valid_host_port(broker) {
                        return Err(Errors::validation(
                            format!(
                                "invalid KAFKA broker address '{broker}'; expected 'host:port'"
                            ),
                            None,
                        ));
                    }
                }
            }
            TemplateVecString::Template(raw) if Self::has_runtime_placeholder(raw) => {}
            _ => {
                return Err(Errors::validation(
                    "KAFKA brokers list must resolve to at least one 'host:port' entry",
                    None,
                ));
            }
        }

        if !Self::has_runtime_placeholder(&self.topic) && !Self::is_valid_topic(&self.topic) {
            return Err(Errors::validation(
                format!(
                    "invalid KAFKA topic '{}'; must be 1..=249 chars of [a-zA-Z0-9._-] and not '.' or '..'",
                    self.topic
                ),
                None,
            ));
        }

        if let Some(group_id) = &self.group_id {
            if !Self::has_runtime_placeholder(group_id) && group_id.trim().is_empty() {
                return Err(Errors::validation(
                    "KAFKA groupId cannot be empty when specified",
                    None,
                ));
            }
        }

        match &self.client_properties {
            None => {}
            Some(TemplateMapString::Value(props)) => {
                for key in props.keys() {
                    if !ALLOWED_KAFKA_CLIENT_PROPERTIES.contains(&key.as_str()) {
                        return Err(Errors::validation(
                            format!(
                                "KAFKA clientProperties key '{key}' is not allowed; permitted keys: {}",
                                ALLOWED_KAFKA_CLIENT_PROPERTIES.join(", ")
                            ),
                            None,
                        ));
                    }
                }
            }
            Some(TemplateMapString::Template(raw)) if Self::has_runtime_placeholder(raw) => {}
            Some(TemplateMapString::Template(raw)) => {
                return Err(Errors::validation(
                    format!("unresolved KAFKA clientProperties placeholder '{raw}'"),
                    None,
                ));
            }
        }

        Ok(())
    }

    fn has_runtime_placeholder(s: &str) -> bool {
        s.contains("{{__RUNTIME_")
    }

    fn is_valid_host_port(s: &str) -> bool {
        let trimmed = s.trim();
        let Some((host, port_str)) = trimmed.rsplit_once(':') else {
            return false;
        };
        let host = host.trim_start_matches('[').trim_end_matches(']');
        if host.is_empty() || host.chars().any(char::is_whitespace) {
            return false;
        }
        matches!(port_str.parse::<u16>(), Ok(p) if p >= 1)
    }

    fn is_valid_topic(topic: &str) -> bool {
        if topic.is_empty() || topic.len() > 249 || topic == "." || topic == ".." {
            return false;
        }
        topic
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
    }
}
