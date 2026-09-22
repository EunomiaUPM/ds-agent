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

//! The wire-level context every protocol starts from, and the pieces DSP and
//! RPC share from there on.
use crate::entities::ids::{CorrelationId, IdempotencyKey, RequestId};
use crate::entities::transfer_message::Direction;
use crate::entities::transfer_process::TransferProcess;
use crate::protocols::dsp::entities::auth::TransferAuthn;
use axum::extract::{FromRequestParts, Path, Request};
use bytes::Bytes;
use chrono::{DateTime, Utc};
use common::dsp_common::normalizer::WireBody;
use connector::ConnectorInstanceDto;
use http::HeaderMap;
use http::request::Parts;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use ymir::errors::{BadFormat, Errors, Outcome};

/// Cap on inbound message bodies. Control-plane messages are tiny; this only
/// bounds a hostile/misbehaving peer.
const MAX_BODY_BYTES: usize = 1024 * 1024;

/// Wire-level context shared by every protocol
#[derive(Debug)]
pub struct TransferContextRaw<T: TransferAuthn> {
    pub request_id: RequestId,
    pub direction: Direction,
    pub request_full_path: String,
    pub request_path: String,
    pub path_id: Option<String>,
    pub method: http::Method,
    pub headers_in: HeaderMap,
    /// The body as the handler sees it — after the DSP namespace normalizer has
    /// rewritten prefixes. This is what gets parsed and expanded.
    pub body_bytes: Bytes,
    pub incoming_at: DateTime<Utc>,
    /// The `Idempotency-Key` header as sent: one ingredient of the effective key,
    /// not the key. See [`crate::protocols::dsp::entities::idempotency`].
    pub supplied_idempotency_key: Option<IdempotencyKey>,
    pub authn: T,
    pub correlation_id: Option<CorrelationId>,
}

/// How each protocol builds its auth from the request parts — the only
/// wire-extraction step that varies between DSP and RPC.
pub trait BuildAuthn: TransferAuthn + Sized {
    fn from_request_parts(parts: &Parts) -> Outcome<Self>;

    /// Header getter, shared by the impls and by the raw-context extraction.
    fn header(headers: &HeaderMap, name: &str) -> Option<String> {
        headers
            .get(name)
            .and_then(|v| v.to_str().ok())
            .map(str::to_string)
    }
}

impl<T: BuildAuthn> TransferContextRaw<T> {
    pub async fn from_request(request: Request) -> Outcome<Self> {
        let incoming_at = Utc::now();
        let (mut parts, body) = request.into_parts();
        let body_bytes = axum::body::to_bytes(body, MAX_BODY_BYTES)
            .await
            .map_err(|_| {
                Errors::format(BadFormat::Received, "failed to read request body", None)
            })?;
        let request_id = T::header(&parts.headers, "x-request-id")
            .map(RequestId::new)
            .unwrap_or_else(RequestId::generate);
        let correlation_id = T::header(&parts.headers, "x-correlation-id").map(CorrelationId::new);
        let supplied_idempotency_key =
            T::header(&parts.headers, "idempotency-key").map(IdempotencyKey::new);
        let request_path = parts.uri.path().to_string();
        let request_full_path = parts
            .uri
            .path_and_query()
            .map(|pq| pq.as_str().to_string())
            .unwrap_or_else(|| request_path.clone());
        let path_params = Path::<HashMap<String, String>>::from_request_parts(&mut parts, &())
            .await
            .map(|Path(p)| p)
            .unwrap_or_default();
        let path_id = path_params.get("id").cloned();

        // Protocol-specific: build this side's auth from the parts.
        let inter_peer_authn = T::from_request_parts(&parts)?;

        Ok(Self {
            request_id,
            direction: Direction::Inbound,
            request_full_path,
            request_path,
            path_id,
            method: parts.method,
            headers_in: parts.headers,
            body_bytes,
            incoming_at,
            supplied_idempotency_key,
            authn: inter_peer_authn,
            correlation_id,
        })
    }
}

impl<T: TransferAuthn> TransferContextRaw<T> {
    /// SHA-256 over the exact bytes sent — the only hash that can attest to the
    /// whole message, and too strict to key on. See [`super::idempotency`].
    pub fn wire_hash(&self) -> [u8; 32] {
        Sha256::digest(&self.body_bytes).into()
    }
}

/// Whether the local process for this transfer already exists or must be minted.
/// Shared by every protocol (DSP, RPC) and the data-plane facades.
#[derive(Debug)]
pub enum TransferContextProcessSlot {
    Existing(TransferProcess),
    New { consumer_pid: String },
}

/// The connector backing this transfer: a provider resolves one from the
/// agreement; a consumer has none.
#[derive(Debug)]
pub enum TransferContextConnectorRole {
    ConsumerNotHavingConnector,
    ProviderHavingConnector(ConnectorInstanceDto),
}
