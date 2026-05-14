use crate::entities::RefreshToken;
use thiserror::Error;
use uuid::Uuid;
use ymir::errors::{Outcome, RepoIntoErrors};

#[mockall::automock]
#[async_trait::async_trait]
pub trait RefreshTokenRepoTrait: Send + Sync {
    async fn create(&self, token: &RefreshToken) -> Outcome<RefreshToken>;
    async fn get_by_jti(&self, jti: &str) -> Outcome<Option<RefreshToken>>;
    async fn revoke(&self, id: Uuid) -> Outcome<()>;
    async fn revoke_all_for_tenant(&self, tenant_id: &str) -> Outcome<()>;
}

#[derive(Debug, Error)]
pub enum RefreshTokenRepoError {
    #[error("Token not found")]
    NotFound,
    #[error("Error accessing refresh tokens: {0}")]
    Db(Box<dyn std::error::Error + Send + Sync>),
}

impl RepoIntoErrors for RefreshTokenRepoError {}
