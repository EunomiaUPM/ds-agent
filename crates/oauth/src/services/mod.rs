use crate::entities::{Claims, CreateUserCommand, PatchUserCommand, TokenResponse, UserInfo, UserView};
use ymir::errors::Outcome;

pub mod auth_service;

#[async_trait::async_trait]
pub trait AuthServiceTrait: Send + Sync + 'static {
    // Auth / token ────────────────────────────────────────────────────────
    async fn issue_token(&self, email: &str, password: &str) -> Outcome<TokenResponse>;
    async fn refresh_token(&self, refresh_token: &str) -> Outcome<TokenResponse>;
    async fn validate_token(&self, access_token: &str) -> Outcome<Claims>;
    async fn userinfo(&self, access_token: &str) -> Outcome<UserInfo>;
    async fn revoke_refresh_token(&self, refresh_token: &str) -> Outcome<()>;

    // User CRUD ───────────────────────────────────────────────────────────
    async fn list_users(&self) -> Outcome<Vec<UserView>>;
    async fn get_user(&self, tenant_id: &str) -> Outcome<UserView>;
    async fn create_user(&self, cmd: &CreateUserCommand) -> Outcome<UserView>;
    /// Applies a partial update. The HTTP layer strips `role` for non-admin callers.
    async fn patch_user(&self, tenant_id: &str, cmd: &PatchUserCommand) -> Outcome<UserView>;
    async fn delete_user(&self, tenant_id: &str) -> Outcome<()>;
}
