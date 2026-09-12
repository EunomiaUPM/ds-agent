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

use crate::data::repo::EventSubscriptionRepo;
use crate::entities::commands::{CreateSubscriptionDto, UpdateSubscriptionDto};
use crate::entities::subscription::SubscriptionRecord;
use crate::errors::EventBusError;

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
        Json(dto): Json<CreateSubscriptionDto>,
    ) -> Result<(StatusCode, Json<SubscriptionRecord>), EventBusError> {
        let sub = repo
            .create_subscription(dto)
            .await
            .map_err(|e| EventBusError::Database(format!("{e:?}")))?;

        Ok((StatusCode::CREATED, Json(sub)))
    }

    // Handler to list all active subscriptions.
    async fn handle_list(
        State(repo): State<Arc<dyn EventSubscriptionRepo>>,
    ) -> Result<Json<Vec<SubscriptionRecord>>, EventBusError> {
        let subs = repo
            .list_subscriptions()
            .await
            .map_err(|e| EventBusError::Database(format!("{e:?}")))?;

        Ok(Json(subs))
    }

    // Handler to fetch subscription details by identifier.
    async fn handle_get(
        State(repo): State<Arc<dyn EventSubscriptionRepo>>,
        Path(id): Path<String>,
    ) -> Result<Json<SubscriptionRecord>, EventBusError> {
        let sub = repo
            .get_subscription(&id)
            .await
            .map_err(|e| EventBusError::Database(format!("{e:?}")))?
            .ok_or_else(|| {
                EventBusError::SubscriptionNotFound(uuid::Uuid::parse_str(&id).unwrap_or_default())
            })?;

        Ok(Json(sub))
    }

    // Handler to update an existing subscription.
    async fn handle_update(
        State(repo): State<Arc<dyn EventSubscriptionRepo>>,
        Path(id): Path<String>,
        Json(dto): Json<UpdateSubscriptionDto>,
    ) -> Result<Json<SubscriptionRecord>, EventBusError> {
        let sub = repo.update_subscription(&id, dto).await.map_err(|e| {
            let err_str = format!("{e:?}");
            if err_str.contains("missing_resource") || err_str.contains("not found") {
                EventBusError::SubscriptionNotFound(uuid::Uuid::parse_str(&id).unwrap_or_default())
            } else {
                EventBusError::Database(err_str)
            }
        })?;

        Ok(Json(sub))
    }

    // Handler to deactivate and remove a subscription.
    async fn handle_delete(
        State(repo): State<Arc<dyn EventSubscriptionRepo>>,
        Path(id): Path<String>,
    ) -> Result<StatusCode, EventBusError> {
        repo.delete_subscription(&id)
            .await
            .map_err(|e| EventBusError::Database(format!("{e:?}")))?;

        Ok(StatusCode::NO_CONTENT)
    }
}
