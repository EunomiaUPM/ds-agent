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

//! Atomic rules for validating claims and token metadata in validation pipelines.

use crate::auth::claims::Claims;
use crate::validation::violation::{codes, violation, Path, Violations};

/// Catalog of atomic validation rules for claims and token attributes.
pub struct AuthRules;

impl AuthRules {
    /// Ensure that a token expiration timestamp is strictly in the future.
    pub fn token_not_expired(exp: u64, now: u64, path: impl Into<Path>) -> Result<(), Violations> {
        let p = path.into();
        if exp >= now {
            Ok(())
        } else {
            Err(violation(p, codes::NOT_ALLOWED, "token has expired"))
        }
    }

    /// Ensure that claims expiration timestamp is strictly in the future.
    pub fn claims_not_expired(
        claims: &Claims,
        now: i64,
        path: impl Into<Path>,
    ) -> Result<(), Violations> {
        let p = path.into();
        if claims.exp >= now {
            Ok(())
        } else {
            Err(violation(p, codes::NOT_ALLOWED, "token has expired"))
        }
    }

    /// Ensure that claims subject (tenant id) is not empty.
    pub fn subject_not_empty(claims: &Claims, path: impl Into<Path>) -> Result<(), Violations> {
        let p = path.into();
        if !claims.sub.trim().is_empty() {
            Ok(())
        } else {
            Err(violation(
                p,
                codes::MISSING,
                "subject (sub) must not be empty",
            ))
        }
    }

    /// Ensure that a tenant identifier is well-formed (non-empty safe identifier).
    pub fn tenant_id_format(tenant_id: &str, path: impl Into<Path>) -> Result<(), Violations> {
        let p = path.into();
        let trimmed = tenant_id.trim();
        if trimmed.is_empty() {
            return Err(violation(p, codes::MISSING, "tenant id must not be empty"));
        }
        if trimmed.len() > 128 {
            return Err(violation(
                p,
                codes::MALFORMED,
                "tenant id exceeds maximum length of 128",
            ));
        }
        if trimmed
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.')
        {
            Ok(())
        } else {
            Err(violation(
                p,
                codes::MALFORMED,
                "tenant id contains invalid characters (allowed: alphanumeric, '-', '.')",
            ))
        }
    }

    /// Ensure that the token audience matches the expected service audience.
    pub fn audience_matches(
        aud: &str,
        expected: &str,
        path: impl Into<Path>,
    ) -> Result<(), Violations> {
        let p = path.into();
        if aud == expected {
            Ok(())
        } else {
            Err(violation(p, codes::NOT_ALLOWED, "invalid token audience"))
        }
    }

    /// Ensure that the token issuer matches the expected identity provider.
    pub fn issuer_matches(
        iss: &str,
        expected: &str,
        path: impl Into<Path>,
    ) -> Result<(), Violations> {
        let p = path.into();
        if iss == expected {
            Ok(())
        } else {
            Err(violation(p, codes::NOT_ALLOWED, "invalid token issuer"))
        }
    }

    /// Ensure that caller claims carry the required role.
    pub fn has_role(
        roles: &[String],
        required: &str,
        path: impl Into<Path>,
    ) -> Result<(), Violations> {
        let p = path.into();
        if roles.iter().any(|r| r == required) {
            Ok(())
        } else {
            Err(violation(
                p,
                codes::NOT_ALLOWED,
                format!("missing required role: {required}"),
            ))
        }
    }
}
