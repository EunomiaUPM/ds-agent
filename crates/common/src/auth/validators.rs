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

//! Composable validation pipelines for identity claims and tenant identifiers.

use chrono::Utc;

use crate::auth::claims::Claims;
use crate::auth::rules::AuthRules;
use crate::validation::Validator;

/// Catalog of pre-composed validators for identity claims and tenant boundaries.
pub struct AuthValidators;

impl AuthValidators {
    /// Validates decoded JWT claims (subject present, token not expired).
    pub fn claims_validator() -> Validator<Claims> {
        let now = Utc::now().timestamp();
        Self::claims_validator_at(now)
    }

    /// Validates decoded JWT claims against an explicit unix timestamp.
    pub fn claims_validator_at(now: i64) -> Validator<Claims> {
        Validator::new()
            .rule(|c: &Claims| AuthRules::subject_not_empty(c, "sub"))
            .then()
            .rule(move |c: &Claims| AuthRules::claims_not_expired(c, now, "exp"))
    }

    /// Validates tenant identifier string format.
    pub fn tenant_id_validator() -> Validator<String> {
        Validator::new()
            .rule(|tenant_id: &String| AuthRules::tenant_id_format(tenant_id, "tenant_id"))
    }
}
