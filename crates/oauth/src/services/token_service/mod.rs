use ymir::errors::Outcome;

pub(crate) mod jwt;
pub(crate) mod service;
pub(crate) mod views;

// Re-export the canonical types from common so existing consumers keep working.
pub use common::auth::claims::Claims;
pub use common::auth::middleware::TokenValidator;

// Service trait ────────────────────────────────────────────────────────────

#[async_trait::async_trait]
pub trait TokenServiceTrait: TokenValidator + Send + Sync + 'static {
    async fn issue_token(&self, email: &str, password: &str) -> Outcome<views::TokenResponse>;
    async fn refresh_token(&self, refresh_jwt: &str) -> Outcome<views::TokenResponse>;
    async fn revoke_refresh_token(&self, refresh_jwt: &str) -> Outcome<()>;
}
