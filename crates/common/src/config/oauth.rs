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

//! Which identity provider authenticates the internal routes.

use serde::{Deserialize, Serialize};
use ymir::types::oauth::UserInfo;

/// Who authenticates the users, chosen in the yaml with `provider`. Without the block, `static`
/// as the system user, so nothing has to be configured locally.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "provider", rename_all = "snake_case")]
pub enum OauthConfig {
    /// The built-in OAuth server. Not operational yet: the agent refuses to start with it.
    BuiltIn,
    /// Keycloak behind oauth2-proxy: the proxy verifies the token and the agent reads it.
    Keycloak,
    /// Every request acts as this user. Local development only.
    Static(UserInfo),
}

impl Default for OauthConfig {
    fn default() -> Self {
        Self::Static(UserInfo::system())
    }
}
