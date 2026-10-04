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

//! Users as services receive them after authentication.

use serde_json::Map;

use crate::oauth::{RolePath, UserInfo};

/// Ready-made users for service tests.
pub struct TestUsers;

impl TestUsers {
    /// Superuser `/admin`: reaches everything.
    pub fn root() -> UserInfo {
        UserInfo::new("root", None, RolePath::root(), Map::new())
    }

    /// `user_id` acting under `role`; panics if `role` is not a valid path.
    pub fn user(user_id: &str, role: &str) -> UserInfo {
        let role = role.parse().expect("test role must be a valid path");
        UserInfo::new(user_id, None, role, Map::new())
    }
}
