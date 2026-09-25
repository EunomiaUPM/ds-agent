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

use crate::boot::seeders::BootSeeder;
use crate::boot::workers::BackgroundWorker;
use crate::facades::AuthPorts;
use crate::module_loader::module_group::ModuleGroup;
use crate::module_loader::service_module::ServiceModuleTrait;
use crate::module_loader::utils::mount;
use axum::Router;
use sea_orm_migration::MigrationTrait;
use tonic::service::{Routes, RoutesBuilder};

/// Root of the composition: a [`ModuleGroup`] of already-wired modules,
/// exposing each composed plane.
pub struct ServiceComposer {
    root: ModuleGroup,
    auth_ports: Option<AuthPorts>,
}

impl ServiceComposer {
    pub fn new() -> Self {
        Self {
            root: ModuleGroup::new("root"),
            auth_ports: None,
        }
    }

    /// Auth ports this process resolved (local or remote), reused by the process-wide surfaces.
    pub fn with_auth_ports(mut self, ports: AuthPorts) -> Self {
        self.auth_ports = Some(ports);
        self
    }

    pub fn auth_ports(&self) -> Option<&AuthPorts> {
        self.auth_ports.as_ref()
    }

    /// Add a module — or a whole [`ModuleGroup`]. Chainable; composition
    /// order = registration order.
    pub fn register(mut self, module: impl ServiceModuleTrait + 'static) -> Self {
        self.root = self.root.register(module);
        self
    }

    /// All modules' migrations, concatenated in registration order.
    pub fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        self.root.migrations()
    }

    /// One axum router with every module's HTTP surface mounted.
    pub fn http_router(&self) -> Router {
        match self.root.http() {
            Some((path, router)) => mount(Router::new(), &path, router),
            None => Router::new(),
        }
    }

    /// One tonic [`Routes`] with every module's gRPC services registered.
    pub fn grpc_routes(&self) -> Routes {
        let mut builder = RoutesBuilder::default();
        self.root.grpc(&mut builder);
        builder.routes()
    }

    /// Every module's encoded file descriptor set, for the reflection service.
    pub fn grpc_descriptors(&self) -> Vec<&'static [u8]> {
        self.root.grpc_descriptors()
    }

    /// Every module's background workers, in registration order.
    pub fn workers(&self) -> Vec<Box<dyn BackgroundWorker>> {
        self.root.workers()
    }

    /// Every module's boot seeders, in registration order.
    pub fn seeders(&self) -> Vec<Box<dyn BootSeeder>> {
        self.root.seeders()
    }
}

impl Default for ServiceComposer {
    fn default() -> Self {
        Self::new()
    }
}
