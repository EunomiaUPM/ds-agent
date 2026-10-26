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

//! The failures of catalog writes that ymir has no constructor for: a conflict with what is
//! stored and a stale version.

use std::backtrace::Backtrace;

use axum::http::StatusCode;
use ymir::errors::{ErrorInfo, Errors};

/// Constructors of the catalog's own failures.
pub struct CatalogErrors;

impl CatalogErrors {
    /// 409: the write contradicts what is stored (a removed resource still linked, a second main).
    pub fn conflict(reason: impl Into<String>) -> Errors {
        Self::with_status(StatusCode::CONFLICT, 4090, "Conflict with the stored catalog", reason)
    }

    /// 412: `If-Match` names a version that is no longer the stored one.
    pub fn stale(reason: impl Into<String>) -> Errors {
        Self::with_status(
            StatusCode::PRECONDITION_FAILED,
            4120,
            "The resource changed since it was read",
            reason,
        )
    }

    fn with_status(status: StatusCode, code: u16, message: &str, reason: impl Into<String>) -> Errors {
        let reason = reason.into();
        Errors::FormatError {
            info: Box::new(ErrorInfo {
                message: message.to_string(),
                error_code: code,
                status_code: status,
                details: Some(reason.clone()),
                reasons: Vec::new(),
            }),
            reason,
            source: None,
            backtrace: Box::new(Backtrace::capture()),
        }
    }
}
