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
use ymir::types::gnap::grant_request::interact::{InteractAction, InteractStart};
use ymir::types::participants::Visibility;
use ymir::types::vcs::VcTypeConfig;

/// Credential request to an authority; `auto` skips the manual steps.
#[derive(Serialize, Deserialize, Debug)]
pub struct ReachAuthority {
    pub id: String,
    pub nick: String,
    pub url: String,
    pub vc_type: VcTypeConfig,
    pub method: InteractStart,
    // #[serde(default)] TODO
    pub auto: Option<bool>,
}

/// Onboarding request to a peer for `actions`; `auto` skips the manual steps.
#[derive(Serialize, Deserialize, Debug)]
pub struct ReachProvider {
    pub id: String,
    pub nick: String,
    pub url: String,
    pub actions: Vec<InteractAction>,
    /// Who sees the request and, once it completes, the relation with the peer.
    pub visibility: Visibility,
    // #[serde(default)] TODO
    pub auto: Option<bool>,
    pub requested: Option<bool>,
}
