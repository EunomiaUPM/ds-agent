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

use crate::protocols::dsp::http::dsp::DspRouter;
use crate::setup::context::AppContext;
use axum::Router;
use common::module_loader::service_module::ServiceModuleTrait;

const DSP_BASE_PATH: &str = "/dsp/current/transfers";

pub(crate) struct DspModule {
    ctx: Arc<AppContext>,
}

impl DspModule {
    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self { ctx }
    }
}

impl ServiceModuleTrait for DspModule {
    fn name(&self) -> &'static str {
        "dsp-transfers"
    }

    fn http(&self) -> Option<(String, Router)> {
        Some((
            DSP_BASE_PATH.to_string(),
            DspRouter::new(self.ctx.ssi_auth_facade.clone()).router(),
        ))
    }
}
