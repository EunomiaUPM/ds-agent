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

//! Ports to the auth agent that every other agent consumes, with their HTTP adapters, and the
//! contracts the catalog and negotiation agents publish.
//!
//! Catalog, negotiation and transfer need two things from the SSI auth agent: the participants a
//! user sees ([`MatesFacadeTrait`]) and the GNAP tokens with the peers ([`GrantsFacadeTrait`]):
//! checking the one a peer presents on the DSP endpoints, and getting the one a user presents
//! when calling a peer. [`AuthPorts`] bundles both. The remote adapters live here and call the
//! auth agent over HTTP; the in-process adapters live in the `auth` crate, which `common` cannot
//! depend on. The composition root picks one or the other.
//!
//! [`MatesFacadeTrait`]: mates_facade::MatesFacadeTrait
//! [`GrantsFacadeTrait`]: grants_facade::GrantsFacadeTrait
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
//! // What `crate::dsp_common::PeerAuth`, the layer of every DSP plane, does:
//! match state.grants.verify_token(token).await {
//!     Ok(peer) => {
//!         request.extensions_mut().insert(peer);
//!         Ok(next.run(request).await)
//!     }
//!     Err(_) => Err(StatusCode::NOT_FOUND),
//! }
//! ```
//!
//! The returned [`VerifiedPeer`] says who the peer is and which role handles what it opens.
//!
//! [`VerifiedPeer`]: grants_facade::VerifiedPeer
//!
//! ## 3. Calling a peer
//!
//! ```rust,ignore
//! let headers = match ports.grants.peer_token(&user, peer_id).await? {
//!     Some(token) => Some(bearer_headers(&token)?),
//!     None => None,
//! };
//! ```
//!
//! ## 4. Reading participants
//!
//! ```rust,ignore
//! let me = ports.mates.get_me_mate().await?;
//! let peer = ports.mates.get_mate_by_id(&user, mate_id).await?;
//! ```
//!
//! Both traits have `mockall` mocks (`MockMatesFacadeTrait`, `MockGrantsFacadeTrait`) for tests.
//!
//! ## 5. Reading another agent's resources
//!
//! The catalog and negotiation agents publish what others read from them as a contract here:
//! the trait, the records that cross ([`catalog::CatalogDataset`], [`negotiation::StoredAgreement`]
//! and friends) and the remote adapter over their API. The owning agent ships the in-process
//! adapter over its services; consumers never depend on the owning agent.
//!
//! ```rust,ignore
//! use common::facades::catalog::{remote::CatalogRemoteFacade, CatalogFacadeTrait};
//! use common::facades::negotiation::remote::NegotiationRemoteFacade;
//!
//! // Standalone transfer agent:
//! let catalog: Arc<dyn CatalogFacadeTrait> = Arc::new(CatalogRemoteFacade::new(config.catalog()));
//! let negotiation = Arc::new(NegotiationRemoteFacade::new(config.contracts()));
//!
//! // Monolith: the owners' local adapters.
//! let catalog = catalog_module.catalog_facade();
//! let agreement = negotiation.get_agreement(&agreement_id).await?; // an OdrlAgreement
//! ```

use serde::{Deserialize, Serialize};
use ymir::types::gnap::grant_request::access::AccessNeed;

pub mod catalog;
pub mod grants_facade;
pub mod mates_facade;
pub mod negotiation;
pub mod ports;
pub mod registry_facade;
// Thin wrapper over ymir's `http_client()`, left over from the client-credentials service
// token; remote facades call `http_client()` directly. Kept as it was, out of the module tree.
// pub mod service_client;
// Replaced by `grants_facade`, which every agent uses now. Kept as it was, out of the module
// tree.
// pub mod ssi_auth_facade;

// pub use service_client::ServiceHttpClient;

pub use ports::AuthPorts;

/// Body of the auth agent's token verification call.
#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyTokenRequest {
    pub token: String,
}

/// Body of the auth agent's call checking that a token covers what a DSP call needs.
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthorizeTokenRequest {
    pub token: String,
    pub need: AccessNeed,
}

/// Body of the auth agent's calls for a token towards a peer.
#[derive(Debug, Serialize, Deserialize)]
pub struct PeerTokenRequest {
    pub need: AccessNeed,
    #[serde(default = "requested_by_default")]
    pub requested: bool,
}

fn requested_by_default() -> bool {
    true
}

#[cfg(test)]
mod tests;
