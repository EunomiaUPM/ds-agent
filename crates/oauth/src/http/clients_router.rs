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
use common::auth::rbac::Rbac;
use serde::Deserialize;
use ymir::errors::AppResult;
use ymir::utils::extract_payload;

use crate::entities::commands::CreateClientCommand;
use crate::http::forms::ClientView;
use crate::services::client_service::ClientServiceTrait;
use crate::services::token_service::TokenServiceTrait;

// Query parameters for listing and filtering registered OAuth clients.
#[derive(Debug, Deserialize, Default)]
pub struct ListClientsQuery {
    pub limit: Option<u64>,
    pub page: Option<u64>,
    pub offset: Option<u64>,
    pub role: Option<String>,
    pub search: Option<String>,
    pub sort: Option<String>,
}

#[derive(Clone)]
pub(crate) struct ClientsRouter {
    token_svc: Arc<dyn TokenServiceTrait>,
    client_svc: Arc<dyn ClientServiceTrait>,
}

impl ClientsRouter {
    pub(crate) fn new(
        token_svc: Arc<dyn TokenServiceTrait>,
        client_svc: Arc<dyn ClientServiceTrait>,
    ) -> Self {
        Self {
            token_svc,
            client_svc,
        }
    }

    pub(crate) fn router(self) -> Router {
        let protected = Router::new()
            .route("/", get(Self::handle_list).post(Self::handle_create))
            .route(
                "/{id}",
                get(Self::handle_get_one).delete(Self::handle_delete),
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
        let token = bearer(req.headers())?.to_owned();
        let claims = s.token_svc.validate_token(&token).await?;
        req.extensions_mut().insert(claims);
        Ok(next.run(req).await)
    }

    async fn handle_list(
        State(s): State<Self>,
        Extension(claims): Extension<Claims>,
        Query(q): Query<ListClientsQuery>,
    ) -> AppResult<Json<Vec<ClientView>>> {
        Rbac::require_admin(&claims)?;
        let mut clients = s.client_svc.list_clients().await?;

        if let Some(ref role) = q.role {
            if role != "all" {
                clients.retain(|c| c.role.to_string().eq_ignore_ascii_case(role));
            }
        }
        if let Some(ref search) = q.search {
            let term = search.to_lowercase();
            clients.retain(|c| {
                c.client_name.to_lowercase().contains(&term)
                    || c.client_id.to_lowercase().contains(&term)
            });
        }
        if let Some(ref sort) = q.sort {
            match sort.as_str() {
                "created_at_asc" => clients.sort_by(|a, b| a.created_at.cmp(&b.created_at)),
                "created_at_desc" => clients.sort_by(|a, b| b.created_at.cmp(&a.created_at)),
                "client_name_asc" => clients.sort_by(|a, b| a.client_name.to_lowercase().cmp(&b.client_name.to_lowercase())),
                "client_name_desc" => clients.sort_by(|a, b| b.client_name.to_lowercase().cmp(&a.client_name.to_lowercase())),
                "client_id_asc" => clients.sort_by(|a, b| a.client_id.cmp(&b.client_id)),
                "client_id_desc" => clients.sort_by(|a, b| b.client_id.cmp(&a.client_id)),
                "role_asc" => clients.sort_by(|a, b| a.role.to_string().cmp(&b.role.to_string())),
                "role_desc" => clients.sort_by(|a, b| b.role.to_string().cmp(&a.role.to_string())),
                _ => clients.sort_by(|a, b| b.created_at.cmp(&a.created_at)),
            }
        } else {
            clients.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        }
        if let Some(limit) = q.limit {
            let offset = q
                .offset
                .or_else(|| q.page.map(|p| p.saturating_sub(1) * limit))
                .unwrap_or(0) as usize;
            let paged: Vec<ClientView> = clients
                .into_iter()
                .skip(offset)
                .take(limit as usize)
                .collect();
            Ok(Json(paged))
        } else {
            Ok(Json(clients))
        }
    }

    async fn handle_get_one(
        State(s): State<Self>,
        Extension(claims): Extension<Claims>,
        Path(id): Path<String>,
    ) -> AppResult<Json<ClientView>> {
        Rbac::require_read(&claims, &id)?;
        Ok(Json(s.client_svc.get_client(&id).await?))
    }

    async fn handle_create(
        State(s): State<Self>,
        Extension(claims): Extension<Claims>,
        payload: Result<Json<CreateClientCommand>, JsonRejection>,
    ) -> AppResult<(StatusCode, Json<ClientView>)> {
        Rbac::require_admin(&claims)?;
        let cmd = extract_payload(payload)?;
        Ok((
            StatusCode::CREATED,
            Json(s.client_svc.create_client(&cmd).await?),
        ))
    }

    async fn handle_delete(
        State(s): State<Self>,
        Extension(claims): Extension<Claims>,
        Path(id): Path<String>,
    ) -> AppResult<StatusCode> {
        Rbac::require_admin(&claims)?;
        s.client_svc.delete_client(&id).await?;
        Ok(StatusCode::NO_CONTENT)
    }
}
