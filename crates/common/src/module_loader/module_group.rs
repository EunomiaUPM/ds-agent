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

use crate::module_loader::service_module::ServiceModuleTrait;
use crate::module_loader::utils::mount;
use axum::Router;
use sea_orm_migration::MigrationTrait;
use tonic::service::RoutesBuilder;

/// A named collection of modules that is itself a [`ServiceModuleTrait`] — the
/// composite that makes composition recursive: groups can contain groups.
/// An optional `prefix` nests every child's HTTP surface under it.
pub struct ModuleGroup<Ctx> {
    name: &'static str,
    prefix: Option<String>,
    modules: Vec<Box<dyn ServiceModuleTrait<Ctx>>>,
}

impl<Ctx> ModuleGroup<Ctx> {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            prefix: None,
            modules: Vec::new(),
        }
    }

    /// Nest all children's HTTP surfaces under `prefix` (e.g. `/dsp`).
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// Add a module — or another group. Chainable.
    pub fn register(mut self, module: impl ServiceModuleTrait<Ctx> + 'static) -> Self {
        self.modules.push(Box::new(module));
        self
    }
}

impl<Ctx> ServiceModuleTrait<Ctx> for ModuleGroup<Ctx> {
    fn name(&self) -> &'static str {
        self.name
    }

    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        self.modules.iter().flat_map(|m| m.migrations()).collect()
    }

    fn http(&self, ctx: &Ctx) -> Option<(String, Router)> {
        let mut router = Router::new();
        let mut mounted = false;
        for module in &self.modules {
            if let Some((path, sub)) = module.http(ctx) {
                tracing::debug!(
                    group = self.name,
                    module = module.name(),
                    path,
                    "mounting HTTP module"
                );
                router = mount(router, &path, sub);
                mounted = true;
            }
        }
        mounted.then(|| (self.prefix.clone().unwrap_or_default(), router))
    }

    fn grpc(&self, ctx: &Ctx, routes: &mut RoutesBuilder) {
        for module in &self.modules {
            module.grpc(ctx, routes);
        }
    }
}
