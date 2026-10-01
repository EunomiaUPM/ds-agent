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

//! Personal access tokens.

use chrono::{DateTime, Utc};
use common::auth::AccessScope;
use common::auth::claims::Claims;
use uuid::Uuid;
use ymir::errors::Outcome;

use crate::entities::query::{Page, Paginated, PatFilter, Sort};
use crate::entities::role::RbacRole;
use crate::services::pat_service::views::{CreatePatResponse, PatView};

pub mod service;
pub mod views;

/// Personal access tokens: long-lived bearer tokens of a tenant.
#[mockall::automock]
#[async_trait::async_trait]
pub trait PatServiceTrait: Send + Sync + 'static {
    /// Creates a token; the raw value is only returned here.
    async fn create_pat(
        &self,
        scope: &AccessScope,
        name: &str,
        role: RbacRole,
        scopes: Vec<String>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Outcome<CreatePatResponse>;

    /// Page of tokens visible to the caller.
    async fn list_pats(
        &self,
        scope: &AccessScope,
        filter: &PatFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<PatView>>;

    /// Marks the token revoked; it stops authenticating at once.
    async fn revoke_pat(&self, scope: &AccessScope, id: Uuid) -> Outcome<()>;

    /// Claims of a raw token; fails when it is unknown, expired or revoked.
    async fn validate_pat(&self, raw_token: &str) -> Outcome<Claims>;
}
