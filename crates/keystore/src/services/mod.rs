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

//! Parameter and secret stores.

//!
//! Entries belong to one user: each caller reads and writes its own, and only the root lists
//! everyone's (or one user's, with the `user_id` filter).

use common::oauth::{RoleTrait, UserInfo};
use ymir::errors::{Errors, Outcome};

pub mod config;
pub mod parameters;
pub mod secrets;

/// Owner a listing filters by: for the root, the one it asks for (`None`, everyone); for the
/// rest, always its own, and a forbidden error if it asks for someone else's.
pub(crate) fn owner_for_list(user: &UserInfo, requested: Option<&str>) -> Outcome<Option<String>> {
    let requested = requested.filter(|owner| !owner.trim().is_empty());
    if user.is_root() {
        return Ok(requested.map(str::to_string));
    }
    match requested {
        Some(owner) if owner != user.id() => Err(Errors::forbidden(
            "forbidden: cannot query another user's entries",
            None,
        )),
        _ => Ok(Some(user.id().to_string())),
    }
}
