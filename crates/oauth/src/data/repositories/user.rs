use thiserror::Error;
use ymir::errors::{Outcome, RepoIntoErrors};

use crate::entities::query::{Page, Sort, UserFilter};
use crate::entities::role::Role;
use crate::entities::user::User;

#[mockall::automock]
#[async_trait::async_trait]
pub(crate) trait UserRepository: Send + Sync {
    async fn get_all(&self, filter: &UserFilter, page: &Page, sort: &Sort) -> Outcome<Vec<User>>;
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
pub(crate) enum UserRepositoryError {
    #[error("user not found")]
    NotFound,
    #[error("user already exists")]
    AlreadyExists,
    #[error("database error: {0}")]
    Db(Box<dyn std::error::Error + Send + Sync>),
}

impl RepoIntoErrors for UserRepositoryError {}
