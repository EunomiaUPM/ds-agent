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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

mod business_router;
mod core_router;
mod gaia_self_issuer_router;
mod gatekeeper_router;
mod mate_router;
mod onboarder_router;
mod vc_requester_router;
mod verifier_router;

pub use business_router::BusinessRouter;
pub use core_router::AuthRouter;
pub use gaia_self_issuer_router::GaiaSelfIssuerRouter;
pub use gatekeeper_router::GateKeeperRouter;
pub use mate_router::MateRouter;
pub use onboarder_router::OnboarderRouter;
pub use vc_requester_router::VcRequesterRouter;
pub use verifier_router::VerifierRouter;
