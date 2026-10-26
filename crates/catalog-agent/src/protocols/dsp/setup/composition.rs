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

//! The DSP plane as a module: the catalog protocol and the catalog RPC under the DSP path.

use std::sync::Arc;

use axum::Router;
use common::module_loader::service_module::ServiceModuleTrait;

use crate::protocols::dsp::http::dsp::DspRouter;
use crate::protocols::dsp::http::rpc::RpcRouter;
use crate::protocols::dsp::setup::context::DspContext;
use crate::setup::context::AppContext;

pub struct DspModule {
    ctx: Arc<DspContext>,
}

impl DspModule {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self {
            ctx: DspContext::build(app),
        }
    }
}

impl ServiceModuleTrait for DspModule {
    fn name(&self) -> &'static str {
        "dsp-catalog"
    }

    fn http(&self) -> Option<(String, Router)> {
        let router = Router::new()
            .merge(DspRouter::new(self.ctx.clone()).router())
            .merge(RpcRouter::new(self.ctx.clone()).router());
        Some((common::dsp_common::paths::CATALOG.to_string(), router))
    }
}
