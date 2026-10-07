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

//! Catalog agent: DCAT 3 catalogs, datasets, distributions, data services and ODRL policies,
//! served to peers over the DSP catalog protocol and to the owner through a management API.
//!
//! It also mounts the connector, keeps cached copies of peer catalogs and builds policies from
//! templates. [`setup::CatalogAgentModule`] composes it into a larger process and
//! [`setup::CatalogAgentBoot`] runs it as its own service. Other crates get its migrations,
//! repository trait and main DTOs from the root; its events use the `catalog:` prefix.
//!
//! Modules: [`entities`], [`services`], [`data`], [`cache`] (Redis), [`grpc`], [`http`],
//! [`protocols`] (DSP), [`facades`], [`setup`].

pub mod cache;
pub mod data;
pub mod entities;
pub mod facades;
pub mod grpc;
pub mod http;
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
/// Cache key of the connector's main catalog and data service (there is one of each).
pub const MAIN_CACHE_KEY: &str = "connector";

pub use data::migrations::get_catalog_migrations;
pub use data::repo_traits::catalog_repo::CatalogRepositoryTrait;
pub use data::repos_sql::catalog_repo::CatalogRepositoryForSql;
pub use entities::catalogs::CatalogDto;
pub use entities::catalogs::NewCatalogDto;
pub use entities::data_services::DataServiceDto;
pub use entities::data_services::NewDataServiceDto;
pub use entities::datasets::DatasetDto;
pub use entities::distributions::DistributionDto;
pub use entities::odrl_policies::OdrlPolicyDto;
