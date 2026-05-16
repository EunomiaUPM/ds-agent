use ymir::errors::Outcome;

use crate::entities::commands::{CreateUserCommand, PatchUserCommand};
use crate::entities::query::{Page, Paginated, Sort, UserFilter};
use crate::services::user_service::views::{UserInfo, UserView};

pub(crate) mod service;
pub(crate) mod views;

// Service trait ────────────────────────────────────────────────────────────

#[async_trait::async_trait]
pub(crate) trait UserServiceTrait: Send + Sync + 'static {
    async fn list_users(
        &self,
        filter: &UserFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<UserView>>;
    async fn get_user(&self, tenant_id: &str) -> Outcome<UserView>;
    async fn userinfo(&self, tenant_id: &str) -> Outcome<UserInfo>;
    async fn create_user(&self, cmd: &CreateUserCommand) -> Outcome<UserView>;
    async fn patch_user(&self, tenant_id: &str, cmd: &PatchUserCommand) -> Outcome<UserView>;
    async fn delete_user(&self, tenant_id: &str) -> Outcome<()>;
}
