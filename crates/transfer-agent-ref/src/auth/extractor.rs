use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use oauth::entities::Claims;
use ymir::errors::{BadFormat, Errors};

/// Validated JWT claims extracted from `Authorization: Bearer <token>`.
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
        let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_default();

        let bearer = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or_else(|| {
                Errors::format(BadFormat::Received, "missing or malformed Authorization header", None)
            })?;

        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;
        let claims = jsonwebtoken::decode::<Claims>(
            bearer,
            &DecodingKey::from_secret(jwt_secret.as_bytes()),
            &validation,
        )
        .map(|d| d.claims)
        .map_err(|e| {
            Errors::format(BadFormat::Received, "invalid or expired token", Some(Box::new(e)))
        })?;

        Ok(AuthClaims(claims))
    }
}
