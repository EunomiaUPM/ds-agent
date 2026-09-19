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

use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::get;
use axum::{Json, Router, middleware};
use common::auth::AccessScope;
use common::auth::http::ExtractedHeaders;
use uuid::Uuid;
use ymir::errors::AppResult;
use ymir::utils::extract_payload;

use crate::entities::commands::CreatePatCommand;
use crate::entities::query::{Paginated, PatQuery};
use crate::services::pat_service::PatServiceTrait;
use crate::services::pat_service::views::{CreatePatResponse, PatView};

#[derive(Clone)]
pub(crate) struct PatsRouter {
    pat_svc: Arc<dyn PatServiceTrait>,
}

impl PatsRouter {
    pub(crate) fn new(pat_svc: Arc<dyn PatServiceTrait>) -> Self {
        Self { pat_svc }
    }

    pub(crate) fn router(self) -> Router {
        Router::new()
            .route("/", get(Self::handle_list).post(Self::handle_create))
            .route("/{id}", axum::routing::delete(Self::handle_delete))
            .with_state(self)
    }

    async fn handle_list(
        State(s): State<Self>,
        scope: AccessScope,
        headers: ExtractedHeaders,
        Query(q): Query<PatQuery>,
    ) -> AppResult<(HeaderMap, Json<Paginated<PatView>>)> {
        let (filter, page, sort) = q.into_domain();
        let result = s.pat_svc.list_pats(&scope, &filter, &page, &sort).await?;
        let response_headers = headers.response_headers_paged(result.total);
        Ok((response_headers, Json(result)))
    }

    async fn handle_create(
        State(s): State<Self>,
        scope: AccessScope,
        payload: Result<Json<CreatePatCommand>, JsonRejection>,
    ) -> AppResult<(StatusCode, Json<CreatePatResponse>)> {
        let cmd = extract_payload(payload)?;
        let pat = s
            .pat_svc
            .create_pat(
                &scope,
                &cmd.name,
                scope.role(),
                cmd.scopes,
                cmd.expires_at,
            )
            .await?;
        Ok((StatusCode::CREATED, Json(pat)))
    }

    async fn handle_delete(
        State(s): State<Self>,
        scope: AccessScope,
        Path(id): Path<Uuid>,
    ) -> AppResult<StatusCode> {
        s.pat_svc.revoke_pat(&scope, id).await?;
        Ok(StatusCode::NO_CONTENT)
    }
}
