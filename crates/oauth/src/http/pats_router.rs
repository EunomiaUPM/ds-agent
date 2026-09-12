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
use axum::extract::{Extension, Path, Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use axum::routing::get;
use axum::{Json, Router, middleware};
use common::auth::claims::Claims;
use common::auth::middleware::bearer;
use uuid::Uuid;
use ymir::errors::AppResult;
use ymir::utils::extract_payload;

use crate::entities::commands::CreatePatCommand;
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
        let token = bearer(req.headers())?.to_owned();
        let claims = s.token_svc.validate_token(&token).await?;
        req.extensions_mut().insert(claims);
        Ok(next.run(req).await)
    }

    async fn handle_list(
        State(s): State<Self>,
        Extension(claims): Extension<Claims>,
    ) -> AppResult<Json<Vec<PatView>>> {
        Ok(Json(s.pat_svc.list_pats(&claims.sub).await?))
    }

    async fn handle_create(
        State(s): State<Self>,
        Extension(claims): Extension<Claims>,
        payload: Result<Json<CreatePatCommand>, JsonRejection>,
    ) -> AppResult<(StatusCode, Json<CreatePatResponse>)> {
        let cmd = extract_payload(payload)?;
        let pat = s
            .pat_svc
            .create_pat(
                &claims.sub,
                &cmd.name,
                claims.role,
                cmd.scopes,
                cmd.expires_at,
            )
            .await?;
        Ok((StatusCode::CREATED, Json(pat)))
    }

    async fn handle_delete(
        State(s): State<Self>,
        Extension(claims): Extension<Claims>,
        Path(id): Path<Uuid>,
    ) -> AppResult<StatusCode> {
        s.pat_svc.revoke_pat(&claims.sub, id).await?;
        Ok(StatusCode::NO_CONTENT)
    }
}
