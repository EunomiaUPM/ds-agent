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

//! Browser helpers fetching public documents of remote connectors, which CORS would block.

use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::Value;

pub(crate) struct DiscoveryHandlers;

impl DiscoveryHandlers {
    pub(crate) async fn did_json(Path(url): Path<String>) -> Response {
        let target = format!("{}/api/v1/wallet/did.json", url.trim_end_matches('/'));
        Self::fetch_json(&target, "DID document").await
    }

    pub(crate) async fn federated_catalog(Path(url): Path<String>) -> Response {
        let target = format!(
            "{}/.well-known/federated-catalog",
            url.trim_end_matches('/')
        );
        Self::fetch_json(&target, "federated catalog").await
    }

    async fn fetch_json(target: &str, what: &str) -> Response {
        let response = match reqwest::get(target).await {
            Ok(response) => response,
            Err(e) => {
                return (
                    StatusCode::BAD_GATEWAY,
                    format!("Failed to fetch {what}: {e}"),
                )
                    .into_response()
            }
        };
        match response.json::<Value>().await {
            Ok(json) => Json(json).into_response(),
            Err(e) => (
                StatusCode::BAD_GATEWAY,
                format!("Failed to parse {what}: {e}"),
            )
                .into_response(),
        }
    }
}
