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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use serde::{Deserialize, Serialize};
use ymir::types::gnap::grant_request::interact::InteractAction;

#[derive(Serialize, Deserialize, Debug)]
pub struct ReachAuthority {
    pub id: String,
    pub slug: String,
    pub url: String,
    pub vc_type: String,
    pub method: String,
    pub auto: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReachProvider {
    pub id: String,
    pub slug: String,
    pub url: String,
    pub actions: Vec<InteractAction>,
    pub auto: Option<bool>,
}
