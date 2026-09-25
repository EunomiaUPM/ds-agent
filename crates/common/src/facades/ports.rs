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
use crate::facades::mates_facade::remote::MatesRemoteFacade;
use crate::facades::mates_facade::MatesFacadeTrait;
use crate::facades::ssi_auth_facade::remote::SSIAuthRemoteFacade;
use crate::facades::ssi_auth_facade::SSIAuthFacadeTrait;
use crate::module_loader::root_context::RootContext;

#[derive(Clone)]
pub struct AuthPorts {
    pub mates: Arc<dyn MatesFacadeTrait>,
    pub ssi_auth: Arc<dyn SSIAuthFacadeTrait>,
}

impl AuthPorts {
    /// Microservices: every port calls the auth agent at `auth` with the service token.
    pub fn remote(auth: &MinKnownConfig, root: &RootContext) -> Self {
        let auth = Arc::new(auth.clone());
        Self {
            mates: Arc::new(MatesRemoteFacade::new(
                auth.clone(),
                root.service_client.clone(),
            )),
            ssi_auth: Arc::new(SSIAuthRemoteFacade::new(auth, root.service_client.clone())),
        }
    }
}
