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
use axum::routing::get;
use axum::{Json, Router};
use common::auth::AccessScope;
use common::paginated_spec::{Cursor, Paginated};
use common::query::QuerySpec;
use ymir::errors::{AppResult, Errors};

use crate::data::repo::EventSubscriptionRepo;
use crate::entities::commands::{CreateSubscriptionDto, UpdateSubscriptionDto};
use crate::entities::queries::SubscriptionFilter;
use crate::entities::subscription::SubscriptionRecord;

pub type SubscriptionsQuery = QuerySpec<SubscriptionFilter>;

// Axum HTTP router for webhook subscription management.
#[derive(Clone)]
pub struct SubscriptionsRouter {
    repo: Arc<dyn EventSubscriptionRepo>,
}

impl SubscriptionsRouter {
    pub fn new(repo: Arc<dyn EventSubscriptionRepo>) -> Self {
        Self { repo }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route("/", get(Self::handle_list).post(Self::handle_create))
            .route(
                "/{id}",
                get(Self::handle_get)
                    .put(Self::handle_update)
                    .delete(Self::handle_delete),
            )
            .with_state(self.repo)
    }

    async fn handle_create(
        State(repo): State<Arc<dyn EventSubscriptionRepo>>,
        scope: AccessScope,
        Json(dto): Json<CreateSubscriptionDto>,
    ) -> AppResult<(StatusCode, Json<SubscriptionRecord>)> {
        let tenant_id = scope.resolve_create_tenant(None)?;
        let sub = repo.create_subscription(&tenant_id, dto).await?;
        Ok((StatusCode::CREATED, Json(sub)))
    }

    async fn handle_list(
        State(repo): State<Arc<dyn EventSubscriptionRepo>>,
        scope: AccessScope,
        Query(query): Query<SubscriptionsQuery>,
    ) -> AppResult<Json<Paginated<SubscriptionRecord>>> {
        let page = query.page.clamped();
        let (subs, total) = repo
            .list_subscriptions(
                scope.tenant_filter().map(str::to_string),
                &query.filter,
                &page,
                &query.sort,
            )
            .await?;
        Ok(Json(Paginated::from_page(
            subs,
            &page,
            Some(total),
            |last| Cursor::encode_composite(&last.created_at, &last.id),
        )))
    }

    async fn handle_get(
        State(repo): State<Arc<dyn EventSubscriptionRepo>>,
        scope: AccessScope,
        Path(id): Path<String>,
    ) -> AppResult<Json<SubscriptionRecord>> {
        let sub = repo
            .get_subscription(scope.tenant_filter().map(str::to_string), &id)
            .await?
            .ok_or_else(|| Errors::missing_resource(&id, "subscription not found", None))?;
        Ok(Json(sub))
    }

    async fn handle_update(
        State(repo): State<Arc<dyn EventSubscriptionRepo>>,
        scope: AccessScope,
        Path(id): Path<String>,
        Json(dto): Json<UpdateSubscriptionDto>,
    ) -> AppResult<Json<SubscriptionRecord>> {
        scope.require_write()?;
        let sub = repo
            .update_subscription(scope.tenant_filter().map(str::to_string), &id, dto)
            .await?;
        Ok(Json(sub))
    }

    async fn handle_delete(
        State(repo): State<Arc<dyn EventSubscriptionRepo>>,
        scope: AccessScope,
        Path(id): Path<String>,
    ) -> AppResult<StatusCode> {
        scope.require_write()?;
        repo.delete_subscription(scope.tenant_filter().map(str::to_string), &id)
            .await?;
        Ok(StatusCode::NO_CONTENT)
    }
}
