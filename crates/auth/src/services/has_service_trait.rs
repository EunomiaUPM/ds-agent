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

use crate::services::callback::CallbackTrait;
use crate::services::gaia_self_attester::GaiaSelfAttesterTrait;
use crate::services::gatekeeper::GateKeeperTrait;
use crate::services::peer_connector::PeerConnectorTrait;
use crate::services::repo::repo_trait::AuthRepoTrait;
use crate::services::vc_requester::VcRequesterTrait;
use std::sync::Arc;

pub trait HasRepo {
    fn repo(&self) -> Arc<dyn AuthRepoTrait>;
}

pub trait HasPeerConnector {
    fn peer_connector(&self) -> Arc<dyn PeerConnectorTrait>;
}

pub trait HasCallback {
    fn callback(&self) -> Arc<dyn CallbackTrait>;
}

pub trait HasGateKeeper {
    fn gatekeeper(&self) -> Arc<dyn GateKeeperTrait>;
}

pub trait HasVcRequester {
    fn vc_requester(&self) -> Arc<dyn VcRequesterTrait>;
}

pub trait HasGaiaSelfAttester {
    fn gaia(&self) -> Arc<dyn GaiaSelfAttesterTrait>;
}
