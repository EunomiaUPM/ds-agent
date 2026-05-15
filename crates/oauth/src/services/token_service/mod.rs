use ymir::errors::Outcome;

use crate::entities::role::Role;

pub(crate) mod jwt;
pub(crate) mod service;
pub(crate) mod views;

// ── Public contract ─────────────────────────────────────────────────────────

/// Decoded access-token claims. Returned by `validate_token` and used by the
/// HTTP layer for RBAC decisions.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: Role,
    pub iat: i64,
    pub exp: i64,
}

// ── Service trait ───────────────────────────────────────────────────────────

#[async_trait::async_trait]
pub trait TokenServiceTrait: Send + Sync + 'static {
    async fn issue_token(&self, email: &str, password: &str) -> Outcome<views::TokenResponse>;
    async fn refresh_token(&self, refresh_jwt: &str) -> Outcome<views::TokenResponse>;
    async fn validate_token(&self, access_token: &str) -> Outcome<Claims>;
    async fn revoke_refresh_token(&self, refresh_jwt: &str) -> Outcome<()>;
}
