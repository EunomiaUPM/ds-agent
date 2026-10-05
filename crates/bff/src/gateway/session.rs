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

//! Who the browser's session is, as the identity provider says.
//!
//! The SPA never sees a token: with Keycloak, oauth2-proxy keeps the session in a cookie and adds
//! the token on its way to the agents; in `static` mode every request is the configured user.
//! The SPA asks here who it is, to show it and to adapt its views to the role (the agents
//! enforce access anyway). A 401 means there is no session: the SPA sends the browser to the
//! proxy's sign-in.

use axum::Json;
use common::oauth::{RoleTrait, UserInfo};
use serde::Serialize;

/// The session's user, as the SPA shows it.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeDto {
    pub user_id: String,
    pub username: Option<String>,
    pub email: Option<String>,
    pub role: String,
    /// Whether it is the root (`/admin`), who sees and acts on everything.
    pub root: bool,
}

pub struct SessionHandlers;

impl SessionHandlers {
    pub async fn me(user: UserInfo) -> Json<MeDto> {
        Json(MeDto {
            user_id: user.id().to_string(),
            username: user.username().map(str::to_string),
            email: user.email().map(str::to_string),
            role: user.role().to_string(),
            root: user.is_root(),
        })
    }
}
