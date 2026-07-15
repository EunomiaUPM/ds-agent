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

use chrono::{DateTime, Utc};
use std::str::FromStr;
use tonic::Status;
use urn::Urn;

/// Helper to convert &str into Option<&str> based on emtpy content
/// Returns `Some(s)` if `s` is non-empty, `None` if it is `""`.
pub(super) fn non_empty(s: &str) -> Option<&str> {
    if s.is_empty() { None } else { Some(s) }
}

/// Convert Urn parsing problem into Status gRPC response
pub(super) fn parse_urn(s: &str, field: &str) -> Result<Urn, Status> {
    Urn::from_str(s).map_err(|e| Status::invalid_argument(format!("{field}: invalid URN — {e}")))
}

/// Convert Date parsing problem into Status gRPC response
pub(super) fn parse_dt(s: &str, field: &str) -> Result<DateTime<Utc>, Status> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| Status::invalid_argument(format!("{field}: invalid RFC3339 — {e}")))
}
