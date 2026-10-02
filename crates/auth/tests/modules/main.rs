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

//! Auth capability modules over an `AuthCore` of doubles: GNAP onboarding with peers and
//! authorities, the gatekeeper and verifier that answer them, and the participant registry.

#[path = "../support/mod.rs"]
mod support;

mod gaia_self_attester;
mod gatekeeper;
mod participant;
mod peer_connector;
mod vc_requester;
mod verifier;
