use std::sync::Arc;

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use chrono::Utc;
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::data::repo::user_repo::UserRepoTrait;
use crate::entities::commands::{CreateUserCommand, PatchUserCommand};
use crate::entities::user::User;
use crate::services::user_service::views::{UserInfo, UserView};
use crate::services::user_service::UserServiceTrait;

pub(crate) struct UserService {
    user_repo: Arc<dyn UserRepoTrait>,
}

impl UserService {
    pub fn new(user_repo: Arc<dyn UserRepoTrait>) -> Self {
        Self { user_repo }
    }
}

#[async_trait::async_trait]
impl UserServiceTrait for UserService {
    async fn list_users(&self) -> Outcome<Vec<UserView>> {
        Ok(self.user_repo.get_all().await?.into_iter().map(UserView::assemble).collect())
    }

    async fn get_user(&self, tenant_id: &str) -> Outcome<UserView> {
        self.user_repo
            .get_by_tenant_id(tenant_id)
            .await?
            .map(UserView::assemble)
            .ok_or_else(|| Errors::format(BadFormat::Received, "user not found", None))
    }

    async fn userinfo(&self, tenant_id: &str) -> Outcome<UserInfo> {
        self.user_repo
            .get_by_tenant_id(tenant_id)
            .await?
            .map(UserInfo::assemble)
            .ok_or_else(|| Errors::format(BadFormat::Received, "user not found", None))
    }

    async fn create_user(&self, cmd: &CreateUserCommand) -> Outcome<UserView> {
        if self.user_repo.get_by_tenant_id(&cmd.tenant_id).await?.is_some() {
            return Err(Errors::format(BadFormat::Received, "tenant_id already in use", None));
        }
        if self.user_repo.get_by_email(&cmd.email).await?.is_some() {
            return Err(Errors::format(BadFormat::Received, "email already in use", None));
        }
        let user = User {
            tenant_id: cmd.tenant_id.clone(),
            email: cmd.email.clone(),
            password_hash: hash_password(&cmd.password)?,
            role: cmd.role,
            created_at: Utc::now(),
            extra_fields: cmd.extra_fields.clone(),
        };
        Ok(UserView::assemble(self.user_repo.create(&user).await?))
    }

    async fn patch_user(&self, tenant_id: &str, cmd: &PatchUserCommand) -> Outcome<UserView> {
        Ok(UserView::assemble(
            self.user_repo
                .patch(tenant_id, cmd.email.clone(), cmd.role, cmd.extra_fields.clone())
                .await?,
        ))
    }

    async fn delete_user(&self, tenant_id: &str) -> Outcome<()> {
        self.user_repo.delete(tenant_id).await
    }
}

fn hash_password(password: &str) -> Outcome<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| Errors::crazy("password hashing failed", Some(e.to_string().into())))
}
