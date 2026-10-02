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

//! Admin user seeded at boot.

use serde::{Deserialize, Serialize};

/// Admin user seeded at boot; its `tenant_id` is the admin tenant.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct AdminSeedConfig {
    pub tenant_id: String,
    pub email: String,
    pub password: String,
}

impl Default for AdminSeedConfig {
    fn default() -> Self {
        Self {
            tenant_id: "admin".to_string(),
            email: "admin@admin.local".to_string(),
            password: "admin".to_string(),
        }
    }
}
