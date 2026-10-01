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

mod gaia_self_attester;
mod gatekeeper;
mod participant;
mod peer_connector;
mod vc_requester;
mod verifier;

pub use gaia_self_attester::GaiaSelfAttesterModule;
pub use gatekeeper::GateKeeperModule;
pub use participant::ParticipantModule;
pub use peer_connector::PeerConnectorModule;
pub use vc_requester::VcRequesterModule;
pub use verifier::VerifierModule;
