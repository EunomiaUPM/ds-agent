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

//! The DSP plane as a module: the DSP, RPC and BFF routes, the auto-start worker, and the TCK
//! worker in TCK mode.

use std::sync::{Arc, Mutex};

use axum::Router;
use ymir::boot::workers::BackgroundWorker;
use common::dsp_common::paths;
use ymir::module_loader::service_module::ServiceModuleTrait;
use crate::protocols::dsp::http::bff::TransferBffRouter;
use crate::protocols::dsp::http::dsp::DspRouter;
use crate::protocols::dsp::http::rpc::TransferRpcRouter;
use crate::protocols::dsp::setup::context::DspContext;
use crate::setup::context::AppContext;

/// DSP transfer: the inbound flow peers call, the outbound flow users call over RPC and BFF,
/// the auto-start worker and the TCK scenario worker when `DSP_TCK_MAPPING` is set.
pub struct DspModule {
    ctx: Arc<DspContext>,
    workers: Mutex<Vec<Box<dyn BackgroundWorker>>>,
}

impl DspModule {
    pub fn new(app: Arc<AppContext>) -> Self {
        let (ctx, workers) = DspContext::build(app);
        Self {
            ctx,
            workers: Mutex::new(workers),
        }
    }
}

impl ServiceModuleTrait for DspModule {
    fn name(&self) -> &'static str {
        "dsp-transfers"
    }

    /// Peers reach the DSP routes with their grant; users reach the RPC and BFF routes with OAuth.
    fn http(&self) -> Option<(String, Router)> {
        let dsp = DspRouter::new(self.ctx.clone()).router();
        let rpc = TransferRpcRouter::new(self.ctx.clone())
            .router()
            .merge(TransferBffRouter::new(self.ctx.clone()).router())
            .route_layer(axum::middleware::from_fn_with_state(
                self.ctx.app.oauth_validator.clone(),
                ymir::http::OauthHttpMiddleware::run,
            ));
        let mut routes = rpc.merge(dsp);
        // The TCK trigger is a test hook without user auth: only served in TCK mode.
        if self.ctx.tck_mode() {
            routes = routes.merge(TransferRpcRouter::new(self.ctx.clone()).tck_router());
        }
        Some((paths::TRANSFERS.to_string(), routes))
    }

    fn workers(&self) -> Vec<Box<dyn BackgroundWorker>> {
        std::mem::take(
            &mut *self
                .workers
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        )
    }
}
