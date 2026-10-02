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

//! The `.well-known/dspace-version` surface and the RPC that reads it from a peer.
//!
//! Every agent process serves the version document at its root: the boot merges
//! [`WellKnownRoot::get_well_known_router`] into the HTTP plane, so modules never mount it
//! themselves. The same router exposes two RPC endpoints the local side uses to discover where
//! a known peer serves DSP 2025-1, by fetching that peer's own version document.
//!
//! ## 1. What it serves
//!
//! | Method | Path | Answer |
//! |---|---|---|
//! | `GET` | `/.well-known/dspace-version` | `VersionResponse` with every supported version |
//! | `GET` | `/.well-known/dspace-version/2025-1` | that version alone |
//! | `POST` | `/rpc/.well-known/dspace-version` | a peer's `VersionResponse` |
//! | `POST` | `/rpc/.well-known/dspace-version/path` | a peer's DSP 2025-1 base URL |
//!
//! The only version advertised is 2025-1 at `/dsp/current`, over HTTPS, with GNAP auth and
//! `did:jwk` identifiers. Its `serviceId` is a URN derived from the path, so it is stable
//! across restarts.
//!
//! ```json
//! { "protocolVersions": [{
//!     "binding": "HTTPS", "path": "/dsp/current", "version": "2025-1",
//!     "auth": { "protocol": "GNAP", "version": "1" },
//!     "identifierType": "did:jwk", "serviceId": "urn:dsp-service-id:..."
//! }] }
//! ```
//!
//! ## 2. Mounting it
//!
//! The boot does this already. `mates` is the process's resolved participant port; without it
//! the router reads participants from the auth agent over HTTP.
//!
//! ```rust,ignore
//! use common::well_known::WellKnownRoot;
//!
//! let router = composer
//!     .http_router()
//!     .merge(WellKnownRoot::get_well_known_router(
//!         &MinKnownConfig::from(common),
//!         composer.auth_ports().map(|p| p.mates.clone()),
//!     )?);
//! ```
//!
//! ## 3. Finding a peer's DSP endpoint
//!
//! [`WellKnownRPCTrait`] looks the participant up in the tenant's registry, fetches
//! `{base_url}/.well-known/dspace-version` and returns the base URL for 2025-1. Agents call it
//! before sending the first message of a process.
//!
//! ```rust,ignore
//! use common::well_known::rpc::{WellKnownRPCRequest, WellKnownRPCTrait};
//!
//! let input = WellKnownRPCRequest { tenant_id, participant_id };
//! let dsp_base = self.rpc.fetch_dataspace_current_path(&input).await?.path;
//! // "https://provider.example.org/dsp/current"
//! ```
//!
//! [`WellKnownRPCTrait`]: rpc::WellKnownRPCTrait

use std::sync::Arc;

use ymir::errors::Outcome;

use crate::auth::ServiceHttpClient;
use crate::config::types::min_known_config::MinKnownConfig;
use crate::config::types::traits::MinKnownConfigTrait;
use crate::facades::mates_facade::remote::MatesRemoteFacade;
use crate::facades::mates_facade::MatesFacadeTrait;
use crate::well_known::dspace_version::dspace_version::WellKnownDSpaceVersionService;
use crate::well_known::router::WellKnownRouter;
use crate::well_known::rpc::rpc::WellKnownRPCService;
use ymir::config::types::HostType;

pub mod dspace_version;
pub mod router;
pub mod rpc;

/// Entry point the boot uses to build the well-known router.
pub struct WellKnownRoot;
impl WellKnownRoot {
    /// `mates` is the process's resolved port; without one, participants are read over HTTP.
    pub fn get_well_known_router(
        config: &MinKnownConfig,
        mates: Option<Arc<dyn MatesFacadeTrait>>,
    ) -> Outcome<axum::Router> {
        let mates_facade = mates.unwrap_or_else(|| Self::remote_mates(config));

        let dspace_version_service = WellKnownDSpaceVersionService::new();
        let dspace_version_rpc = Arc::new(WellKnownRPCService::new(mates_facade.clone()));
        let router = WellKnownRouter::new(dspace_version_service, dspace_version_rpc.clone());
        Ok(router.router())
    }

    fn remote_mates(config: &MinKnownConfig) -> Arc<dyn MatesFacadeTrait> {
        let config = Arc::new(config.clone());
        let service_client = Arc::new(ServiceHttpClient::new(
            &config.service_client,
            &config.get_host(HostType::Http),
        ));
        Arc::new(MatesRemoteFacade::new(config, service_client))
    }
}
