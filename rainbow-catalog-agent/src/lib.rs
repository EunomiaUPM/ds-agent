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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

#![allow(unused)]
pub(crate) mod cache;
pub(crate) mod config;
pub(crate) mod data;
pub(crate) mod entities;
pub(crate) mod errors;
pub(crate) mod grpc;
pub(crate) mod http;
pub(crate) mod protocols;
pub mod setup;

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
