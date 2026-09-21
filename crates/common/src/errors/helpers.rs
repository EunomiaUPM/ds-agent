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

use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MissingAction {
    Token,
    Wallet,
    Did,
    Onboarding,
    Key,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BadFormat {
    Sent,
    Received,
    Unknown,
}
impl fmt::Display for MissingAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            MissingAction::Token => "Token",
            MissingAction::Wallet => "Wallet",
            MissingAction::Did => "DID",
            MissingAction::Key => "Key",
            MissingAction::Onboarding => "Onboarding",
            _ => "Unknown",
        };
        write!(f, "{}", s)
    }
}

use ymir::errors::{Errors, Outcome};

/// Extension trait for Option to convert None into a 404 missing resource error.
pub trait NotFoundExt<T> {
    /// Maps None to Errors::missing_resource with the given id and entity name.
    #[allow(clippy::result_large_err)]
    fn or_not_found(self, id: impl std::fmt::Display, entity_name: &str) -> Outcome<T>;
}

impl<T> NotFoundExt<T> for Option<T> {
    #[allow(clippy::result_large_err)]
    fn or_not_found(self, id: impl std::fmt::Display, entity_name: &str) -> Outcome<T> {
        self.ok_or_else(|| ResourceError::not_found(id, entity_name))
    }
}

/// Factory for standard missing resource (404) errors.
pub struct ResourceError;

impl ResourceError {
    /// Constructs an Errors::missing_resource (404) error.
    #[allow(clippy::result_large_err)]
    pub fn not_found(id: impl std::fmt::Display, entity_name: &str) -> Errors {
        Errors::missing_resource(id.to_string(), format!("{entity_name} not found"), None)
    }
}
