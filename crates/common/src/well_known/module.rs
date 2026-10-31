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

use std::sync::Arc;

use axum::Router;
use ymir::config::types::CommonConfig;
use ymir::errors::Outcome;
use ymir::module_loader::service_module::ServiceModuleTrait;

use crate::config::types::min_known_config::MinKnownConfig;
use crate::facades::mates_facade::MatesFacadeTrait;
use crate::well_known::WellKnownRoot;

/// The well-known surface as a module, mounted at the root of every agent process.
pub struct WellKnownModule {
    router: Router,
}

impl WellKnownModule {
    /// `mates` is the process's resolved port; without one, participants are read over HTTP.
    pub fn new(common: &CommonConfig, mates: Option<Arc<dyn MatesFacadeTrait>>) -> Outcome<Self> {
        let router = WellKnownRoot::get_well_known_router(&MinKnownConfig::from(common), mates)?;
        Ok(Self { router })
    }
}

impl ServiceModuleTrait for WellKnownModule {
    fn name(&self) -> &'static str {
        "well-known"
    }

    fn http(&self) -> Option<(String, Router)> {
        Some((String::new(), self.router.clone()))
    }
}
