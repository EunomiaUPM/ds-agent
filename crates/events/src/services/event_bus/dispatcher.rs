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

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use common::telemetry::TraceParent;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use tracing::Instrument;
use ymir::services::client::{ClientService, ClientTrait};
use ymir::types::http::{Method, StreamBody};

use crate::entities::envelope::EventEnvelope;

type HmacSha256 = Hmac<Sha256>;

// Dispatches HTTP webhook deliveries with HMAC-SHA256 signatures and tracking headers.
#[derive(Clone)]
pub struct EventDispatcher {
    client: Arc<ClientService>,
}

impl EventDispatcher {
    // Create dispatcher with specified client timeout.
    pub fn new(timeout: Duration) -> Self {
        let client = ClientService::builder().timeout(Some(timeout)).build();
        Self {
            client: Arc::new(client),
        }
    }

    // Compute HMAC-SHA256 signature formatted as standard sha256=hex.
    pub fn compute_signature(secret: &str, payload: &[u8]) -> String {
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .expect("HMAC-SHA256 accepts any key size");
        mac.update(payload);
        let bytes = mac.finalize().into_bytes();
        let hex = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();
        format!("sha256={hex}")
    }

    // Dispatch envelope to callback URL with optional HMAC secret and custom headers.
    pub async fn dispatch(
        &self,
        callback_url: &str,
        envelope: &EventEnvelope,
        secret: Option<&str>,
        custom_headers: Option<&HashMap<String, String>>,
    ) -> Result<StatusCode, String> {
        let payload_bytes = serde_json::to_vec(&envelope.payload)
            .map_err(|e| format!("failed to serialize payload: {e}"))?;

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        Self::insert(&mut headers, "X-Event-Id", envelope.id.as_str());
        Self::insert(&mut headers, "X-Event-Topic", envelope.topic.as_str());
        Self::insert(
            &mut headers,
            "X-Event-Timestamp",
            &envelope.timestamp.to_rfc3339(),
        );
        if let Some(corr_id) = &envelope.correlation_id {
            Self::insert(&mut headers, "X-Correlation-Id", corr_id.as_str());
        }
        if let Some(sec) = secret {
            let sig = Self::compute_signature(sec, &payload_bytes);
            Self::insert(&mut headers, "X-Hub-Signature-256", &sig);
        }
        for (k, v) in custom_headers.into_iter().flatten() {
            Self::insert(&mut headers, k, v);
        }

        // Sent once and any status returned: redelivery is the retry worker's job.
        let resp = self
            .client
            .stream(
                Method::POST,
                callback_url,
                Some(headers),
                StreamBody::from(payload_bytes),
                None,
            )
            .instrument(Self::delivery_span(envelope))
            .await
            .map_err(|e| format!("HTTP request error: {e}"))?;

        Ok(resp.status())
    }

    /// Producer span of one webhook attempt, linked to the trace that published the event;
    /// its context is what the subscriber receives as `traceparent`.
    fn delivery_span(envelope: &EventEnvelope) -> tracing::Span {
        let span = tracing::info_span!(
            "event.deliver",
            otel.name = %format!("{} publish", envelope.topic.as_str()),
            otel.kind = "producer",
            messaging.system = "webhook",
            messaging.message.id = %envelope.id,
            messaging.destination.name = %envelope.topic.as_str(),
        );
        TraceParent::link(&span, envelope.trace_context.as_deref());
        span
    }

    /// Adds a header, skipping names or values that are not valid HTTP.
    fn insert(headers: &mut HeaderMap, name: &str, value: &str) {
        if let (Ok(name), Ok(val)) = (
            HeaderName::from_bytes(name.as_bytes()),
            HeaderValue::from_str(value),
        ) {
            headers.insert(name, val);
        }
    }
}
