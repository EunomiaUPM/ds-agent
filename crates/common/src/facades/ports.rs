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

//! Auth ports every agent consumes, embedded in each crate's `setup::ports` bundle.

use std::sync::Arc;

use crate::config::types::min_known_config::MinKnownConfig;
use crate::facades::grants_facade::remote::GrantsRemoteFacade;
use crate::facades::grants_facade::GrantsFacadeTrait;
use crate::facades::mates_facade::remote::MatesRemoteFacade;
use crate::facades::mates_facade::MatesFacadeTrait;

/// Auth agent ports an agent depends on, local or remote.
#[derive(Clone)]
pub struct AuthPorts {
    pub mates: Arc<dyn MatesFacadeTrait>,
    pub grants: Arc<dyn GrantsFacadeTrait>,
    // Replaced by `grants` (its `verify_token`); kept until the agents move over.
    // pub ssi_auth: Arc<dyn SSIAuthFacadeTrait>,
}

impl AuthPorts {
    /// Microservices: every port calls the auth agent at `auth` over HTTP.
    pub fn remote(auth: &MinKnownConfig) -> Self {
        let auth = Arc::new(auth.clone());
        Self {
            mates: Arc::new(MatesRemoteFacade::new(auth.clone())),
            grants: Arc::new(GrantsRemoteFacade::new(auth)),
        }
    }
}
