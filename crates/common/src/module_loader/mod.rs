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

//! Composable service modules: how an agent is split into slices and put back together.
//!
//! A module is one self-contained slice of an agent (OAuth, the keystore, a protocol surface).
//! It is built with its dependencies and contributes migrations, HTTP and gRPC surfaces,
//! background workers and boot seeders through [`ServiceModuleTrait`]. A [`ModuleGroup`] is a
//! module made of modules, [`ServiceComposer`] is the root the boot reads every plane from, and
//! [`RootContext`] holds the infrastructure all of them share.
//!
//! [`ServiceModuleTrait`]: service_module::ServiceModuleTrait
//! [`ModuleGroup`]: module_group::ModuleGroup
//! [`ServiceComposer`]: service_composer::ServiceComposer
//! [`RootContext`]: root_context::RootContext
//!
//! ## 1. Writing a module
//!
//! Build the module in a `compose` constructor from the root context and implement only the
//! planes it has; every hook defaults to nothing.
//!
//! ```rust,ignore
//! use axum::Router;
//! use common::module_loader::root_context::RootContext;
//! use common::module_loader::service_module::ServiceModuleTrait;
//! use sea_orm_migration::MigrationTrait;
//!
//! pub struct KeystoreModule {
//!     prefix: String,
//!     ctx: AppContext,
//! }
//!
//! impl KeystoreModule {
//!     pub fn compose(
//!         config: &ApplicationConfig,
//!         root: &RootContext,
//!         bus: Option<EventBus>,
//!     ) -> Self {
//!         Self {
//!             prefix: format!("{}/keystore", config.common().get_api_version()),
//!             ctx: AppContext::build(config, root, bus),
//!         }
//!     }
//!
//!     // Static, so a migrator can list it without composing anything.
//!     pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
//!         crate::get_keystore_migrations()
//!     }
//! }
//!
//! impl ServiceModuleTrait for KeystoreModule {
//!     fn name(&self) -> &'static str {
//!         "keystore"
//!     }
//!
//!     fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
//!         Self::migrations()
//!     }
//!
//!     // Absolute mount prefix and router. An empty prefix merges the router at the root.
//!     fn http(&self) -> Option<(String, Router)> {
//!         Some((self.prefix.clone(), KeystoreRouter::new(/* services */).router()))
//!     }
//! }
//! ```
//!
//! ## 2. Workers, seeders and gRPC
//!
//! Long-running tasks go through `workers()`, never `tokio::spawn`. Seeders are built on the
//! module's own services. A gRPC plane registers its services and its descriptor set, which
//! feeds the reflection service.
//!
//! ```rust,ignore
//! impl ServiceModuleTrait for EventsModule {
//!     // ...
//!     fn workers(&self) -> Vec<Box<dyn BackgroundWorker>> {
//!         vec![Box::new(self.ctx.retry_worker.clone())]
//!     }
//! }
//!
//! impl ServiceModuleTrait for CatalogAdminModule {
//!     // ...
//!     fn grpc(&self, routes: &mut RoutesBuilder) {
//!         let ctx = &self.ctx;
//!         routes.add_service(CatalogEntityServiceServer::new(CatalogEntityGrpc::new(
//!             ctx.catalog_svc.clone(),
//!             ctx.oauth_validator.clone(),
//!         )));
//!     }
//!
//!     fn grpc_descriptors(&self) -> Vec<&'static [u8]> {
//!         vec![crate::grpc::api::FILE_DESCRIPTOR_SET]
//!     }
//! }
//! ```
//!
//! ## 3. Grouping modules
//!
//! A [`ModuleGroup`] collects modules and is a module itself, so groups nest. Every hook is the
//! concatenation of its children's, in registration order. `with_prefix` nests all the
//! children's HTTP surfaces under one path.
//!
//! ```rust,ignore
//! use common::module_loader::module_group::ModuleGroup;
//!
//! let modules = ModuleGroup::new("catalog-agent")
//!     .register(DspModule::build(ctx.clone()).await?)
//!     .register(CatalogAdminModule::new(ctx.clone()))
//!     .register(connector);
//!
//! // Inside the owning module, delegate each hook to the group.
//! fn http(&self) -> Option<(String, Router)> {
//!     self.modules.http()
//! }
//! ```
//!
//! ## 4. Composing a process
//!
//! An agent's `BootstrapServiceTrait::compose` returns a [`ServiceComposer`] with every module it
//! hosts. The boot then reads one router, one set of gRPC routes, the workers and the seeders
//! from it. Auth ports, when set, back the process-wide well-known surface.
//!
//! ```rust,ignore
//! use common::module_loader::service_composer::ServiceComposer;
//!
//! async fn compose(config: &TransferConfig, root: &RootContext) -> Outcome<ServiceComposer> {
//!     let ports = TransferPorts::remote(config, root).await?;
//!     Ok(ServiceComposer::new()
//!         .register(OAuthModule::compose(config.common(), root, None))
//!         .register(TransferAgentModule::compose(config, root, None, &ports))
//!         .with_auth_ports(ports.auth.clone()))
//! }
//!
//! let router = composer.http_router();
//! let routes = composer.grpc_routes();
//! let workers = composer.workers();
//! ```
//!
//! ## 5. The root context
//!
//! [`RootContext`] is built once per process by the boot: one vault, one database pool, one
//! token validator and one service HTTP client. Modules take what they need from it and never
//! open their own connections. The validator comes from a factory because it lives in `oauth`,
//! which `common` cannot depend on.
//!
//! ```rust,ignore
//! let root = RootContext::connect(config.common(), vault, OAuthModule::validator).await?;
//!
//! let repo = SeaOrmSecretRepo::new(root.db.clone());
//! let oauth_validator = root.validator.clone();
//! ```
//!
//! ## 6. Migrations
//!
//! `MigratorTrait::migrations()` is a static function, so an agent concatenates each module's
//! static `migrations()` in foreign-key order without building any module.
//!
//! ```rust,ignore
//! fn migrations() -> Vec<Box<dyn MigrationTrait>> {
//!     [OAuthModule::migrations(), TransferAgentModule::migrations()]
//!         .into_iter()
//!         .flatten()
//!         .collect()
//! }
//! ```

pub mod module_group;
pub mod root_context;
pub mod service_composer;
pub mod service_module;
mod utils;
