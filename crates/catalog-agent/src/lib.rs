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

//! Catalog agent: DCAT resources stored as RDF graphs, one neutral core projected through
//! profile adapters (DSP for peers, DCAT 3 and DCAT-AP for public readers), managed through a
//! JSON-LD API and diagnosed by a doctor against every registered profile.
//!
//! It also mounts the connector, keeps the catalogs fetched from peers and builds policies from
//! templates. [`setup::CatalogAgentModule`] composes it into a larger process and
//! [`setup::CatalogAgentBoot`] runs it as its own service; its events use the `catalog:` prefix.
//!
//! Modules: [`entities`], [`services`], [`data`], [`cache`], [`profiles`], [`protocols`] (DSP),
//! [`http`], [`grpc`], [`facades`], [`setup`].

pub mod cache;
pub mod data;
pub mod entities;
pub mod facades;
pub mod grpc;
pub mod http;
pub mod profiles;
pub mod protocols;
pub mod services;
pub mod setup;

/// Service id used in logs and telemetry.
pub const SERVICE_NAME: &str = "catalog-agent";
/// Name shown in the boot banner.
pub const SERVICE_BIG_NAME: &str = "Catalog Agent";
/// Domain name of the catalog events.
pub const EVENT_DOMAIN: &str = "catalog";
/// Topic prefix of the catalog events.
pub const EVENT_PREFIX: &str = "catalog:";
