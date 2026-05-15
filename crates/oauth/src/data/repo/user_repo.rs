use crate::entities::role::Role;
use crate::entities::user::User;
use thiserror::Error;
use ymir::errors::{Outcome, RepoIntoErrors};

#[mockall::automock]
#[async_trait::async_trait]
pub trait UserRepoTrait: Send + Sync {
    async fn get_all(&self) -> Outcome<Vec<User>>;
    async fn get_by_tenant_id(&self, tenant_id: &str) -> Outcome<Option<User>>;
    async fn get_by_email(&self, email: &str) -> Outcome<Option<User>>;
    async fn create(&self, user: &User) -> Outcome<User>;
    async fn patch(
        &self,
        tenant_id: &str,
        email: Option<String>,
        role: Option<Role>,
        extra_fields: Option<serde_json::Value>,
    ) -> Outcome<User>;
    async fn delete(&self, tenant_id: &str) -> Outcome<()>;
}

#[derive(Debug, Error)]
pub enum UserRepoError {
    #[error("User not found")]
    NotFound,
    #[error("User already exists")]
    AlreadyExists,
    #[error("Error accessing users: {0}")]
    Db(Box<dyn std::error::Error + Send + Sync>),
}

impl RepoIntoErrors for UserRepoError {}
