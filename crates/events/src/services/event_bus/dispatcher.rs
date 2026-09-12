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
use std::time::Duration;

use hmac::{Hmac, Mac};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::{Client, StatusCode};
use sha2::Sha256;

use crate::entities::envelope::EventEnvelope;

type HmacSha256 = Hmac<Sha256>;

// Dispatches HTTP webhook deliveries with HMAC-SHA256 signatures and tracking headers.
#[derive(Clone)]
pub struct EventDispatcher {
    client: Client,
}

impl EventDispatcher {
    // Create dispatcher with specified client timeout.
    pub fn new(timeout: Duration) -> Self {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .unwrap_or_default();
        Self { client }
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

        let mut req = self
            .client
            .post(callback_url)
            .header("Content-Type", "application/json")
            .header("X-Event-Id", envelope.id.as_str())
            .header("X-Event-Topic", envelope.topic.as_str())
            .header("X-Event-Timestamp", envelope.timestamp.to_rfc3339());

        if let Some(corr_id) = &envelope.correlation_id {
            req = req.header("X-Correlation-Id", corr_id.as_str());
        }

        if let Some(sec) = secret {
            let sig = Self::compute_signature(sec, &payload_bytes);
            req = req.header("X-Hub-Signature-256", sig);
        }

        if let Some(headers) = custom_headers {
            let mut header_map = HeaderMap::new();
            for (k, v) in headers {
                if let (Ok(name), Ok(val)) = (
                    HeaderName::from_bytes(k.as_bytes()),
                    HeaderValue::from_str(v),
                ) {
                    header_map.insert(name, val);
                }
            }
            req = req.headers(header_map);
        }

        let resp = req
            .body(payload_bytes)
            .send()
            .await
            .map_err(|e| format!("HTTP request error: {e}"))?;

        Ok(resp.status())
    }
}
