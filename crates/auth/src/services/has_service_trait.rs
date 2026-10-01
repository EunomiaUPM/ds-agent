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

use crate::data::factory::AuthRepoTrait;
use crate::services::callback::CallbackTrait;
use crate::services::gaia_self_attester::GaiaSelfAttesterTrait;
use crate::services::gatekeeper::GateKeeperTrait;
use crate::services::peer_connector::PeerConnectorTrait;
use crate::services::vc_requester::VcRequesterTrait;
use common::config::services::SsiAuthConfig;
use std::sync::Arc;

/// Access to the auth config, for the default methods of the modules.
pub trait HasConfig {
    fn config(&self) -> Arc<SsiAuthConfig>;
}

/// Access to the repositories.
pub trait HasRepo {
    fn repo(&self) -> Arc<dyn AuthRepoTrait>;
}

/// Access to the peer connector.
pub trait HasPeerConnector {
    fn peer_connector(&self) -> Arc<dyn PeerConnectorTrait>;
}

/// Access to the callback service.
pub trait HasCallback {
    fn callback(&self) -> Arc<dyn CallbackTrait>;
}

/// Access to the gatekeeper.
pub trait HasGateKeeper {
    fn gatekeeper(&self) -> Arc<dyn GateKeeperTrait>;
}

/// Access to the VC requester.
pub trait HasVcRequester {
    fn vc_requester(&self) -> Arc<dyn VcRequesterTrait>;
}

/// Access to the Gaia-X self-attester.
pub trait HasGaiaSelfAttester {
    fn gaia(&self) -> Arc<dyn GaiaSelfAttesterTrait>;
}
