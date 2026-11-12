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

//! Domain conflict errors for connector template and instance lifecycle guards.

use std::backtrace::Backtrace;

use axum::http::StatusCode;
use ymir::errors::{ErrorInfo, Errors};

/// Constructors of the connector's HTTP 409 conflict failures.
pub struct ConnectorErrors;

impl ConnectorErrors {
    /// 409: the delete contradicts active state (e.g. active transfers using an instance, or
    /// existing instances referencing a template).
    pub fn conflict(reason: impl Into<String>) -> Errors {
        let reason = reason.into();
        Errors::FormatError {
            info: Box::new(ErrorInfo {
                message: "Conflict with active connector state".to_string(),
                error_code: 4090,
                status_code: StatusCode::CONFLICT,
                details: Some(reason.clone()),
                reasons: Vec::new(),
            }),
            reason,
            source: None,
            backtrace: Box::new(Backtrace::capture()),
        }
    }

    /// Whether `err` maps to HTTP 409 Conflict.
    pub fn is_conflict(err: &Errors) -> bool {
        err.info().status_code == StatusCode::CONFLICT
    }
}
