use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::HeaderMap;
use axum::middleware::Next;
use axum::response::Response;
use axum::{middleware, Router};
use ymir::errors::{AppResult, BadFormat, Errors, Outcome};

use oauth::services::token_service::TokenServiceTrait;

pub(crate) mod extractors;
pub(crate) mod transfer_message_router;
pub(crate) mod transfer_process_router;

pub(crate) fn build_router(
    token_svc: Arc<dyn TokenServiceTrait>,
    process_router: Router,
    message_router: Router,
) -> Router {
    Router::new()
        .merge(process_router)
        .merge(message_router)
        .route_layer(middleware::from_fn_with_state(token_svc, auth_middleware))
}

async fn auth_middleware(
    State(token_svc): State<Arc<dyn TokenServiceTrait>>,
    mut req: Request,
    next: Next,
) -> AppResult<Response> {
    let token = bearer(req.headers())?.to_owned();
    let claims = token_svc.validate_token(&token).await?;
    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}

fn bearer(headers: &HeaderMap) -> Outcome<&str> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| {
            Errors::format(BadFormat::Received, "missing or malformed Authorization header", None)
        })
}
