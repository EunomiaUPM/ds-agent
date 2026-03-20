/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

use crate::entities::resource::ProtocolSpec;
use serde::{Deserialize, Serialize};

/// Push lifecycle: a subscribe spec plus an optional unsubscribe spec.
///
/// After a successful subscribe call the remote side will push data to the
/// registered callback URL.  The `unsubscribe` spec, when present, is called
/// to deregister the callback.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PushLifecycle {
    pub subscribe: ProtocolSpec,
    pub unsubscribe: Option<ProtocolSpec>,
}
