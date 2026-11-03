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

//! The public references as a module, mounted at `/open` of every agent process.

use std::sync::Arc;

use axum::Router;
use ymir::config::types::{CommonConfig, HostType};
use ymir::module_loader::service_module::ServiceModuleTrait;

use crate::config::types::min_known_config::MinKnownConfig;
use crate::config::types::traits::MinKnownConfigTrait;
use crate::public_refs::{PublicRefsRouter, PublicResolverTrait};
use crate::routes;

/// The public references of the agents in this process.
pub struct PublicRefsModule {
    router: Router,
}

impl PublicRefsModule {
    /// Canonical URLs hang from the process's exterior HTTP host in `common`.
    pub fn new(common: &CommonConfig, resolvers: Vec<Arc<dyn PublicResolverTrait>>) -> Self {
        let base = MinKnownConfig::from(common).get_host(HostType::Http);
        Self {
            router: PublicRefsRouter::new(base, resolvers).router(),
        }
    }
}

impl ServiceModuleTrait for PublicRefsModule {
    fn name(&self) -> &'static str {
        "public-refs"
    }

    fn http(&self) -> Option<(String, Router)> {
        Some((routes::public_refs::PREFIX.to_string(), self.router.clone()))
    }
}
