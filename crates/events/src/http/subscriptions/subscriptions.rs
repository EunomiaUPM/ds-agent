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

//! Subscription routes.

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use common::oauth::{Owner, OwnerScope, UserInfo};
use common::paginated_spec::{Cursor, Paginated};
use common::query::QuerySpec;
use ymir::errors::{AppResult, Errors};

use crate::data::repo::EventSubscriptionRepo;
use crate::entities::commands::{CreateSubscriptionDto, UpdateSubscriptionDto};
use crate::entities::queries::SubscriptionFilter;
use crate::services::event_bus::SubscriptionView;

/// Query string of the subscription listing.
pub type SubscriptionsQuery = QuerySpec<SubscriptionFilter>;

/// Webhook subscription routes.
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
        user: UserInfo,
        Json(dto): Json<CreateSubscriptionDto>,
    ) -> AppResult<(StatusCode, Json<SubscriptionView>)> {
        let owner = Owner::of(&user, dto.visibility.clone());
        let sub = repo.create_subscription(owner, dto).await?;
        Ok((StatusCode::CREATED, Json(SubscriptionView::assemble(sub))))
    }

    async fn handle_list(
        State(repo): State<Arc<dyn EventSubscriptionRepo>>,
        user: UserInfo,
        Query(query): Query<SubscriptionsQuery>,
    ) -> AppResult<Json<Paginated<SubscriptionView>>> {
        let page = query.page.clamped();
        let (subs, total) = repo
            .list_subscriptions(
                &OwnerScope::seeing(&user),
                &query.filter,
                &page,
                &query.sort,
            )
            .await?;
        Ok(Json(
            Paginated::from_page(subs, &page, Some(total), |last| {
                Cursor::encode_composite(&last.created_at, &last.id)
            })
            .map(SubscriptionView::assemble),
        ))
    }

    async fn handle_get(
        State(repo): State<Arc<dyn EventSubscriptionRepo>>,
        user: UserInfo,
        Path(id): Path<String>,
    ) -> AppResult<Json<SubscriptionView>> {
        let sub = repo
            .get_subscription(&OwnerScope::seeing(&user), &id)
            .await?
            .ok_or_else(|| Errors::missing_resource(&id, "subscription not found", None))?;
        Ok(Json(SubscriptionView::assemble(sub)))
    }

    async fn handle_update(
        State(repo): State<Arc<dyn EventSubscriptionRepo>>,
        user: UserInfo,
        Path(id): Path<String>,
        Json(dto): Json<UpdateSubscriptionDto>,
    ) -> AppResult<Json<SubscriptionView>> {
        let sub = repo
            .update_subscription(&OwnerScope::acting(&user), &id, dto)
            .await?;
        Ok(Json(SubscriptionView::assemble(sub)))
    }

    async fn handle_delete(
        State(repo): State<Arc<dyn EventSubscriptionRepo>>,
        user: UserInfo,
        Path(id): Path<String>,
    ) -> AppResult<StatusCode> {
        repo.delete_subscription(&OwnerScope::acting(&user), &id)
            .await?;
        Ok(StatusCode::NO_CONTENT)
    }
}
