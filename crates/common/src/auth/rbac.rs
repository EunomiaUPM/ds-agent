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

use ymir::errors::{Errors, Outcome};

use crate::auth::claims::{Claims, RbacRole};

/// Stateless RBAC guard. All methods take a reference to the validated [`Claims`]
/// extracted by the auth middleware.
pub struct Rbac;

impl Rbac {
    /// Only admins may proceed.
    pub fn require_admin(claims: &Claims) -> Outcome<()> {
        if claims.role == RbacRole::Admin {
            Ok(())
        } else {
            Err(forbidden())
        }
    }

    /// Admin may read any tenant's data; Owner/Reader may only read their own.
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
            RbacRole::Reader => Err(forbidden()),
        }
    }

    fn assert_own_tenant(claims: &Claims, tenant_id: &str) -> Outcome<()> {
        if claims.sub == tenant_id {
            Ok(())
        } else {
            Err(forbidden())
        }
    }
}

fn forbidden() -> Errors {
    Errors::forbidden("forbidden: insufficient permissions", None)
}
