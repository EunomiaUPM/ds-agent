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
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use thiserror::Error;
use uuid::Uuid;

/// Error variants encountered during event bus and delivery operations.
#[derive(Debug, Error)]
pub enum EventBusError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Subscription not found: {0}")]
    SubscriptionNotFound(Uuid),

    #[error("Event not found: {0}")]
    EventNotFound(Uuid),

    #[error("Delivery not found: {0}")]
    DeliveryNotFound(Uuid),

    #[error("Dead letter not found: {0}")]
    DeadLetterNotFound(Uuid),

    #[error("Dispatch failed: {0}")]
    DispatchFailed(String),

    #[error("Invalid topic: {0}")]
    InvalidTopic(String),

    #[error("Invalid topic pattern: {0}")]
    InvalidTopicPattern(String),

    #[error("Invalid URN: {0}")]
    InvalidUrn(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Broadcast channel error: {0}")]
    BroadcastError(String),
}

impl IntoResponse for EventBusError {
    fn into_response(self) -> Response {
        let (status, code) = match self {
            Self::SubscriptionNotFound(_)
            | Self::EventNotFound(_)
            | Self::DeliveryNotFound(_)
            | Self::DeadLetterNotFound(_) => (StatusCode::NOT_FOUND, 4040),
            Self::InvalidTopic(_)
            | Self::InvalidTopicPattern(_)
            | Self::InvalidUrn(_)
            | Self::BadRequest(_) => (StatusCode::BAD_REQUEST, 4000),
            Self::Serialization(_) => (StatusCode::BAD_REQUEST, 4001),
            Self::DispatchFailed(_) => (StatusCode::BAD_GATEWAY, 5020),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, 5000),
        };

        (
            status,
            Json(json!({
                "message": self.to_string(),
                "error_code": code
            })),
        )
            .into_response()
    }
}
