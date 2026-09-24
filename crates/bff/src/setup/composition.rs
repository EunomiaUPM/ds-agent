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
use common::module_loader::service_module::ServiceModuleTrait;

use crate::gateway::GatewayHttpRouter;
use crate::setup::context::AppContext;

pub const SERVICE_NAME: &str = "gateway";

/// Composable service module integrating the BFF gateway and admin frontend.
pub struct BffModule {
    ctx: Arc<AppContext>,
}

impl BffModule {
    /// Construct a new BffModule with application context.
    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self { ctx }
    }
}

impl ServiceModuleTrait for BffModule {
    fn name(&self) -> &'static str {
        SERVICE_NAME
    }

    fn http(&self) -> Option<(String, Router)> {
        let router = GatewayHttpRouter::new(self.ctx.clone()).router();
        Some((String::new(), Router::new().nest("/admin", router)))
    }
}
