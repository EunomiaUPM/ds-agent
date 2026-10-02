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

//! Ports to the auth agent that every other agent consumes, with their HTTP adapters.
//!
//! Catalog, negotiation and transfer need two things from the SSI auth agent: the known
//! participants of a tenant ([`MatesFacadeTrait`]) and the check of a peer's GNAP token on the
//! DSP endpoints ([`SSIAuthFacadeTrait`]). [`AuthPorts`] bundles both. The remote adapters live
//! here and call the auth agent with the service token; the in-process adapters live in the
//! `auth` crate, which `common` cannot depend on. The composition root picks one or the other.
//!
//! [`MatesFacadeTrait`]: mates_facade::MatesFacadeTrait
//! [`SSIAuthFacadeTrait`]: ssi_auth_facade::SSIAuthFacadeTrait
//!
//! ## 1. Choosing the adapters
//!
//! Each agent's `setup::ports` carries an `AuthPorts`. A standalone agent builds the remote one
//! from the auth agent's address; the monolith passes the auth module's local ports.
//!
//! ```rust,ignore
//! use common::facades::AuthPorts;
//!
//! impl CatalogPorts {
//!     pub fn remote(config: &CatalogConfig, root: &RootContext) -> Self {
//!         Self { auth: AuthPorts::remote(config.ssi_auth(), root) }
//!     }
//!
//!     pub fn local(auth: AuthPorts) -> Self {
//!         Self { auth }
//!     }
//! }
//!
//! // In the monolith:
//! let catalog_ports = CatalogPorts::local(auth.local_ports());
//! ```
//!
//! ## 2. Authenticating a peer on a DSP endpoint
//!
//! ```rust,ignore
//! match state.ssi_auth.verify_token(token).await {
//!     Ok(mate) => {
//!         request.extensions_mut().insert(mate);
//!         Ok(next.run(request).await)
//!     }
//!     Err(_) => Err(StatusCode::UNAUTHORIZED),
//! }
//! ```
//!
//! The returned `Mates` record carries the tenant the peer onboarded into, which is what
//! `crate::dsp_common::DspActor::peer` needs.
//!
//! ## 3. Reading participants
//!
//! ```rust,ignore
//! let me = ports.mates.get_me_mate(tenant_id.clone()).await?;
//! let peer = ports.mates.get_mate_by_id(tenant_id, mate_id).await?;
//! ```
//!
//! Both traits have `mockall` mocks (`MockMatesFacadeTrait`, `MockSSIAuthFacadeTrait`) for tests.

use serde::{Deserialize, Serialize};

pub mod mates_facade;
pub mod ports;
pub mod ssi_auth_facade;

pub use ports::AuthPorts;

/// Body of the auth agent's token verification call.
#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyTokenRequest {
    pub token: String,
}
