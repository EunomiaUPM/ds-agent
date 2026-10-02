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

use serde::{Deserialize, Serialize};

/// How this participant presents itself to the authority when requesting credentials.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct EntityClientConfig {
    pub class_id: String, // como se denomina una entidad a si misma
    pub display: Option<DisplayInfo>,
}

/// Display name and links shown to the authority.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DisplayInfo {
    pub name: String,
    pub uri: Option<String>,
    pub logo_uri: Option<String>,
}
