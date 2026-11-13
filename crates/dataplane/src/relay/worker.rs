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

//! Per-transfer supervised `KafkaRelayWorker`: consumes records from a Kafka topic with
//! manual offset commits (`enable.auto.commit=false`), validates the Consumer `DataAddress`
//! sink against SSRF rules, delivers payloads via HTTP `POST` with `X-Kafka-*` headers and the
//! Consumer capability token, logs `TransferEvent` rows, and integrates with `DeliveryPolicy`
//! (`FINITE` auto-completion and `maxRetries` failure signaling).

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::sync::Arc;
use std::time::Duration;

use axum::body::Bytes;
use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use base64::{engine::general_purpose::STANDARD, Engine};
use connector::TransferLifetime;
use hyper::Method;
use serde_json::json;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};
use url::Host;
use urn::Urn;
use ymir::oauth::Owner;
use ymir::services::client::ClientService;

use crate::data::factory_trait::DataplaneRepoTrait;
use crate::data::sea_orm::orm::transfer_event::{LogLevel, NewTransferEvent};
use crate::engine::dataplane_drivers::proxy::http::HttpProxyDriver;
use crate::engine::dataplane_manager::dataplane_proxy::DataplaneProxyEgress;
use crate::engine::dataplane_manager::dataplane_runtime::{DeliveryPolicy, ResolvedAuthCredentials};
use crate::proxy::DeliveryTracker;
use crate::relay::consumer::{KafkaConsumerConfig, KafkaConsumerSession, KafkaRecord};
use crate::relay::lease::KafkaLeaseManager;
use crate::signals::{DataplaneSignal, DataplaneSignalSender};

/// Default exponential backoff schedule (ms) applied between transient sink failures.
pub const DEFAULT_KAFKA_RELAY_BACKOFF_MS: &[u64] = &[250, 500, 1000, 2000, 4000];

const RELAY_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const RELAY_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Complete per-transfer parameters needed to launch a `KafkaRelayWorker`.
#[derive(Clone, Debug)]
pub struct KafkaRelayTaskSpec {
    pub dataplane_id: String,
    pub transfer_process_id: Urn,
    pub owner: Owner,
    pub consumer_config: KafkaConsumerConfig,
    pub egress: DataplaneProxyEgress,
    pub delivery_policy: DeliveryPolicy,
}

enum SinkTargetError {
    InvalidUrl,
    Forbidden(&'static str),
    ResolutionFailed,
}

enum DeliveryOutcome {
    Delivered(u16),
    PermanentFailure { status: u16, reason: String },
    TransientFailure { status: Option<u16>, reason: String },
}

/// Active per-transfer streaming loop (`Kafka -> HTTP sink`).
pub struct KafkaRelayWorker {
    pub spec: KafkaRelayTaskSpec,
    pub session: Box<dyn KafkaConsumerSession>,
    pub client: Arc<ClientService>,
    pub lease: Arc<KafkaLeaseManager>,
    pub repo: Option<Arc<dyn DataplaneRepoTrait>>,
    pub tracker: Arc<DeliveryTracker>,
    pub signals: DataplaneSignalSender,
    pub allow_private_egress: bool,
    pub backoff_ms: Vec<u64>,
}

impl KafkaRelayWorker {
    pub fn build_http_client(accept_invalid_certs: bool) -> ClientService {
        ClientService::builder()
            .accept_invalid_certs(accept_invalid_certs)
            .connect_timeout(RELAY_CONNECT_TIMEOUT)
            .timeout(Some(RELAY_REQUEST_TIMEOUT))
            .build()
    }

    /// Runs the relay loop until cancelled, lease is lost, a `FINITE` single-shot delivery
    /// completes, or an unrecoverable / retry-exhausted error triggers `DataplaneSignal::Failed`.
    pub async fn run(mut self, cancel: CancellationToken) {
        let dp_id = self.spec.dataplane_id.clone();
        let (sink_url, sink_credentials) = match Self::extract_sink(&self.spec.egress) {
            Ok(pair) => pair,
            Err(reason) => {
                self.emit_failed("DATAPLANE_CONFIG_ERROR", reason).await;
                self.lease.release(&dp_id).await;
                return;
            }
        };

        let mut lease_ticker = tokio::time::interval(self.lease.renew_interval());
        lease_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        // Consume the immediate tick — the lease was just acquired by the supervisor.
        lease_ticker.tick().await;

        info!(
            dataplane_id = %dp_id,
            topic = %self.spec.consumer_config.topic,
            group_id = %self.spec.consumer_config.group_id,
            "KafkaRelayWorker started"
        );

        loop {
            let record = tokio::select! {
                biased;
                _ = cancel.cancelled() => {
                    debug!(dataplane_id = %dp_id, "KafkaRelayWorker cancelled cleanly");
                    break;
                }
                _ = lease_ticker.tick() => {
                    if !self.lease.renew(&dp_id).await {
                        warn!(
                            dataplane_id = %dp_id,
                            "KafkaRelayWorker lost distributed lease; stopping local loop"
                        );
                        return;
                    }
                    continue;
                }
                rec = self.session.recv() => match rec {
                    Ok(r) => r,
                    Err(err) => {
                        let exceeded = self
                            .tracker
                            .record_transient_failure(&dp_id, self.spec.delivery_policy.max_retries)
                            .await;
                        warn!(
                            dataplane_id = %dp_id,
                            error = %err,
                            exceeded,
                            "Kafka consumer recv error"
                        );
                        if exceeded {
                            self.emit_failed(
                                "DATAPLANE_KAFKA_CONSUMER_ERROR",
                                format!("Kafka consumer error exceeded maxRetries: {err}"),
                            )
                            .await;
                            break;
                        }
                        if self.sleep_or_cancel(&cancel, 0).await {
                            break;
                        }
                        continue;
                    }
                }
            };

            if !self
                .deliver_with_retries(&record, &sink_url, &sink_credentials, &cancel, &mut lease_ticker)
                .await
            {
                break;
            }

            if let Err(err) = self
                .session
                .commit(&record.topic, record.partition, record.offset)
                .await
            {
                warn!(
                    dataplane_id = %dp_id,
                    topic = %record.topic,
                    partition = record.partition,
                    offset = record.offset,
                    error = %err,
                    "Failed to commit Kafka offset after 2xx sink delivery"
                );
            }

            if self.spec.delivery_policy.lifetime == TransferLifetime::Finite {
                if self.tracker.complete_finite_slot(&dp_id).await {
                    info!(
                        dataplane_id = %dp_id,
                        transfer_process_id = %self.spec.transfer_process_id,
                        "FINITE Kafka relay delivered record; signaling Completed"
                    );
                    self.signals.emit(DataplaneSignal::Completed {
                        transfer_process_id: self.spec.transfer_process_id.clone(),
                    });
                }
                break;
            }
        }

        self.lease.release(&dp_id).await;
    }

    /// Delivers a single `KafkaRecord` to `sink_url`, retrying transient failures up to
    /// `delivery_policy.max_retries`. Returns `true` when delivered (`2xx`) and `false` when
    /// cancelled, lease-lost, or failed permanently.
    async fn deliver_with_retries(
        &self,
        record: &KafkaRecord,
        sink_url: &str,
        sink_credentials: &ResolvedAuthCredentials,
        cancel: &CancellationToken,
        lease_ticker: &mut tokio::time::Interval,
    ) -> bool {
        let dp_id = &self.spec.dataplane_id;
        let max_retries = self.spec.delivery_policy.max_retries;
        let mut attempt: usize = 0;

        loop {
            match validate_sink_url(sink_url, self.allow_private_egress).await {
                Ok(()) => {}
                Err(SinkTargetError::InvalidUrl) => {
                    self.emit_failed("DATAPLANE_INVALID_TARGET_URL", "invalid consumer sink URL")
                        .await;
                    return false;
                }
                Err(SinkTargetError::Forbidden(reason)) => {
                    self.emit_failed("DATAPLANE_FORBIDDEN_TARGET", reason).await;
                    return false;
                }
                Err(SinkTargetError::ResolutionFailed) => {
                    if self
                        .tracker
                        .record_transient_failure(dp_id, max_retries)
                        .await
                    {
                        self.emit_failed(
                            "DATAPLANE_MAX_RETRIES_EXCEEDED",
                            "unable to resolve consumer sink host",
                        )
                        .await;
                        return false;
                    }
                    if self
                        .backoff_with_lease_renewal(cancel, lease_ticker, attempt)
                        .await
                    {
                        return false;
                    }
                    attempt = attempt.saturating_add(1);
                    continue;
                }
            }

            match self.post_record(record, sink_url, sink_credentials).await {
                DeliveryOutcome::Delivered(status) => {
                    self.tracker.record_success(dp_id).await;
                    self.log_event(
                        LogLevel::Info,
                        format!(
                            "Relayed Kafka {}:{}@{} -> {} ({})",
                            record.topic, record.partition, record.offset, sink_url, status
                        ),
                        record,
                        sink_url,
                        Some(status),
                    )
                    .await;
                    return true;
                }
                DeliveryOutcome::PermanentFailure { status, reason } => {
                    self.log_event(
                        LogLevel::Error,
                        format!("Permanent sink rejection ({status}): {reason}"),
                        record,
                        sink_url,
                        Some(status),
                    )
                    .await;
                    self.emit_failed(
                        "DATAPLANE_UPSTREAM_REJECTED",
                        format!("consumer sink rejected push with HTTP {status}"),
                    )
                    .await;
                    return false;
                }
                DeliveryOutcome::TransientFailure { status, reason } => {
                    let exceeded = self
                        .tracker
                        .record_transient_failure(dp_id, max_retries)
                        .await;
                    warn!(
                        dataplane_id = %dp_id,
                        attempt,
                        max_retries,
                        status = ?status,
                        reason = %reason,
                        exceeded,
                        "Transient failure delivering Kafka record to HTTP sink"
                    );
                    if exceeded {
                        self.log_event(
                            LogLevel::Error,
                            format!(
                                "Exceeded maxRetries ({max_retries}) delivering Kafka record: {reason}"
                            ),
                            record,
                            sink_url,
                            status,
                        )
                        .await;
                        self.emit_failed(
                            "DATAPLANE_MAX_RETRIES_EXCEEDED",
                            format!("Kafka HTTP sink delivery exceeded maxRetries ({max_retries}): {reason}"),
                        )
                        .await;
                        return false;
                    }

                    if self
                        .backoff_with_lease_renewal(cancel, lease_ticker, attempt)
                        .await
                    {
                        return false;
                    }
                    attempt = attempt.saturating_add(1);
                }
            }
        }
    }

    async fn post_record(
        &self,
        record: &KafkaRecord,
        sink_url: &str,
        sink_credentials: &ResolvedAuthCredentials,
    ) -> DeliveryOutcome {
        let headers = Self::build_kafka_headers(record);
        let body = Bytes::from(record.payload.clone());

        let res = HttpProxyDriver::forward(
            &self.client,
            Method::POST,
            sink_url,
            None,
            &headers,
            body,
            sink_credentials,
        )
        .await;

        match res {
            Ok(resp) if resp.status().is_success() => {
                DeliveryOutcome::Delivered(resp.status().as_u16())
            }
            Ok(resp) => {
                let status = resp.status();
                let code = status.as_u16();
                if status == StatusCode::REQUEST_TIMEOUT
                    || status == StatusCode::TOO_MANY_REQUESTS
                    || status.is_server_error()
                {
                    DeliveryOutcome::TransientFailure {
                        status: Some(code),
                        reason: format!("HTTP {code}"),
                    }
                } else {
                    DeliveryOutcome::PermanentFailure {
                        status: code,
                        reason: format!("HTTP {code}"),
                    }
                }
            }
            Err(err) => DeliveryOutcome::TransientFailure {
                status: None,
                reason: err.to_string(),
            },
        }
    }

    fn build_kafka_headers(record: &KafkaRecord) -> HeaderMap {
        let mut headers = HeaderMap::new();
        let content_type = if looks_like_json(&record.payload) {
            "application/json"
        } else {
            "application/octet-stream"
        };
        headers.insert(CONTENT_TYPE, HeaderValue::from_static(content_type));

        if let Ok(v) = HeaderValue::from_str(&record.topic) {
            headers.insert(HeaderName::from_static("x-kafka-topic"), v);
        }
        if let Ok(v) = HeaderValue::from_str(&record.partition.to_string()) {
            headers.insert(HeaderName::from_static("x-kafka-partition"), v);
        }
        if let Ok(v) = HeaderValue::from_str(&record.offset.to_string()) {
            headers.insert(HeaderName::from_static("x-kafka-offset"), v);
        }
        if let Some(ts) = record.timestamp_ms {
            if let Ok(v) = HeaderValue::from_str(&ts.to_string()) {
                headers.insert(HeaderName::from_static("x-kafka-timestamp"), v);
            }
        }
        if let Some(key_bytes) = &record.key {
            let encoded = match std::str::from_utf8(key_bytes) {
                Ok(utf8) if HeaderValue::from_str(utf8).is_ok() => utf8.to_string(),
                _ => format!("base64:{}", STANDARD.encode(key_bytes)),
            };
            if let Ok(v) = HeaderValue::from_str(&encoded) {
                headers.insert(HeaderName::from_static("x-kafka-key"), v);
            }
        }
        headers
    }

    fn extract_sink(
        egress: &DataplaneProxyEgress,
    ) -> Result<(String, ResolvedAuthCredentials), &'static str> {
        match egress {
            DataplaneProxyEgress::HttpProxy {
                path,
                token_type,
                token,
            }
            | DataplaneProxyEgress::DataClient {
                path,
                token_type,
                token,
            } if !path.trim().is_empty() => {
                let creds = match token.as_deref().filter(|t| !t.is_empty()) {
                    Some(tok) => match token_type.as_deref() {
                        None => ResolvedAuthCredentials::BearerToken {
                            token: tok.to_string(),
                        },
                        Some(kind) if kind.eq_ignore_ascii_case("Bearer") => {
                            ResolvedAuthCredentials::BearerToken {
                                token: tok.to_string(),
                            }
                        }
                        Some(kind) => ResolvedAuthCredentials::OAuth2 {
                            access_token: tok.to_string(),
                            token_type: kind.to_string(),
                            expires_at: None,
                            refresh_token: None,
                        },
                    },
                    None => ResolvedAuthCredentials::NoAuth,
                };
                Ok((path.clone(), creds))
            }
            _ => Err("missing or unsupported HTTP sink egress configuration"),
        }
    }

    async fn backoff_with_lease_renewal(
        &self,
        cancel: &CancellationToken,
        lease_ticker: &mut tokio::time::Interval,
        attempt: usize,
    ) -> bool {
        let ms = self
            .backoff_ms
            .get(attempt)
            .copied()
            .or_else(|| self.backoff_ms.last().copied())
            .unwrap_or(250);
        let sleep = tokio::time::sleep(Duration::from_millis(ms));
        tokio::pin!(sleep);

        loop {
            tokio::select! {
                biased;
                _ = cancel.cancelled() => return true,
                _ = &mut sleep => return false,
                _ = lease_ticker.tick() => {
                    if !self.lease.renew(&self.spec.dataplane_id).await {
                        warn!(
                            dataplane_id = %self.spec.dataplane_id,
                            "KafkaRelayWorker lost lease during retry backoff"
                        );
                        return true;
                    }
                }
            }
        }
    }

    async fn sleep_or_cancel(&self, cancel: &CancellationToken, attempt: usize) -> bool {
        let ms = self
            .backoff_ms
            .get(attempt)
            .copied()
            .or_else(|| self.backoff_ms.last().copied())
            .unwrap_or(250);
        tokio::select! {
            _ = cancel.cancelled() => true,
            _ = tokio::time::sleep(Duration::from_millis(ms)) => false,
        }
    }

    async fn emit_failed(&self, code: &str, reason: impl Into<String>) {
        let reason = reason.into();
        error!(
            dataplane_id = %self.spec.dataplane_id,
            transfer_process_id = %self.spec.transfer_process_id,
            code,
            reason = %reason,
            "KafkaRelayWorker emitting Failed signal"
        );
        self.signals.emit(DataplaneSignal::Failed {
            transfer_process_id: self.spec.transfer_process_id.clone(),
            code: code.to_string(),
            reason,
        });
    }

    async fn log_event(
        &self,
        level: LogLevel,
        message: String,
        record: &KafkaRecord,
        sink_url: &str,
        status: Option<u16>,
    ) {
        let Some(repo) = &self.repo else {
            return;
        };
        let event = NewTransferEvent {
            owner: self.spec.owner.clone(),
            transfer_id: self.spec.dataplane_id.clone(),
            level,
            component: "KafkaRelay".to_string(),
            message,
            data: Some(json!({
                "role": "PROVIDER",
                "mode": "PUSH",
                "ingress": "KafkaConsumer",
                "egress": "HttpSink",
                "topic": record.topic,
                "partition": record.partition,
                "offset": record.offset,
                "target": sink_url,
                "status": status,
            })),
        };
        if let Err(e) = repo
            .get_transfer_events_repo()
            .create_transfer_event(&event)
            .await
        {
            error!("Failed to persist KafkaRelay transfer event: {e:?}");
        }
    }
}

fn looks_like_json(payload: &[u8]) -> bool {
    let Ok(s) = std::str::from_utf8(payload) else {
        return false;
    };
    let trimmed = s.trim();
    if !(trimmed.starts_with('{') || trimmed.starts_with('[')) {
        return false;
    }
    serde_json::from_str::<serde::de::IgnoredAny>(trimmed).is_ok()
}

async fn validate_sink_url(target: &str, allow_private_egress: bool) -> Result<(), SinkTargetError> {
    let parsed = url::Url::parse(target).map_err(|_| SinkTargetError::InvalidUrl)?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(SinkTargetError::Forbidden("unsupported target URL scheme"));
    }
    let Some(host) = parsed.host() else {
        return Err(SinkTargetError::InvalidUrl);
    };

    if allow_private_egress {
        return Ok(());
    }

    match host {
        Host::Ipv4(ip) => {
            if !is_global_ipv4(ip) {
                return Err(SinkTargetError::Forbidden(
                    "private, loopback, or reserved target IP blocked",
                ));
            }
        }
        Host::Ipv6(ip) => {
            if !is_global_ipv6(ip) {
                return Err(SinkTargetError::Forbidden(
                    "private, loopback, or reserved target IP blocked",
                ));
            }
        }
        Host::Domain(domain) => {
            let lower = domain.trim_end_matches('.').to_ascii_lowercase();
            if lower == "localhost"
                || lower.ends_with(".localhost")
                || lower.ends_with(".internal")
            {
                return Err(SinkTargetError::Forbidden(
                    "local or internal target host blocked",
                ));
            }
            let port = parsed.port_or_known_default().unwrap_or(80);
            let addrs: Vec<_> = tokio::net::lookup_host((domain, port))
                .await
                .map_err(|_| SinkTargetError::ResolutionFailed)?
                .collect();
            if addrs.is_empty() {
                return Err(SinkTargetError::ResolutionFailed);
            }
            if addrs.iter().any(|addr| !is_global_ip(addr.ip())) {
                return Err(SinkTargetError::Forbidden(
                    "target host resolves to private, loopback, or reserved IP",
                ));
            }
        }
    }
    Ok(())
}

fn is_global_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_global_ipv4(v4),
        IpAddr::V6(v6) => is_global_ipv6(v6),
    }
}

fn is_global_ipv4(ip: Ipv4Addr) -> bool {
    let octets = ip.octets();
    if octets[0] == 0
        || ip.is_private()
        || ip.is_loopback()
        || ip.is_link_local()
        || ip.is_multicast()
        || ip.is_documentation()
        || ip.is_broadcast()
    {
        return false;
    }
    if octets[0] == 100 && (octets[1] & 0b1100_0000) == 64 {
        return false;
    }
    if octets[0] == 192 && octets[1] == 0 && octets[2] == 0 {
        return false;
    }
    if octets[0] == 198 && (octets[1] & 0xfe) == 18 {
        return false;
    }
    if octets[0] >= 240 {
        return false;
    }
    true
}

fn is_global_ipv6(ip: Ipv6Addr) -> bool {
    if ip.is_unspecified() || ip.is_loopback() || ip.is_multicast() {
        return false;
    }
    if let Some(v4) = ip.to_ipv4_mapped() {
        return is_global_ipv4(v4);
    }
    let segs = ip.segments();
    if (segs[0] & 0xfe00) == 0xfc00
        || (segs[0] & 0xffc0) == 0xfe80
        || (segs[0] == 0x2001 && segs[1] == 0x0db8)
    {
        return false;
    }
    true
}
