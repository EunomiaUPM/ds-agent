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

use axum::http::StatusCode;
use thiserror::Error;
use ymir::errors::{BadFormat, Errors, PetitionFailure};

#[derive(Debug, Error)]
pub enum DataplaneError {
    // Transfer lifecycle ──────────────────────────────────────────────────
    #[error("Dataplane Error: Transfer not found: {transfer_process_id}")]
    TransferNotFound { transfer_process_id: String },

    #[error("Dataplane Error: Command '{command}' is not valid in the current context")]
    UnexpectedCommand { command: String },

    // Driver / connector setup ────────────────────────────────────────────
    #[error("Dataplane Error: Connector instance is not available")]
    ConnectorNotAvailable,

    #[error("Dataplane Error: No driver available for role={role}, interaction_mode={mode}")]
    NoDriverForCombination { role: String, mode: String },

    #[error("Dataplane Error: Auth config type mismatch: driver expected {expected}")]
    AuthConfigMismatch { expected: String },

    #[error("Dataplane Error: Feature not implemented: {feature}")]
    FeatureNotImplemented { feature: String },

    #[error("Dataplane Error: Wrong interaction type: expected {expected}, found {found}")]
    WrongInteractionType { expected: String, found: String },

    #[error("Dataplane Error: Unsupported protocol: {protocol}")]
    UnsupportedProtocol { protocol: String },

    #[error("Dataplane Error: Required transfer context is missing: {detail}")]
    MissingTransferContext { detail: String },

    // Authentication (outgoing auth calls) ────────────────────────────────
    #[error("Dataplane Error: Auth network error calling {url}: {reason}")]
    AuthNetworkError { url: String, reason: String },

    #[error("Dataplane Error: Auth endpoint {url} returned HTTP {status}: {body}")]
    AuthEndpointError { url: String, status: u16, body: String },

    #[error("Dataplane Error: Failed to parse auth endpoint response from {url}: {reason}")]
    AuthResponseParseFailed { url: String, reason: String },

    // Proxy HTTP ───────────────────────────────────────────────────────────
    #[error("Dataplane Error: Invalid HTTP header '{header}': {reason}")]
    InvalidHeaderValue { header: String, reason: String },

    #[error("Dataplane Error: Proxy HTTP request {method} {url} failed: {reason}")]
    ProxyRequestFailed { method: String, url: String, reason: String },

    // PubSub ──────────────────────────────────────────────────────────────
    #[error("Dataplane Error: No connector available for pubsub operation '{operation}'")]
    PubSubConnectorNotAvailable { operation: String },

    #[error("Dataplane Error: PubSub {method} to {url} failed: {reason}")]
    PubSubRequestFailed { method: String, url: String, reason: String },

    // State machine / serialization ────────────────────────────────────────
    #[error("Dataplane Error: Failed to serialize runtime state: {reason}")]
    RuntimeSerializationFailed { reason: String },

    // Cache (Redis) ────────────────────────────────────────────────────────
    #[error("Dataplane Error: Redis cache operation failed: {reason}")]
    CacheQueryFailed { reason: String },

    // Configuration ────────────────────────────────────────────────────────
    #[error("Dataplane Error: RoleConfig::NotDefined is not valid in dataplane context")]
    InvalidRoleConfig,
}

const DP: &str = "[Dataplane]";

impl From<DataplaneError> for Errors {
    fn from(e: DataplaneError) -> Errors {
        match e {
            DataplaneError::TransferNotFound { transfer_process_id } => {
                Errors::missing_resource(transfer_process_id, format!("{DP} Transfer process not found in dataplane"), None)
            }

            DataplaneError::FeatureNotImplemented { feature } => {
                Errors::not_impl(format!("{DP} {feature}"), None)
            }

            DataplaneError::UnsupportedProtocol { protocol } => {
                Errors::not_impl(format!("{DP} Protocol not supported: {protocol}"), None)
            }

            DataplaneError::AuthNetworkError { url, reason } => {
                Errors::petition(url, "POST", None, PetitionFailure::Network, format!("{DP} {reason}"), None)
            }

            DataplaneError::AuthEndpointError { url, status, body } => {
                let sc = StatusCode::from_u16(status).unwrap_or(StatusCode::BAD_GATEWAY);
                Errors::petition(url, "POST", Some(sc), PetitionFailure::HttpStatus(sc), format!("{DP} {body}"), None)
            }

            DataplaneError::AuthResponseParseFailed { url, reason } => {
                Errors::petition(url, "POST", None, PetitionFailure::BodyDeserialization, format!("{DP} {reason}"), None)
            }

            DataplaneError::ProxyRequestFailed { method, url, reason } => {
                Errors::petition(url, method, None, PetitionFailure::Network, format!("{DP} {reason}"), None)
            }

            DataplaneError::PubSubRequestFailed { method, url, reason } => {
                Errors::petition(url, method, None, PetitionFailure::Network, format!("{DP} {reason}"), None)
            }

            DataplaneError::InvalidHeaderValue { header, reason } => {
                Errors::format(BadFormat::Sent, format!("{DP} [header={header}] {reason}"), None)
            }

            DataplaneError::UnexpectedCommand { command } => {
                Errors::crazy(format!("{DP} Unexpected command: {command}"), None)
            }

            DataplaneError::ConnectorNotAvailable => {
                Errors::crazy(format!("{DP} Connector instance not available"), None)
            }

            DataplaneError::NoDriverForCombination { role, mode } => {
                Errors::crazy(format!("{DP} No driver for role={role} mode={mode}"), None)
            }

            DataplaneError::AuthConfigMismatch { expected } => {
                Errors::crazy(format!("{DP} Auth config mismatch: expected {expected}"), None)
            }

            DataplaneError::WrongInteractionType { expected, found } => {
                Errors::crazy(format!("{DP} Wrong interaction type: expected={expected} found={found}"), None)
            }

            DataplaneError::MissingTransferContext { detail } => {
                Errors::crazy(format!("{DP} Missing transfer context: {detail}"), None)
            }

            DataplaneError::PubSubConnectorNotAvailable { operation } => {
                Errors::crazy(format!("{DP} No connector for pubsub operation: {operation}"), None)
            }

            DataplaneError::RuntimeSerializationFailed { reason } => {
                Errors::crazy(format!("{DP} Runtime serialization failed: {reason}"), None)
            }

            DataplaneError::CacheQueryFailed { reason } => {
                Errors::crazy(format!("{DP} Redis cache operation failed: {reason}"), None)
            }

            DataplaneError::InvalidRoleConfig => {
                Errors::crazy(format!("{DP} RoleConfig::NotDefined is not valid in dataplane context"), None)
            }
        }
    }
}
