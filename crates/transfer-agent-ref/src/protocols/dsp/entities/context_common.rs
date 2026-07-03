use crate::entities::ids::{CorrelationId, IdempotencyKey, RequestId};
use crate::entities::message_envelope::Direction;
use crate::entities::transfer_process::TransferProcess;
use crate::protocols::dsp::entities::auth::TransferAuthn;
use axum::extract::Request;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use connector::ConnectorInstanceDto;
use http::HeaderMap;
use http::request::Parts;
use ymir::errors::{BadFormat, Errors, Outcome};

/// Common context data. Shared in DSP and RPC context

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
    pub method: http::Method,
    pub headers_in: HeaderMap,
    pub body_bytes: Bytes,
    pub incoming_at: DateTime<Utc>,
    pub idempotency_key: Option<IdempotencyKey>,
    pub authn: T,
    pub correlation_id: Option<CorrelationId>,
}

/// How each protocol builds its auth from the request parts.
/// DSP reads the peeridentity the middleware already resolved;
/// RPC resolves the calling user.
/// This is the only wire-extraction step that varies per protocol.
pub trait BuildAuthn: TransferAuthn + Sized {
    fn from_request_parts(parts: &Parts) -> Outcome<Self>;
}

impl<T: BuildAuthn> TransferContextRaw<T> {
    pub async fn from_request(request: Request) -> Outcome<Self> {
        let incoming_at = Utc::now();
        let (parts, body) = request.into_parts();

        let body_bytes = axum::body::to_bytes(body, MAX_BODY_BYTES)
            .await
            .map_err(|_| {
                Errors::format(BadFormat::Received, "failed to read request body", None)
            })?;

        let request_id = header(&parts.headers, "x-request-id")
            .map(RequestId::new)
            .unwrap_or_else(RequestId::generate);
        let correlation_id = header(&parts.headers, "x-correlation-id").map(CorrelationId::new);
        let idempotency_key = header(&parts.headers, "idempotency-key").map(IdempotencyKey::new);

        let request_path = parts.uri.path().to_string();
        let request_full_path = parts
            .uri
            .path_and_query()
            .map(|pq| pq.as_str().to_string())
            .unwrap_or_else(|| request_path.clone());

        // Protocol-specific: build this side's auth from the parts.
        let inter_peer_authn = T::from_request_parts(&parts)?;

        Ok(Self {
            request_id,
            direction: Direction::Inbound,
            request_full_path,
            request_path,
            method: parts.method,
            headers_in: parts.headers,
            body_bytes,
            incoming_at,
            idempotency_key,
            authn: inter_peer_authn,
            correlation_id,
        })
    }
}

/// Header getter, shared by the protocol `BuildAuthn` impls.
pub(crate) fn header(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
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
