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

use axum::Router;

use crate::module_loader::service_module::ServiceModuleTrait;

/// Wraps an agent's already-built HTTP router until the agent grows a real module;
/// the router is merged at the level where the wrapper is registered.
pub struct ToBeDeprecatedRouterModule {
    name: &'static str,
    router: Router,
}

impl ToBeDeprecatedRouterModule {
    pub fn merged(name: &'static str, router: Router) -> Self {
        Self { name, router }
    }
}

impl ServiceModuleTrait for ToBeDeprecatedRouterModule {
    fn name(&self) -> &'static str {
        self.name
    }

    fn http(&self) -> Option<(String, Router)> {
        Some((String::new(), self.router.clone()))
    }
}
