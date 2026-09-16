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
use axum::extract::{Path, Query, Request, State};
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use axum::routing::get;
use axum::{Json, Router, middleware};
use common::auth::AccessScope;
use common::auth::http::{AuthHttpMiddleware, ExtractedHeaders};
use common::auth::validators::AuthValidators;
use uuid::Uuid;
use ymir::errors::{AppResult, Errors};
use ymir::utils::extract_payload;

use crate::entities::commands::CreatePatCommand;
use crate::entities::query::{Paginated, PatQuery};
use crate::services::pat_service::PatServiceTrait;
use crate::services::pat_service::views::{CreatePatResponse, PatView};
use crate::services::token_service::TokenServiceTrait;

#[derive(Clone)]
pub(crate) struct PatsRouter {
    token_svc: Arc<dyn TokenServiceTrait>,
    pat_svc: Arc<dyn PatServiceTrait>,
}

impl PatsRouter {
    pub(crate) fn new(
        token_svc: Arc<dyn TokenServiceTrait>,
        pat_svc: Arc<dyn PatServiceTrait>,
    ) -> Self {
        Self { token_svc, pat_svc }
    }

    pub(crate) fn router(self) -> Router {
        let protected = Router::new()
            .route("/", get(Self::handle_list).post(Self::handle_create))
            .route("/{id}", axum::routing::delete(Self::handle_delete))
            .route_layer(middleware::from_fn_with_state(
                self.clone(),
                Self::auth_middleware,
            ));

        Router::new().merge(protected).with_state(self)
    }

    async fn auth_middleware(
        State(s): State<Self>,
        mut req: Request,
        next: Next,
    ) -> AppResult<Response> {
        let token = AuthHttpMiddleware::bearer(req.headers())?.to_owned();
        let claims = s.token_svc.validate_token(&token).await?;
        AuthValidators::claims_validator()
            .validate(&claims)
            .map_err(|vs| Errors::unauthorized(vs.to_string(), None))?;
        req.extensions_mut().insert(claims);
        let mut resp = next.run(req).await;
        AuthHttpMiddleware::apply_security_headers(&mut resp);
        Ok(resp)
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
