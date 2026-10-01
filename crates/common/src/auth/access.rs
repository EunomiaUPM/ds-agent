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

use ymir::errors::{BadFormat, Errors, Outcome};

use crate::auth::claims::{Claims, RbacRole};
use crate::auth::validators::AuthValidators;

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
    /// An admin that selected a tenant explicitly sees only that tenant.
    pinned: bool,
}

impl AccessScope {
    /// Creates a new access scope bound to a tenant and caller role.
    pub fn new(claims: &Claims, tenant: &str) -> Self {
        Self {
            acting_tenant: tenant.to_string(),
            role: claims.role,
            pinned: false,
        }
    }

    /// Builds the caller scope from a requested tenant header, shared by HTTP and gRPC adapters.
    /// Non-admins may only request their own tenant; a missing header falls back to the claims tenant.
    /// An admin naming a tenant is pinned to it; without the header an admin sees every tenant.
    #[allow(clippy::result_large_err)]
    pub fn from_tenant_header(claims: &Claims, requested: Option<&str>) -> Outcome<Self> {
        let tenant_id = match requested {
            Some(raw) => {
                AuthValidators::tenant_id_validator()
                    .validate(&raw.to_string())
                    .map_err(|vs| Errors::format(BadFormat::Received, vs.to_string(), None))?;
                if !claims.is_admin() && claims.tenant_id() != raw {
                    return Err(Errors::forbidden(
                        "forbidden: caller tenant does not match requested tenant",
                        None,
                    ));
                }
                raw
            }
            None => claims.tenant_id(),
        };
        let mut scope = Self::new(claims, tenant_id);
        scope.pinned = requested.is_some();
        Ok(scope)
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
            pinned: false,
        }
    }

    /// Scope of an in-process facade call: the service token (Admin) pinned to `tenant`,
    /// exactly what the HTTP extractor builds from the service token plus `x-tenant-id`.
    pub fn service(tenant: &str) -> Self {
        Self {
            acting_tenant: tenant.to_string(),
            role: RbacRole::Admin,
            pinned: true,
        }
    }

    /// Scope of an in-process facade call that sends no `x-tenant-id`: the service token
    /// (Admin of `home_tenant`) unpinned, so it sees every tenant.
    pub fn service_cross_tenant(home_tenant: &str) -> Self {
        Self {
            acting_tenant: home_tenant.to_string(),
            role: RbacRole::Admin,
            pinned: false,
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

    /// Authorizes read access for a target tenant in the service layer.
    pub fn require_read_tenant(&self, target_tenant: &str) -> Outcome<()> {
        self.require_read()?;
        self.ensure_tenant_access(target_tenant)
    }

    /// Authorizes write access for a target tenant in the service layer.
    pub fn require_write_tenant(&self, target_tenant: &str) -> Outcome<()> {
        self.require_write()?;
        self.ensure_tenant_access(target_tenant)
    }

    /// Authorizes administrative access in the service layer.
    pub fn require_admin(&self) -> Outcome<()> {
        if self.is_admin() {
            Ok(())
        } else {
            Err(Rbac::forbidden())
        }
    }

    /// Tenant a read must match: a tenant sees only its own records, an admin every tenant
    /// unless it pinned one.
    pub fn tenant_filter(&self) -> Option<&str> {
        (!self.is_admin() || self.pinned).then_some(self.acting_tenant.as_str())
    }

    /// The tenant a newly created resource should default to or be forced into.
    pub fn acting_tenant(&self) -> &String {
        &self.acting_tenant
    }

    /// Resolves target tenant for a new resource and enforces write permission.
    /// Non-admins are forced to acting tenant; admins use requested or default to acting.
    pub fn resolve_create_tenant(&self, requested_tenant: Option<&str>) -> Outcome<String> {
        let target = if self.is_admin() {
            requested_tenant
                .filter(|s| !s.trim().is_empty())
                .unwrap_or(&self.acting_tenant)
        } else {
            &self.acting_tenant
        };
        self.require_write_tenant(target)?;
        Ok(target.to_string())
    }

    /// Resolves the effective tenant filter for list queries.
    /// Non-admins cannot query foreign tenants; admins can filter or query across all.
    pub fn resolve_query_tenant(&self, requested_tenant: Option<&str>) -> Outcome<Option<String>> {
        if self.is_admin() && !self.pinned {
            Ok(requested_tenant
                .filter(|s| !s.trim().is_empty())
                .map(|s| s.to_string()))
        } else if let Some(req) = requested_tenant.filter(|s| !s.trim().is_empty()) {
            if req != self.acting_tenant {
                Err(Rbac::forbidden())
            } else {
                Ok(Some(self.acting_tenant.clone()))
            }
        } else {
            Ok(Some(self.acting_tenant.clone()))
        }
    }

    /// Whether this scope permits operating on a resource owned by `owner`.
    pub fn permits(&self, owner: &str) -> bool {
        self.tenant_filter().is_none_or(|tenant| tenant == owner)
    }

    /// Fails unless `owner` is visible to this caller; a foreign record looks like a missing one.
    pub fn ensure_visible(&self, owner: &str, id: &str) -> Outcome<()> {
        if self.permits(owner) {
            Ok(())
        } else {
            Err(Errors::missing_resource(
                id.to_string(),
                "resource not found",
                None,
            ))
        }
    }
}
