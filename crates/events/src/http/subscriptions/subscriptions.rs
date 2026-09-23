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

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};

use common::auth::AccessScope;

use crate::data::repo::EventSubscriptionRepo;
use crate::entities::commands::{CreateSubscriptionDto, UpdateSubscriptionDto};
use crate::entities::subscription::SubscriptionRecord;
use ymir::errors::{AppResult, Errors};

// Axum HTTP router handling webhook subscription lifecycle endpoints.
#[derive(Clone)]
pub struct SubscriptionsRouter {
    repo: Arc<dyn EventSubscriptionRepo>,
}

impl SubscriptionsRouter {
    // Create router backed by the subscription repository.
    pub fn new(repo: Arc<dyn EventSubscriptionRepo>) -> Self {
        Self { repo }
    }

    // Build Axum sub-router for subscription CRUD endpoints.
    pub fn router(self) -> Router {
        Router::new()
            .route("/", post(Self::handle_create))
            .route("/", get(Self::handle_list))
            .route("/{id}", get(Self::handle_get))
            .route("/{id}", put(Self::handle_update))
            .route("/{id}", delete(Self::handle_delete))
            .with_state(self.repo)
    }

    // Handler to register a new subscription.
    async fn handle_create(
        State(repo): State<Arc<dyn EventSubscriptionRepo>>,
        scope: AccessScope,
        Json(dto): Json<CreateSubscriptionDto>,
    ) -> AppResult<(StatusCode, Json<SubscriptionRecord>)> {
        let tenant_id = scope.acting_tenant();
        let sub = repo.create_subscription(tenant_id, dto).await?;

        Ok((StatusCode::CREATED, Json(sub)))
    }

    // Handler to list all active subscriptions.
    async fn handle_list(
        State(repo): State<Arc<dyn EventSubscriptionRepo>>,
        scope: AccessScope,
    ) -> AppResult<Json<Vec<SubscriptionRecord>>> {
        let tenant_id = scope.acting_tenant();
        let subs = repo.list_subscriptions(tenant_id).await?;

        Ok(Json(subs))
    }

    // Handler to fetch subscription details by identifier.
    async fn handle_get(
        State(repo): State<Arc<dyn EventSubscriptionRepo>>,
        scope: AccessScope,
        Path(id): Path<String>,
    ) -> AppResult<Json<SubscriptionRecord>> {
        let tenant_id = scope.acting_tenant();
        let sub = repo
            .get_subscription(tenant_id, &id)
            .await?
            .ok_or_else(|| Errors::missing_resource(&id, "subscription not found", None))?;

        Ok(Json(sub))
    }

    // Handler to update an existing subscription.
    async fn handle_update(
        State(repo): State<Arc<dyn EventSubscriptionRepo>>,
        scope: AccessScope,
        Path(id): Path<String>,
        Json(dto): Json<UpdateSubscriptionDto>,
    ) -> AppResult<Json<SubscriptionRecord>> {
        let tenant_id = scope.acting_tenant();
        let sub = repo.update_subscription(tenant_id, &id, dto).await?;

        Ok(Json(sub))
    }

    // Handler to deactivate and remove a subscription.
    async fn handle_delete(
        State(repo): State<Arc<dyn EventSubscriptionRepo>>,
        scope: AccessScope,
        Path(id): Path<String>,
    ) -> AppResult<StatusCode> {
        let tenant_id = scope.acting_tenant();
        repo.delete_subscription(tenant_id, &id).await?;

        Ok(StatusCode::NO_CONTENT)
    }
}
