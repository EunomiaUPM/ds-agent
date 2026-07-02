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

use common::auth::claims::{Claims, Role};
use common::auth::rbac::Rbac;
use ymir::errors::Outcome;

use crate::entities::ids::TenantId;

/// Tenant authorization context of a caller.
///
/// Built once at the transport boundary (HTTP extractor or gRPC interceptor)
/// from the validated [`Claims`] plus the `X-Tenant-ID` the caller is acting as,
/// and then handed to the service layer. Centralizing it here means every tenant
/// scoping and ownership rule lives in one place instead of being re-implemented
/// by each transport (which is how the HTTP and gRPC handlers used to drift).
#[derive(Debug, Clone)]
pub(crate) struct AccessScope {
    /// The tenant the caller is acting as (their `X-Tenant-ID`).
    acting_tenant: TenantId,
    /// `true` for admins, who may cross tenant boundaries.
    unrestricted: bool,
}

impl AccessScope {
    /// Read scope: any authenticated role may read, but non-admins are confined
    /// to their own tenant. Fails (`403`) if the caller may not read this tenant.
    #[allow(clippy::result_large_err)]
    pub fn for_read(claims: &Claims, tenant: &TenantId) -> Outcome<Self> {
        Rbac::require_read(claims, tenant.as_str())?;
        Ok(Self::from_role(claims.role, tenant))
    }

    /// Write scope: readers are rejected (`403`), owners are confined to their own
    /// tenant, admins are unrestricted.
    #[allow(clippy::result_large_err)]
    pub fn for_write(claims: &Claims, tenant: &TenantId) -> Outcome<Self> {
        Rbac::require_write(claims, tenant.as_str())?;
        Ok(Self::from_role(claims.role, tenant))
    }

    /// Builds a scope directly from a role + tenant (used by tests and the
    /// transport constructors above).
    pub fn from_role(role: Role, tenant: &TenantId) -> Self {
        Self {
            acting_tenant: tenant.clone(),
            unrestricted: role == Role::Admin,
        }
    }

    /// Tenant to force into list filters. `None` means unrestricted (admin),
    /// so the caller-supplied filter is left untouched.
    pub fn tenant_filter(&self) -> Option<TenantId> {
        (!self.unrestricted).then(|| self.acting_tenant.clone())
    }

    /// The tenant a newly created resource should default to / be forced into.
    pub fn acting_tenant(&self) -> &TenantId {
        &self.acting_tenant
    }

    /// Whether this scope may touch a resource owned by `owner`.
    pub fn permits(&self, owner: &TenantId) -> bool {
        self.unrestricted || &self.acting_tenant == owner
    }
}
