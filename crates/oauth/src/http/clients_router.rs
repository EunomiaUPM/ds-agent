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
use ymir::errors::AppResult;
use ymir::utils::extract_payload;

use crate::entities::commands::CreateClientCommand;
use crate::entities::query::{ClientQuery, Paginated};
use crate::services::client_service::ClientServiceTrait;
use crate::services::client_service::views::ClientView;

#[derive(Clone)]
pub(crate) struct ClientsRouter {
    client_svc: Arc<dyn ClientServiceTrait>,
}

impl ClientsRouter {
    pub(crate) fn new(client_svc: Arc<dyn ClientServiceTrait>) -> Self {
        Self { client_svc }
    }

    pub(crate) fn router(self) -> Router {
        Router::new()
            .route("/", get(Self::handle_list).post(Self::handle_create))
            .route(
                "/{id}",
                get(Self::handle_get_one).delete(Self::handle_delete),
            )
            .with_state(self)
    }

    async fn handle_list(
        State(s): State<Self>,
        scope: AccessScope,
        headers: ExtractedHeaders,
        Query(q): Query<ClientQuery>,
    ) -> AppResult<(HeaderMap, Json<Paginated<ClientView>>)> {
        let (filter, page, sort) = q.into_domain();
        let result = s
            .client_svc
            .list_clients(&scope, &filter, &page, &sort)
            .await?;
        let response_headers = headers.response_headers_paged(result.total);
        Ok((response_headers, Json(result)))
    }

    async fn handle_get_one(
        State(s): State<Self>,
        scope: AccessScope,
        Path(id): Path<String>,
    ) -> AppResult<Json<ClientView>> {
        Ok(Json(s.client_svc.get_client(&scope, &id).await?))
    }

    async fn handle_create(
        State(s): State<Self>,
        scope: AccessScope,
        payload: Result<Json<CreateClientCommand>, JsonRejection>,
    ) -> AppResult<(StatusCode, Json<ClientView>)> {
        let cmd = extract_payload(payload)?;
        Ok((
            StatusCode::CREATED,
            Json(s.client_svc.create_client(&scope, &cmd).await?),
        ))
    }

    async fn handle_delete(
        State(s): State<Self>,
        scope: AccessScope,
        Path(id): Path<String>,
    ) -> AppResult<StatusCode> {
        s.client_svc.delete_client(&scope, &id).await?;
        Ok(StatusCode::NO_CONTENT)
    }
}
