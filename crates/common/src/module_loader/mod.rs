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

//! Composable service modules.
//!
//! A module is one self-contained slice of an agent (OAuth, the admin API,
//! a protocol surface, …). Implementing `ServiceModuleTrait` is the contract
//! that makes it composable: a module is constructed with its dependencies
//! (constructor injection) and contributes its DB migrations and its HTTP
//! and/or gRPC surface.
//!
//! [`service_composer::ServiceComposer`] collects registered modules and
//! exposes each plane:
//!
//! ```ignore
//! let composer = ServiceComposer::new()
//!     .register(OAuthModule::new(oauth_config, db))
//!     .register(ApiModule::new(process, message, validator))
//!     .register(DspModule::new(ssi_auth));
//! let http = composer.http_router();          // axum Router, all modules nested
//! let grpc = composer.grpc_routes();          // tonic Routes, all services added
//! let migs = composer.migrations();           // every module's migrations, in order
//! ```
//!
//! `sea_orm_migration`'s `MigratorTrait::migrations()` is a static fn, so an
//! agent's migrator concatenates each module's migration list directly
//! (e.g. `oauth::get_oauth_migrations()`) without building any module.

pub mod module_group;
pub mod service_composer;
pub mod service_module;
mod utils;
