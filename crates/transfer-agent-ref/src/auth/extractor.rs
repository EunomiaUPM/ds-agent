use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use oauth::services::token_service::Claims;
use ymir::errors::Errors;

/// Validated JWT claims injected by the auth middleware.
pub(crate) struct AuthClaims(pub Claims);

impl std::ops::Deref for AuthClaims {
    type Target = Claims;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S: Send + Sync> FromRequestParts<S> for AuthClaims {
    type Rejection = Errors;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Claims>()
            .cloned()
            .map(AuthClaims)
            .ok_or_else(|| Errors::crazy("auth middleware not applied to this route", None))
    }
}
