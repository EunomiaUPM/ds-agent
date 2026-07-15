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

use crate::entities::key::Key;
use crate::entities::version::Version;
use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Metadata {
    pub key: Key,
    pub version: Version,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: String, // TODO for multitenancy
    pub updated_by: String,
    pub deleted_at: Option<DateTime<Utc>>,
    pub description: Option<String>,
}
