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

//! Entry metadata. Access scopes come from `common::test_utils::scopes`.

use chrono::Utc;
use keystore::entities::key::Key;
use keystore::entities::metadata::Metadata;
use keystore::entities::version::Version;

/// Metadata of a fresh entry at the initial version, created by `tester`.
pub fn metadata(tenant: &str, key: Key) -> Metadata {
    let now = Utc::now();
    Metadata {
        user_id: tenant.to_string(),
        key,
        version: Version::INITIAL,
        created_at: now,
        updated_at: now,
        created_by: "tester".to_string(),
        updated_by: "tester".to_string(),
        deleted_at: None,
        description: None,
    }
}
