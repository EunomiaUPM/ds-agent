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

//! Shared foundation of every agent: how a process boots and composes its modules, and the
//! cross-cutting pieces they all rely on.
//!
//! Each module explains how to use it in its own docs.
//!
//! - Boot and composition: [`boot`], [`module_loader`], [`config`], [`telemetry`], [`info_banner`].
//! - HTTP and gRPC: [`http_tracing`], [`http_global_404`], [`middleware`], [`grpc`],
//!   [`well_known`].
//! - Security: [`oauth`], [`facades`], [`vault_utils`].
//! - Errors and data: [`errors`], [`paginated_spec`], [`query`], [`validation`], [`cache`],
//!   [`batch_requests`].
//! - Linked data and DSP: [`rdf`], [`dsp_common`].
//! - Utilities: [`utils`], [`serde_utils`], [`id_mac`], [`test_utils`].

pub mod batch_requests;
pub mod boot;
pub mod cache;
pub mod config;
pub mod dsp_common;
pub mod errors;
pub mod facades;
pub mod grpc;
pub mod http_global_404;
pub mod http_tracing;
pub mod id_mac;
pub mod info_banner;
pub mod middleware;
pub mod oauth;
pub mod module_loader;
pub mod paginated_spec;
pub mod query;
pub mod rdf;
pub mod routes;
pub mod serde_utils;
pub mod telemetry;
pub mod test_utils;
pub mod utils;
pub mod validation;
pub mod vault_utils;
pub mod well_known;
