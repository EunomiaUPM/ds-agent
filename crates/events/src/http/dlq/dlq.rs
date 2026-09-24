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

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use common::auth::AccessScope;
use common::paginated_spec::{Cursor, Paginated};
use common::query::QuerySpec;
use serde_json::json;
use ymir::errors::{AppResult, Errors};

use crate::entities::dead_letter::DeadLetterRecord;
use crate::entities::delivery::EventDeliveryRecord;
use crate::entities::queries::DeadLetterFilter;
use crate::services::event_bus::EventBus;

pub type DeadLettersQuery = QuerySpec<DeadLetterFilter>;

// Axum HTTP router handling Dead Letter Queue inspection and redrive endpoints.
#[derive(Clone)]
pub struct DeadLetterRouter {
    bus: Arc<EventBus>,
}

impl DeadLetterRouter {
    pub fn new(bus: Arc<EventBus>) -> Self {
        Self { bus }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route("/", get(Self::handle_list))
            .route("/replay-all", post(Self::handle_replay_all))
            .route("/{id}", get(Self::handle_get).delete(Self::handle_delete))
            .route("/{id}/replay", post(Self::handle_replay))
            .with_state(self.bus)
    }

    async fn handle_list(
        State(bus): State<Arc<EventBus>>,
        scope: AccessScope,
        Query(query): Query<DeadLettersQuery>,
    ) -> AppResult<Json<Paginated<DeadLetterRecord>>> {
        let page = query.page.clamped();
        let (dead_letters, total) = bus
            .dlq_repo()
            .list_dead_letters(
                scope.tenant_filter().map(str::to_string),
                &query.filter,
                &page,
                &query.sort,
            )
            .await?;
        Ok(Json(Paginated::from_page(
            dead_letters,
            &page,
            Some(total),
            |last| Cursor::encode_composite(&last.failed_at, &last.id),
        )))
    }

    async fn handle_get(
        State(bus): State<Arc<EventBus>>,
        scope: AccessScope,
        Path(id): Path<String>,
    ) -> AppResult<Json<DeadLetterRecord>> {
        let dead_letter = bus
            .dlq_repo()
            .get_dead_letter(scope.tenant_filter().map(str::to_string), &id)
            .await?
            .ok_or_else(|| Errors::missing_resource(&id, "dead letter not found", None))?;
        Ok(Json(dead_letter))
    }

    async fn handle_replay(
        State(bus): State<Arc<EventBus>>,
        scope: AccessScope,
        Path(id): Path<String>,
    ) -> AppResult<Json<EventDeliveryRecord>> {
        scope.require_write()?;
        let delivery = bus
            .replay_dead_letter(scope.tenant_filter().map(str::to_string), &id)
            .await?;
        Ok(Json(delivery))
    }

    async fn handle_replay_all(
        State(bus): State<Arc<EventBus>>,
        scope: AccessScope,
    ) -> AppResult<Json<serde_json::Value>> {
        scope.require_write()?;
        let count = bus
            .replay_all_dead_letters(scope.tenant_filter().map(str::to_string))
            .await?;
        Ok(Json(json!({ "replayed_count": count })))
    }

    async fn handle_delete(
        State(bus): State<Arc<EventBus>>,
        scope: AccessScope,
        Path(id): Path<String>,
    ) -> AppResult<StatusCode> {
        scope.require_write()?;
        bus.dlq_repo()
            .delete_dead_letter(scope.tenant_filter().map(str::to_string), &id)
            .await?;
        Ok(StatusCode::NO_CONTENT)
    }
}
