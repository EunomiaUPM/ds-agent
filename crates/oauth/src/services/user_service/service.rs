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

use chrono::Utc;
use common::auth::AccessScope;
use common::errors::NotFoundExt;
use common::paginated_spec::Cursor;
use common::query::QueryFilter;
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::data::repositories::user::UserRepository;
use crate::entities::commands::{CreateUserCommand, PatchUserCommand};
use crate::entities::query::{Page, Paginated, Sort, UserFilter};
use crate::entities::user::User;
use crate::services::password;
use crate::services::user_service::UserServiceTrait;
use crate::services::user_service::views::{UserInfo, UserView};

pub struct UserService {
    user_repo: Arc<dyn UserRepository>,
    event_bus: Option<events::EventBus>,
}

impl UserService {
    pub fn new(user_repo: Arc<dyn UserRepository>) -> Self {
        Self {
            user_repo,
            event_bus: None,
        }
    }

    pub fn with_event_bus(mut self, event_bus: Option<events::EventBus>) -> Self {
        self.event_bus = event_bus;
        self
    }
}

#[async_trait::async_trait]
impl UserServiceTrait for UserService {
    #[tracing::instrument(level = "info", skip_all, err, fields(tenant = %scope.acting_tenant()))]
    async fn list_users(
        &self,
        scope: &AccessScope,
        filter: &UserFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<UserView>> {
        scope.require_read()?;
        filter.validate()?;
        let mut filter = filter.clone();
        filter.tenant_id = scope.resolve_query_tenant(filter.tenant_id.as_deref())?;
        let page = page.clamped();
        let (users, total) = tokio::try_join!(
            self.user_repo.get_all(&filter, &page, sort),
            self.user_repo.count(&filter),
        )?;

        let views: Vec<UserView> = users.into_iter().map(UserView::assemble).collect();
        Ok(Paginated::from_page(views, &page, Some(total), |u| {
            Cursor::encode_timestamp(&u.created_at)
        }))
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(tenant = %scope.acting_tenant()))]
    async fn get_user(&self, scope: &AccessScope, tenant_id: &str) -> Outcome<UserView> {
        scope.require_read_tenant(tenant_id)?;
        let user = self
            .user_repo
            .get_by_tenant_id(tenant_id)
            .await?
            .or_not_found(tenant_id, "user")?;
        Ok(UserView::assemble(user))
    }

    /// A convenience method for token service management
    /// Only difference to `get_user` is the [`UserInfo`] struct
    #[tracing::instrument(level = "info", skip_all, err, fields(tenant = %scope.acting_tenant()))]
    async fn user_info(&self, scope: &AccessScope, tenant_id: &str) -> Outcome<UserInfo> {
        scope.require_read_tenant(tenant_id)?;
        let user = self
            .user_repo
            .get_by_tenant_id(tenant_id)
            .await?
            .or_not_found(tenant_id, "user")?;
        Ok(UserInfo::assemble(user))
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(tenant = %scope.acting_tenant()))]
    async fn create_user(&self, scope: &AccessScope, cmd: &CreateUserCommand) -> Outcome<UserView> {
        scope.require_admin()?;
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
        let view = UserView::assemble(self.user_repo.create(&user).await?);
        events::emit_action!(
            self.event_bus,
            &view.tenant_id,
            crate::EVENT_PREFIX,
            "user",
            "create",
            &view
        );
        Ok(view)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(tenant = %scope.acting_tenant()))]
    async fn patch_user(
        &self,
        scope: &AccessScope,
        tenant_id: &str,
        cmd: &PatchUserCommand,
    ) -> Outcome<UserView> {
        scope.require_write_tenant(tenant_id)?;
        if !scope.is_admin() && cmd.role.is_some() {
            return Err(Errors::format(
                BadFormat::Received,
                "forbidden: only admins can change a user's role",
                None,
            ));
        }
        let view = UserView::assemble(
            self.user_repo
                .patch(
                    tenant_id,
                    cmd.email.clone(),
                    cmd.role,
                    cmd.extra_fields.clone(),
                )
                .await?,
        );
        events::emit_action!(
            self.event_bus,
            &view.tenant_id,
            crate::EVENT_PREFIX,
            "user",
            "edit",
            &view
        );
        Ok(view)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(tenant = %scope.acting_tenant()))]
    async fn delete_user(&self, scope: &AccessScope, tenant_id: &str) -> Outcome<()> {
        scope.require_admin()?;
        self.user_repo.delete(tenant_id).await?;
        events::emit_action!(
            self.event_bus,
            tenant_id,
            crate::EVENT_PREFIX,
            "user",
            "delete",
            &events::EntityDeletedDto::new(tenant_id)
        );
        Ok(())
    }
}
