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

//! Agent configuration: the YAML file every binary reads at boot and the typed views over it.
//!
//! One file describes a whole participant, with one section per agent (`monolith`, `catalog`,
//! `contracts`, `transfer`, `ssi_auth`, `gateway`). [`ApplicationConfig`] is the whole file; each
//! agent's config type in [`services`] is one section. Every section embeds a [`CommonConfig`]
//! with hosts, database, API version and connection flags, and the agents it talks to are
//! described by a [`MinKnownConfig`] with just their hosts and API version.
//!
//! [`CommonConfig`]: services::CommonConfig
//! [`MinKnownConfig`]: types::min_known_config::MinKnownConfig
//!
//! ## 1. The file
//!
//! YAML anchors keep the shared parts in one place. Files live in `static/environment/config`.
//!
//! ```yaml
//! common_config: &common_config
//!   hosts: { http: { protocol: 'http', url: '127.0.0.1', port: '1200' }, grpc: null }
//!   db: { db_type: Postgres, url: '127.0.0.1', port: '1400' }
//!   api: { version: 'v1' }
//!   connection: { is_local: true, is_prod: false, is_vault_real: false, has_tls_proxy: false }
//!
//! monolith:
//!   common: *common_config
//!   cache: *cache_config
//!
//! transfer:
//!   common: *common_config
//!   cache: *cache_config
//!   contracts: *min_known_config
//!   catalog: *min_known_config
//!   ssi_auth: *min_known_config
//!   is_catalog_datahub: false
//! ```
//!
//! `admin_seed`, `service_client`, `jwt_secret` and the token lifetimes have defaults, so dev
//! files can leave them out. The cache defaults to `Noop`.
//!
//! ## 2. Loading
//!
//! Every config type implements [`ConfigLoader`]. An agent's `load` reads the whole file and keeps
//! its own section; if that fails it reads the file as that section alone, so a standalone
//! agent can ship a file with just its part. A relative path is resolved from the `common`
//! crate directory, which is why dev commands pass `../../static/...`.
//!
//! ```rust,ignore
//! use common::config::services::TransferConfig;
//! use common::config::types::traits::ConfigLoader;
//!
//! let config = TransferConfig::load("../../static/environment/config/dev/dev.provider.yaml")?;
//! ```
//!
//! The boot does this itself through `BootstrapServiceTrait::Config`; agents rarely call it.
//!
//! [`ConfigLoader`]: types::traits::ConfigLoader
//!
//! ## 3. Reading values
//!
//! [`CommonConfigTrait`] gives the common section of any config, and ymir's traits read hosts and
//! the API prefix from it. Each agent config also has its own trait in [`services::traits`]
//! with the sections it needs.
//!
//! ```rust,ignore
//! use common::config::types::traits::CommonConfigTrait;
//! use ymir::config::traits::{ApiConfigTrait, HostsConfigTrait};
//! use ymir::config::types::HostType;
//!
//! let common = config.common();
//! let host = common.get_host(HostType::Http);       // "http://127.0.0.1:1200"
//! let prefix = format!("{}/keystore", common.get_api_version()); // "/api/v1/keystore"
//! let tenant = &config.admin_seed().tenant_id;
//!
//! let cache = config.cache();                         // TransferConfigTrait
//! ```
//!
//! [`CommonConfigTrait`]: types::traits::CommonConfigTrait
//!
//! ## 4. Reaching another agent
//!
//! A remote facade builds its URLs from the other agent's [`MinKnownConfig`]. In the monolith
//! these point back at the same process.
//!
//! ```rust,ignore
//! use common::config::types::traits::MinKnownConfigTrait;
//!
//! let contracts = config.contracts();
//! let agreements_url = format!(
//!     "{}{}/{}/agreements",
//!     contracts.get_host(HostType::Http),
//!     contracts.get_api_version(),
//!     negotiation_agent::SERVICE_NAME,
//! );
//! ```
//!
//! ## 5. The monolith
//!
//! [`ApplicationConfig`] is the monolith's config. Its own `monolith.common` drives the process
//! (ports, vault, database), and `transfer()`, `catalog()` and the rest hand each module its
//! section, or an error when the file lacks it. Loading it as the monolith's config fails at
//! once, naming every missing section.

mod config;
mod parse_from;
pub mod services;
pub mod types;

pub use config::ApplicationConfig;

#[cfg(test)]
mod tests;
