/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

use std::sync::Arc;

use base64::Engine;
use chrono::Utc;
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::data::repositories::user::UserRepository;
use crate::entities::commands::{CreateUserCommand, PatchUserCommand};
use crate::entities::query::{Page, Paginated, Sort, UserFilter};
use crate::entities::user::User;
use crate::services::password;
use crate::services::user_service::UserServiceTrait;
use crate::services::user_service::views::{UserInfo, UserView};

pub(crate) struct UserService {
    user_repo: Arc<dyn UserRepository>,
}

impl UserService {
    pub fn new(user_repo: Arc<dyn UserRepository>) -> Self {
        Self { user_repo }
    }
}

#[async_trait::async_trait]
impl UserServiceTrait for UserService {
    async fn list_users(
        &self,
        filter: &UserFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<UserView>> {
        let fetch_page = Page {
            limit: page.limit + 1,
            cursor: page.cursor.clone(),
        };
        let mut users = self.user_repo.get_all(filter, &fetch_page, sort).await?;

        let has_more = users.len() > page.limit as usize;
        if has_more {
            users.truncate(page.limit as usize);
        }

        let next_cursor = if has_more {
            users.last().map(|u| {
                base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(u.created_at.to_rfc3339())
            })
        } else {
            None
        };

        Ok(Paginated {
            items: users.into_iter().map(UserView::assemble).collect(),
            next_cursor,
            total: None,
        })
    }

    async fn get_user(&self, tenant_id: &str) -> Outcome<UserView> {
        self.user_repo
            .get_by_tenant_id(tenant_id)
            .await?
            .map(UserView::assemble)
            .ok_or_else(|| Errors::format(BadFormat::Received, "user not found", None))
    }

    /// A convenience method for token service management
    /// Only difference to `get_user` is the [`UserInfo`] struct
    async fn user_info(&self, tenant_id: &str) -> Outcome<UserInfo> {
        self.user_repo
            .get_by_tenant_id(tenant_id)
            .await?
            .map(UserInfo::assemble)
            .ok_or_else(|| Errors::format(BadFormat::Received, "user not found", None))
    }

    async fn create_user(&self, cmd: &CreateUserCommand) -> Outcome<UserView> {
        if self
            .user_repo
            .get_by_tenant_id(&cmd.tenant_id)
            .await?
            .is_some()
        {
            return Err(Errors::format(
                BadFormat::Received,
                "tenant_id already in use",
                None,
            ));
        }
        if self.user_repo.get_by_email(&cmd.email).await?.is_some() {
            return Err(Errors::format(
                BadFormat::Received,
                "email already in use",
                None,
            ));
        }
        let (password_hash, password_salt) = password::hash_password(&cmd.password)?;
        let user = User {
            tenant_id: cmd.tenant_id.clone(),
            email: cmd.email.clone(),
            password_hash,
            password_salt,
            role: cmd.role,
            created_at: Utc::now(),
            extra_fields: cmd.extra_fields.clone(),
        };
        Ok(UserView::assemble(self.user_repo.create(&user).await?))
    }

    async fn patch_user(&self, tenant_id: &str, cmd: &PatchUserCommand) -> Outcome<UserView> {
        Ok(UserView::assemble(
            self.user_repo
                .patch(
                    tenant_id,
                    cmd.email.clone(),
                    cmd.role,
                    cmd.extra_fields.clone(),
                )
                .await?,
        ))
    }

    async fn delete_user(&self, tenant_id: &str) -> Outcome<()> {
        self.user_repo.delete(tenant_id).await
    }
}
