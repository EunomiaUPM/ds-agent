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
use axum::extract::{Extension, Path, Query, Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use axum::routing::get;
use axum::{Json, Router, middleware};
use common::auth::claims::Claims;
use common::auth::middleware::bearer;
use serde::Deserialize;
use uuid::Uuid;
use ymir::errors::AppResult;
use ymir::utils::extract_payload;

use crate::entities::commands::CreatePatCommand;
use crate::services::pat_service::PatServiceTrait;
use crate::services::pat_service::views::{CreatePatResponse, PatView};
use crate::services::token_service::TokenServiceTrait;

#[derive(Debug, Deserialize, Default)]
pub struct ListPatsQuery {
    pub limit: Option<u64>,
    pub page: Option<u64>,
    pub offset: Option<u64>,
    pub role: Option<String>,
    pub status: Option<String>,
    pub search: Option<String>,
    pub sort: Option<String>,
}

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
        Query(q): Query<ListPatsQuery>,
    ) -> AppResult<Json<Vec<PatView>>> {
        let mut pats = s.pat_svc.list_pats(&claims.sub).await?;

        if let Some(ref status) = q.status {
            if status == "active" {
                pats.retain(|p| !p.revoked);
            } else if status == "revoked" {
                pats.retain(|p| p.revoked);
            }
        }
        if let Some(ref role) = q.role {
            if role != "all" {
                pats.retain(|p| p.role.to_string().eq_ignore_ascii_case(role));
            }
        }
        if let Some(ref search) = q.search {
            let term = search.to_lowercase();
            pats.retain(|p| {
                p.name.to_lowercase().contains(&term)
                    || p.token_prefix.to_lowercase().contains(&term)
                    || p.role.to_string().to_lowercase().contains(&term)
            });
        }
        if let Some(ref sort) = q.sort {
            match sort.as_str() {
                "created_at_asc" => pats.sort_by(|a, b| a.created_at.cmp(&b.created_at)),
                "created_at_desc" => pats.sort_by(|a, b| b.created_at.cmp(&a.created_at)),
                "name_asc" => pats.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase())),
                "name_desc" => pats.sort_by(|a, b| b.name.to_lowercase().cmp(&a.name.to_lowercase())),
                "token_prefix_asc" => pats.sort_by(|a, b| a.token_prefix.cmp(&b.token_prefix)),
                "token_prefix_desc" => pats.sort_by(|a, b| b.token_prefix.cmp(&a.token_prefix)),
                "expires_at_asc" => pats.sort_by(|a, b| a.expires_at.cmp(&b.expires_at)),
                "expires_at_desc" => pats.sort_by(|a, b| b.expires_at.cmp(&a.expires_at)),
                _ => pats.sort_by(|a, b| b.created_at.cmp(&a.created_at)),
            }
        } else {
            pats.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        }
        if let Some(limit) = q.limit {
            let offset = q
                .offset
                .or_else(|| q.page.map(|p| p.saturating_sub(1) * limit))
                .unwrap_or(0) as usize;
            let paged: Vec<PatView> = pats.into_iter().skip(offset).take(limit as usize).collect();
            Ok(Json(paged))
        } else {
            Ok(Json(pats))
        }
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
