/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

mod core_business;
mod core_gatekeeper;
mod core_trait;
mod core_verifier;
mod gaia_self_issuer_trait;
mod mate_trait;
mod onboarder_trait;
mod vc_requester;

pub use core_business::CoreBusinessTrait;
pub use core_gatekeeper::CoreGateKeeperTrait;
pub use core_trait::AuthCoreTrait;
pub use core_verifier::CoreVerifierTrait;
pub use gaia_self_issuer_trait::CoreGaiaSelfIssuerTrait;
pub use mate_trait::CoreMateTrait;
pub use onboarder_trait::CoreOnboarderTrait;
pub use vc_requester::CoreVcRequesterTrait;
