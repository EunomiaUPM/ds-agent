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

//! Access scopes per role, as services receive them after authentication.

use crate::auth::{AccessScope, RbacRole};

/// Ready-made scopes for service tests.
pub struct TestScopes;

impl TestScopes {
    /// Unpinned Admin of `admin-tenant`: sees every tenant.
    pub fn admin() -> AccessScope {
        AccessScope::from_role(RbacRole::Admin, "admin-tenant")
    }

    /// Unpinned Admin acting on `tenant` by default.
    pub fn admin_of(tenant: &str) -> AccessScope {
        AccessScope::from_role(RbacRole::Admin, tenant)
    }

    pub fn owner(tenant: &str) -> AccessScope {
        AccessScope::from_role(RbacRole::Owner, tenant)
    }

    pub fn reader(tenant: &str) -> AccessScope {
        AccessScope::from_role(RbacRole::Reader, tenant)
    }
}
