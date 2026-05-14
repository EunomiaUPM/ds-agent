use std::sync::Arc;

use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRef, Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::get;
use axum::{Json, Router};
use ymir::errors::{AppResult, BadFormat, Errors};
use ymir::utils::extract_payload;

use crate::entities::{Claims, CreateUserCommand, PatchUserCommand, Role, UserView};
use crate::services::AuthServiceTrait;

// Router ──────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct UsersRouter {
    service: Arc<dyn AuthServiceTrait>,
}

impl FromRef<UsersRouter> for Arc<dyn AuthServiceTrait> {
    fn from_ref(state: &UsersRouter) -> Self {
        state.service.clone()
    }
}

impl UsersRouter {
    pub fn new(service: Arc<dyn AuthServiceTrait>) -> Self {
        Self { service }
    }

    /// Mount under a prefix such as `/oauth/users`.
    ///
    /// | Method   | Path   | Admin | Owner       | Reader      |
    /// |----------|--------|-------|-------------|-------------|
    /// | GET      | `/`    | ✓ all | ✗           | ✗           |
    /// | GET      | `/{id}`| ✓ any | ✓ self only | ✓ self only |
    /// | POST     | `/`    | ✓     | ✗           | ✗           |
    /// | PATCH    | `/{id}`| ✓ any | ✓ self only | ✗           |
    /// | DELETE   | `/{id}`| ✓     | ✗           | ✗           |
    pub fn router(self) -> Router {
        Router::new()
            .route("/", get(Self::handle_list).post(Self::handle_create))
            .route(
                "/{id}",
                get(Self::handle_get_one)
                    .patch(Self::handle_patch)
                    .delete(Self::handle_delete),
            )
            .with_state(self)
    }

    // Handlers ─────────────────────────────────────────────────────────────

    /// `GET /` — list all users. Admin only.
    async fn handle_list(
        State(state): State<Self>,
        headers: HeaderMap,
    ) -> AppResult<Json<Vec<UserView>>> {
        let claims = auth(&state, &headers).await?;
        require_admin(&claims)?;
        Ok(Json(state.service.list_users().await?))
    }

    /// `GET /{id}` — get one user.
    /// Admin: any. Owner / Reader: self only.
    async fn handle_get_one(
        State(state): State<Self>,
        headers: HeaderMap,
        Path(id): Path<String>,
    ) -> AppResult<Json<UserView>> {
        let claims = auth(&state, &headers).await?;
        require_read_access(&claims, &id)?;
        Ok(Json(state.service.get_user(&id).await?))
    }

    /// `POST /` — create a new user. Admin only.
    async fn handle_create(
        State(state): State<Self>,
        headers: HeaderMap,
        payload: Result<Json<CreateUserCommand>, JsonRejection>,
    ) -> AppResult<(StatusCode, Json<UserView>)> {
        let claims = auth(&state, &headers).await?;
        require_admin(&claims)?;
        let cmd = extract_payload(payload)?;
        Ok((StatusCode::CREATED, Json(state.service.create_user(&cmd).await?)))
    }

    /// `PATCH /{id}` — partial update.
    /// Admin: any field on any user.
    /// Owner: own account only; `role` changes are rejected.
    /// Reader: forbidden.
    async fn handle_patch(
        State(state): State<Self>,
        headers: HeaderMap,
        Path(id): Path<String>,
        payload: Result<Json<PatchUserCommand>, JsonRejection>,
    ) -> AppResult<Json<UserView>> {
        let claims = auth(&state, &headers).await?;
        let cmd = extract_payload(payload)?;

        match claims.role {
            Role::Admin => {}
            Role::Owner if claims.sub == id => {
                if cmd.role.is_some() {
                    return Err(Errors::format(
                        BadFormat::Received,
                        "forbidden: only admins can change a user's role",
                        None,
                    ));
                }
            }
            _ => return Err(forbidden()),
        }

        Ok(Json(state.service.patch_user(&id, &cmd).await?))
    }

    /// `DELETE /{id}` — delete a user. Admin only.
    async fn handle_delete(
        State(state): State<Self>,
        headers: HeaderMap,
        Path(id): Path<String>,
    ) -> AppResult<StatusCode> {
        let claims = auth(&state, &headers).await?;
        require_admin(&claims)?;
        state.service.delete_user(&id).await?;
        Ok(StatusCode::NO_CONTENT)
    }
}

// RBAC helpers ─────────────────────────────────────────────────────────────

/// Extracts and validates the bearer token, returning the decoded claims.
async fn auth(state: &UsersRouter, headers: &HeaderMap) -> Outcome<Claims> {
    let token = extract_bearer(headers)?;
    state.service.validate_token(token).await
}

/// Admin can access anything. Owner/Reader can only access their own resource.
fn require_read_access(claims: &Claims, target_tenant_id: &str) -> Outcome<()> {
    match claims.role {
        Role::Admin => Ok(()),
        _ if claims.sub == target_tenant_id => Ok(()),
        _ => Err(forbidden()),
    }
}

fn require_admin(claims: &Claims) -> Outcome<()> {
    if claims.role == Role::Admin { Ok(()) } else { Err(forbidden()) }
}

fn forbidden() -> Errors {
    Errors::format(BadFormat::Received, "forbidden: insufficient permissions", None)
}

fn extract_bearer(headers: &HeaderMap) -> Outcome<&str> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| {
            Errors::format(BadFormat::Received, "missing or malformed Authorization header", None)
        })
}

type Outcome<T> = ymir::errors::Outcome<T>;
