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

//! Pull lifecycle: the consumer fetches the data.

use crate::entities::interaction::TransferLifetime;
use crate::entities::resource::ProtocolSpec;
use serde::{Deserialize, Serialize};

/// Pull lifecycle: a single protocol spec used to fetch data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PullLifecycle {
    #[serde(default)]
    pub lifetime: TransferLifetime,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_retries: Option<u32>,
    pub data_access: ProtocolSpec,
}

impl PullLifecycle {
    pub fn new(data_access: ProtocolSpec) -> Self {
        Self {
            lifetime: TransferLifetime::Infinite,
            max_retries: None,
            data_access,
        }
    }
}
