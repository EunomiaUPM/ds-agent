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

use common::auth::AccessScope;
use ymir::errors::Outcome;

use crate::entities::commands::{CreateUserCommand, PatchUserCommand};
use crate::entities::query::{Page, Paginated, Sort, UserFilter};
use crate::services::user_service::views::{UserInfo, UserView};

pub mod service;
pub mod views;

// Service trait ────────────────────────────────────────────────────────────

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait UserServiceTrait: Send + Sync + 'static {
    async fn list_users(
        &self,
        scope: &AccessScope,
        filter: &UserFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<UserView>>;
    async fn get_user(&self, scope: &AccessScope, tenant_id: &str) -> Outcome<UserView>;
    async fn user_info(&self, scope: &AccessScope, tenant_id: &str) -> Outcome<UserInfo>;
    async fn create_user(&self, scope: &AccessScope, cmd: &CreateUserCommand) -> Outcome<UserView>;
    async fn patch_user(
        &self,
        scope: &AccessScope,
        tenant_id: &str,
        cmd: &PatchUserCommand,
    ) -> Outcome<UserView>;
    async fn delete_user(&self, scope: &AccessScope, tenant_id: &str) -> Outcome<()>;
}
