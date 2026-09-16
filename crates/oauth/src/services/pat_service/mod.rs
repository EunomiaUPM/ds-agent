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

use chrono::{DateTime, Utc};
use common::auth::AccessScope;
use common::auth::claims::Claims;
use uuid::Uuid;
use ymir::errors::Outcome;

use crate::entities::query::{Page, Paginated, PatFilter, Sort};
use crate::entities::role::RbacRole;
use crate::services::pat_service::views::{CreatePatResponse, PatView};

pub(crate) mod service;
pub mod views;

#[async_trait::async_trait]
pub(crate) trait PatServiceTrait: Send + Sync + 'static {
    async fn create_pat(
        &self,
        scope: &AccessScope,
        name: &str,
        role: RbacRole,
        scopes: Vec<String>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Outcome<CreatePatResponse>;

    async fn list_pats(
        &self,
        scope: &AccessScope,
        filter: &PatFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<PatView>>;

    async fn revoke_pat(&self, scope: &AccessScope, id: Uuid) -> Outcome<()>;

    async fn validate_pat(&self, raw_token: &str) -> Outcome<Claims>;
}
