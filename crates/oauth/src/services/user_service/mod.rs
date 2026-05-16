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
