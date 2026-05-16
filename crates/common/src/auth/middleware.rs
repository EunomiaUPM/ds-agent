use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::HeaderMap;
use axum::middleware::Next;
use axum::response::Response;
use ymir::errors::{AppResult, BadFormat, Errors, Outcome};

use crate::auth::claims::Claims;

/// Implemented by any service that can validate a bearer token and return decoded claims.
#[async_trait::async_trait]
pub trait TokenValidator: Send + Sync + 'static {
    async fn validate_token(&self, token: &str) -> Outcome<Claims>;
}

/// Axum middleware: extracts the bearer token, validates it, and inserts [`Claims`] into
/// request extensions. Must be applied before any handler that calls [`super::rbac::Rbac`].
pub async fn auth_middleware(
    State(validator): State<Arc<dyn TokenValidator>>,
    mut req: Request,
    next: Next,
) -> AppResult<Response> {
    let token = bearer(req.headers())?.to_owned();
    let claims = validator.validate_token(&token).await?;
    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}

pub fn bearer(headers: &HeaderMap) -> Outcome<&str> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| {
            Errors::format(
                BadFormat::Received,
                "missing or malformed Authorization header",
                None,
            )
        })
}
