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
use ymir::errors::{AppResult, Errors};
use ymir::utils::extract_payload;

use crate::entities::commands::{CreateUserCommand, PatchUserCommand};
use crate::entities::query::{Paginated, UserQuery};
use crate::services::token_service::TokenServiceTrait;
use crate::services::user_service::UserServiceTrait;
use crate::services::user_service::views::UserView;

#[derive(Clone)]
pub(crate) struct UsersRouter {
    token_svc: Arc<dyn TokenServiceTrait>,
    user_svc: Arc<dyn UserServiceTrait>,
}

impl UsersRouter {
    pub(crate) fn new(
        token_svc: Arc<dyn TokenServiceTrait>,
        user_svc: Arc<dyn UserServiceTrait>,
    ) -> Self {
        Self {
            token_svc,
            user_svc,
        }
    }

    pub(crate) fn router(self) -> Router {
        let protected = Router::new()
            .route("/", get(Self::handle_list).post(Self::handle_create))
            .route(
                "/{id}",
                get(Self::handle_get_one)
                    .patch(Self::handle_patch)
                    .delete(Self::handle_delete),
            )
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
        Query(q): Query<UserQuery>,
    ) -> AppResult<(HeaderMap, Json<Paginated<UserView>>)> {
        let (filter, page, sort) = q.into_domain();
        let result = s.user_svc.list_users(&scope, &filter, &page, &sort).await?;
        let response_headers = headers.response_headers_paged(result.total);
        Ok((response_headers, Json(result)))
    }

    async fn handle_get_one(
        State(s): State<Self>,
        scope: AccessScope,
        Path(id): Path<String>,
    ) -> AppResult<Json<UserView>> {
        Ok(Json(s.user_svc.get_user(&scope, &id).await?))
    }

    async fn handle_create(
        State(s): State<Self>,
        scope: AccessScope,
        payload: Result<Json<CreateUserCommand>, JsonRejection>,
    ) -> AppResult<(StatusCode, Json<UserView>)> {
        let cmd = extract_payload(payload)?;
        Ok((
            StatusCode::CREATED,
            Json(s.user_svc.create_user(&scope, &cmd).await?),
        ))
    }

    async fn handle_patch(
        State(s): State<Self>,
        scope: AccessScope,
        Path(id): Path<String>,
        payload: Result<Json<PatchUserCommand>, JsonRejection>,
    ) -> AppResult<Json<UserView>> {
        let cmd = extract_payload(payload)?;
        Ok(Json(s.user_svc.patch_user(&scope, &id, &cmd).await?))
    }

    async fn handle_delete(
        State(s): State<Self>,
        scope: AccessScope,
        Path(id): Path<String>,
    ) -> AppResult<StatusCode> {
        s.user_svc.delete_user(&scope, &id).await?;
        Ok(StatusCode::NO_CONTENT)
    }
}
