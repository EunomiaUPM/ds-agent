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

use std::str::FromStr;
use std::sync::Arc;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use events::bus::EventBus;
use serde::Deserialize;
use serde_json::json;
use urn::Urn;

#[derive(Debug, Deserialize)]
pub struct ListEventsQuery {
    pub topic: Option<String>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

pub struct BffEventFeedRouter;

impl BffEventFeedRouter {
    /// Handle listing events for the UI audit/activity feed.
    pub async fn handle_list(bus: Arc<EventBus>, q: ListEventsQuery) -> Response {
        let limit = q.limit.unwrap_or(50).min(100);
        let offset = q.offset.unwrap_or(0);

        match bus
            .event_repo()
            .list_events(q.topic.as_deref(), limit, offset)
            .await
        {
            Ok(events) => (StatusCode::OK, Json(events)).into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("{e:?}") })),
            )
                .into_response(),
        }
    }

    /// Handle fetching single event details by URN or UUID.
    pub async fn handle_get(bus: Arc<EventBus>, id: String) -> Response {
        let urn = match Urn::from_str(&id).or_else(|_| Urn::from_str(&format!("urn:uuid:{id}"))) {
            Ok(u) => u,
            Err(_) => return (StatusCode::BAD_REQUEST, "invalid event URN format").into_response(),
        };

        match bus.event_repo().get_event_by_id(&urn).await {
            Ok(Some(ev)) => (StatusCode::OK, Json(ev)).into_response(),
            Ok(None) => (StatusCode::NOT_FOUND, "event not found").into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("{e:?}") })),
            )
                .into_response(),
        }
    }
}
