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

use crate::entities::filters::ParticipantFilter;
use crate::http::path_id::decode_path_id;
use crate::modules::ParticipantModule;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use common::batch_requests::BatchRequestsAsString;
use common::paginated_spec::Paginated;
use common::query::{QueryFilter, QuerySpec};
use common::routes::auth::mates;
use serde::Deserialize;
use ymir::data::entities::shared::participant::{Model, Plan};
use ymir::errors::AppResult;
use ymir::types::oauth::{RoleTrait, UserInfo};
use ymir::types::participants::Visibility;
use ymir::utils::extract_payload;

pub type ParticipantQuery = QuerySpec<ParticipantFilter>;

/// Routes of the participant registry (`/mates`).
pub struct ParticipantRouter {
    manager: Arc<dyn ParticipantModule>,
}

impl ParticipantRouter {
    // ==========================================================================================
    // Sub-routers
    // ==========================================================================================

    pub fn new(mater: Arc<dyn ParticipantModule>) -> ParticipantRouter {
        ParticipantRouter { manager: mater }
    }

    /// Participant routes; mounted behind the OAuth guard.
    pub fn internal(&self) -> Router {
        Router::new()
            .route(mates::ALL, get(Self::get_all))
            .route(mates::MYSELF, get(Self::get_myself))
            .route(mates::BY_ID, get(Self::get_by_id).put(Self::update_by_id))
            .route(mates::BATCH, post(Self::get_batch))
            .route(mates::ROOT, post(Self::create))
            .route(mates::SYNC, post(Self::sync))
            .with_state(self.manager.clone())
    }

    // ==========================================================================================
    // Internal requests: users, behind the OAuth guard (queries)
    // ==========================================================================================

    async fn get_all(
        State(manager): State<Arc<dyn ParticipantModule>>,
        user: UserInfo,
        Query(query): Query<ParticipantQuery>,
    ) -> AppResult<Json<Paginated<Model>>> {
        query.filter.validate()?;
        Ok(Json(
            manager
                .get_all(&user, &query.filter, &query.page, &query.sort)
                .await?,
        ))
    }

    /// `id` comes in base64url (see `path_id`).
    async fn get_by_id(
        State(manager): State<Arc<dyn ParticipantModule>>,
        user: UserInfo,
        Path(id): Path<String>,
    ) -> AppResult<Json<Model>> {
        let id = decode_path_id(&id)?;
        Ok(Json(manager.get_by_id(&user, &id).await?))
    }

    async fn get_myself(
        State(manager): State<Arc<dyn ParticipantModule>>,
        user: UserInfo,
    ) -> AppResult<Json<Model>> {
        Ok(Json(manager.get_myself(&user).await?))
    }

    async fn get_batch(
        State(manager): State<Arc<dyn ParticipantModule>>,
        user: UserInfo,
        payload: Result<Json<BatchRequestsAsString>, JsonRejection>,
    ) -> AppResult<Json<Vec<Model>>> {
        let payload = extract_payload(payload)?;
        Ok(Json(manager.get_participant_batch(&user, payload).await?))
    }

    // ==========================================================================================
    // Internal requests: users, behind the OAuth guard (actions)
    // ==========================================================================================

    /// `id` comes in base64url (see `path_id`).
    async fn update_by_id(
        State(manager): State<Arc<dyn ParticipantModule>>,
        user: UserInfo,
        Path(id): Path<String>,
        payload: Result<Json<serde_json::Value>, JsonRejection>,
    ) -> AppResult<Json<Model>> {
        let id = decode_path_id(&id)?;
        let payload = extract_payload(payload)?;
        Ok(Json(
            manager
                .update_extra_fields_by_id(&user, &id, payload)
                .await?,
        ))
    }

    async fn create(
        State(manager): State<Arc<dyn ParticipantModule>>,
        user: UserInfo,
        payload: Result<Json<PartVis>, JsonRejection>,
    ) -> AppResult<Json<Model>> {
        let payload = extract_payload(payload)?;
        Ok(Json(
            manager
                .create_participant(&user, payload.plan, payload.visibility)
                .await?,
        ))
    }

    async fn sync(
        State(manager): State<Arc<dyn ParticipantModule>>,
        user: UserInfo,
    ) -> AppResult<Json<u64>> {
        user.require_root()?;
        Ok(Json(manager.sync_directory().await?))
    }
}

/// Body of `POST /mates`: the participant to add, with the visibility of the relation with it
/// as one more field.
#[derive(Debug, Deserialize)]
pub struct PartVis {
    #[serde(flatten)]
    plan: Plan,
    visibility: Visibility,
}
