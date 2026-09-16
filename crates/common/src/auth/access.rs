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

//! Authorization policy, RBAC guards, and multi-tenant access scopes.

use ymir::errors::{Errors, Outcome};

use crate::auth::claims::{Claims, RbacRole};

/// Stateless RBAC policy guard for claims.
pub struct Rbac;

impl Rbac {
    /// Only admins may proceed.
    pub fn require_admin(claims: &Claims) -> Outcome<()> {
        if claims.role == RbacRole::Admin {
            Ok(())
        } else {
            Err(Self::forbidden())
        }
    }

    /// Admin may read any tenant's data; Owner and Reader may only read their own.
    pub fn require_read(claims: &Claims, tenant_id: &str) -> Outcome<()> {
        match claims.role {
            RbacRole::Admin => Ok(()),
            RbacRole::Owner | RbacRole::Reader => Self::assert_own_tenant(claims, tenant_id),
        }
    }

    /// Admin may write any tenant's data; Owner may write their own; Reader is denied.
    pub fn require_write(claims: &Claims, tenant_id: &str) -> Outcome<()> {
        match claims.role {
            RbacRole::Admin => Ok(()),
            RbacRole::Owner => Self::assert_own_tenant(claims, tenant_id),
            RbacRole::Reader => Err(Self::forbidden()),
        }
    }

    fn assert_own_tenant(claims: &Claims, tenant_id: &str) -> Outcome<()> {
        if claims.sub == tenant_id {
            Ok(())
        } else {
            Err(Self::forbidden())
        }
    }

    /// Generates standard 403 Forbidden error.
    pub fn forbidden() -> Errors {
        Errors::forbidden("forbidden: insufficient permissions", None)
    }
}

/// Tenant authorization context of a caller evaluated at the service layer.
#[derive(Debug, Clone)]
pub struct AccessScope {
    acting_tenant: String,
    role: RbacRole,
}

impl AccessScope {
    /// Creates a new access scope bound to a tenant and caller role.
    pub fn new(claims: &Claims, tenant: &str) -> Self {
        Self {
            acting_tenant: tenant.to_string(),
            role: claims.role,
        }
    }

    /// Read scope: non-admins are confined to their own tenant.
    pub fn for_read(claims: &Claims, tenant: &str) -> Outcome<Self> {
        Rbac::require_read(claims, tenant)?;
        Ok(Self::new(claims, tenant))
    }

    /// Write scope: readers are rejected, owners are confined to their own tenant.
    pub fn for_write(claims: &Claims, tenant: &str) -> Outcome<Self> {
        Rbac::require_write(claims, tenant)?;
        Ok(Self::new(claims, tenant))
    }

    /// Builds a scope directly from a role and tenant identifier.
    pub fn from_role(role: RbacRole, tenant: &str) -> Self {
        Self {
            acting_tenant: tenant.to_string(),
            role,
        }
    }

    /// Role held by the authenticated caller.
    pub fn role(&self) -> RbacRole {
        self.role
    }

    /// Whether this caller has administrative privileges.
    pub fn is_admin(&self) -> bool {
        self.role == RbacRole::Admin
    }

    /// Authorizes read access in the service layer.
    pub fn require_read(&self) -> Outcome<()> {
        Ok(())
    }

    /// Authorizes write access in the service layer (rejects Reader with 403).
    pub fn require_write(&self) -> Outcome<()> {
        match self.role {
            RbacRole::Admin | RbacRole::Owner => Ok(()),
            RbacRole::Reader => Err(Rbac::forbidden()),
        }
    }

    /// Ensures that this scope has access to the target tenant.
    pub fn ensure_tenant_access(&self, target_tenant: &str) -> Outcome<()> {
        if self.is_admin() || self.acting_tenant == target_tenant {
            Ok(())
        } else {
            Err(Rbac::forbidden())
        }
    }

    /// Authorizes administrative access in the service layer.
    pub fn require_admin(&self) -> Outcome<()> {
        if self.is_admin() {
            Ok(())
        } else {
            Err(Rbac::forbidden())
        }
    }

    /// Tenant to force into list filters, or `None` if unrestricted (admin).
    pub fn tenant_filter(&self) -> Option<String> {
        (!self.is_admin()).then(|| self.acting_tenant.clone())
    }

    /// The tenant a newly created resource should default to or be forced into.
    pub fn acting_tenant(&self) -> &String {
        &self.acting_tenant
    }

    /// Whether this scope permits operating on a resource owned by `owner`.
    pub fn permits(&self, owner: &str) -> bool {
        self.is_admin() || self.acting_tenant == owner
    }
}
