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

use crate::config::types::cache::CacheConfig;
use crate::config::types::min_known_config::MinKnownConfig;
use ymir::config::traits::{CommonConfigTrait, ConfigLoader};

/// What the transfer agent reads from its config.
pub trait TransferConfigTrait: CommonConfigTrait + ConfigLoader {
    /// Address of the negotiation agent.
    fn contracts(&self) -> &MinKnownConfig;
    /// Address of the catalog agent.
    fn catalog(&self) -> &MinKnownConfig;
    /// Address of the auth agent.
    fn ssi_auth(&self) -> &MinKnownConfig;
    fn cache(&self) -> &CacheConfig;
    /// Whether the catalog is backed by a datahub.
    fn is_catalog_datahub(&self) -> bool;
    /// Whether the HTTP data proxy accepts invalid/self-signed upstream TLS certificates (dev only).
    fn proxy_accept_invalid_certs(&self) -> bool {
        false
    }
    /// Whether the HTTP data proxy permits outbound egress to loopback/RFC1918/link-local addresses (dev/Docker local only).
    fn proxy_allow_private_egress(&self) -> bool {
        false
    }
    /// Default number of transient upstream/sink retries tolerated on infinite transfers before
    /// emitting a failure signal to terminate the DSP transfer (`3` by default).
    fn proxy_max_retries(&self) -> u32 {
        3
    }
}
