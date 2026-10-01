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

use crate::entities::commands::CreateClientCommand;
use crate::entities::query::{ClientFilter, Page, Paginated, Sort};
use crate::services::client_service::views::ClientView;

pub mod service;
pub mod views;

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait ClientServiceTrait: Send + Sync + 'static {
    async fn list_clients(
        &self,
        scope: &AccessScope,
        filter: &ClientFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<ClientView>>;
    async fn get_client(&self, scope: &AccessScope, client_id: &str) -> Outcome<ClientView>;
    async fn create_client(
        &self,
        scope: &AccessScope,
        cmd: &CreateClientCommand,
    ) -> Outcome<ClientView>;
    async fn delete_client(&self, scope: &AccessScope, client_id: &str) -> Outcome<()>;
}
