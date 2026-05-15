use std::sync::Arc;

use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::get;
use axum::{Json, Router};
use ymir::errors::{AppResult, BadFormat, Errors, Outcome};
use ymir::utils::extract_payload;

use crate::entities::commands::{CreateUserCommand, PatchUserCommand};
use crate::entities::role::Role;
use crate::http::helpers::bearer;
use crate::services::token_service::{Claims, TokenServiceTrait};
use crate::services::user_service::views::UserView;
use crate::services::user_service::UserServiceTrait;

#[derive(Clone)]
pub(crate) struct UsersRouter {
    token_svc: Arc<dyn TokenServiceTrait>,
    user_svc: Arc<dyn UserServiceTrait>,
}

impl UsersRouter {
    pub(crate) fn new(token_svc: Arc<dyn TokenServiceTrait>, user_svc: Arc<dyn UserServiceTrait>) -> Self {
        Self { token_svc, user_svc }
    }

    pub(crate) fn router(self) -> Router {
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

    async fn auth(&self, headers: &HeaderMap) -> Outcome<Claims> {
        self.token_svc.validate_token(bearer(headers)?).await
    }

    async fn handle_list(State(s): State<Self>, headers: HeaderMap) -> AppResult<Json<Vec<UserView>>> {
        let c = s.auth(&headers).await?;
        require_admin(&c)?;
        Ok(Json(s.user_svc.list_users().await?))
    }

    async fn handle_get_one(
        State(s): State<Self>,
        headers: HeaderMap,
        Path(id): Path<String>,
    ) -> AppResult<Json<UserView>> {
        let c = s.auth(&headers).await?;
        require_read_access(&c, &id)?;
        Ok(Json(s.user_svc.get_user(&id).await?))
    }

    async fn handle_create(
        State(s): State<Self>,
        headers: HeaderMap,
        payload: Result<Json<CreateUserCommand>, JsonRejection>,
    ) -> AppResult<(StatusCode, Json<UserView>)> {
        let c = s.auth(&headers).await?;
        require_admin(&c)?;
        let cmd = extract_payload(payload)?;
        Ok((StatusCode::CREATED, Json(s.user_svc.create_user(&cmd).await?)))
    }

    async fn handle_patch(
        State(s): State<Self>,
        headers: HeaderMap,
        Path(id): Path<String>,
        payload: Result<Json<PatchUserCommand>, JsonRejection>,
    ) -> AppResult<Json<UserView>> {
        let c = s.auth(&headers).await?;
        let cmd = extract_payload(payload)?;

        match c.role {
            Role::Admin => {}
            Role::Owner if c.sub == id => {
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

        Ok(Json(s.user_svc.patch_user(&id, &cmd).await?))
    }

    async fn handle_delete(
        State(s): State<Self>,
        headers: HeaderMap,
        Path(id): Path<String>,
    ) -> AppResult<StatusCode> {
        let c = s.auth(&headers).await?;
        require_admin(&c)?;
        s.user_svc.delete_user(&id).await?;
        Ok(StatusCode::NO_CONTENT)
    }
}

fn require_read_access(c: &Claims, target: &str) -> Outcome<()> {
    match c.role {
        Role::Admin => Ok(()),
        _ if c.sub == target => Ok(()),
        _ => Err(forbidden()),
    }
}

fn require_admin(c: &Claims) -> Outcome<()> {
    if c.role == Role::Admin { Ok(()) } else { Err(forbidden()) }
}

fn forbidden() -> Errors {
    Errors::format(BadFormat::Received, "forbidden: insufficient permissions", None)
}
