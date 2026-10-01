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

//! Domain filters for keystore queries.

use crate::entities::key::KeyPrefix;
use common::query::QueryFilter;
use serde::{Deserialize, Serialize};

/// Filter criteria for querying keystore entries by key prefix and tenant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PrefixFilter {
    #[serde(default)]
    pub prefix: Option<String>,
    #[serde(default)]
    pub tenant_id: Option<String>,
}

impl QueryFilter for PrefixFilter {
    fn is_empty(&self) -> bool {
        self.prefix.is_none() && self.tenant_id.is_none()
    }
}

impl From<KeyPrefix> for PrefixFilter {
    fn from(prefix: KeyPrefix) -> Self {
        Self {
            prefix: if prefix.as_str().is_empty() {
                None
            } else {
                Some(prefix.as_str().to_string())
            },
            tenant_id: None,
        }
    }
}

impl From<&KeyPrefix> for PrefixFilter {
    fn from(prefix: &KeyPrefix) -> Self {
        Self {
            prefix: if prefix.as_str().is_empty() {
                None
            } else {
                Some(prefix.as_str().to_string())
            },
            tenant_id: None,
        }
    }
}
