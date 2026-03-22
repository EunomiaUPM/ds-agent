/*
 *
 *  * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
 *  *
 *  * This program is free software: you can redistribute it and/or modify
 *  * it under the terms of the GNU General Public License as published by
 *  * the Free Software Foundation, either version 3 of the License, or
 *  * (at your option) any later version.
 *  *
 *  * This program is distributed in the hope that it will be useful,
 *  * but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  * GNU General Public License for more details.
 *  *
 *  * You should have received a copy of the GNU General Public License
 *  * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use crate::core::notification::notification_err::NotificationErrors;
use crate::core::subscription::subscription_err::SubscriptionErrorMessage;
use axum::response::{IntoResponse, Response};
use axum::Json;
use reqwest::StatusCode;

impl IntoResponse for NotificationErrors {
    fn into_response(self) -> Response {
        match self {
            e @ NotificationErrors::DbErr(..) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(SubscriptionErrorMessage {
                    code: "500".to_string(),
                    title: "INTERNAL_ERROR".to_string(),
                    message: e.to_string(),
                }),
            )
                .into_response(),
            e @ NotificationErrors::NotFound { .. } => (
                StatusCode::NOT_FOUND,
                Json(SubscriptionErrorMessage {
                    code: "404".to_string(),
                    title: "NOT_FOUND".to_string(),
                    message: e.to_string(),
                }),
            )
                .into_response(),
            e @ NotificationErrors::JsonRejection { .. } => (
                StatusCode::BAD_REQUEST,
                Json(SubscriptionErrorMessage {
                    code: "400".to_string(),
                    title: "JSON_MALFORMED".to_string(),
                    message: e.to_string(),
                }),
            )
                .into_response(),
            e @ NotificationErrors::UrnUuidSchema { .. } => (
                StatusCode::BAD_REQUEST,
                Json(SubscriptionErrorMessage {
                    code: "400".to_string(),
                    title: "UUID_SCHEMA".to_string(),
                    message: e.to_string(),
                }),
            )
                .into_response(),
        }
    }
}
