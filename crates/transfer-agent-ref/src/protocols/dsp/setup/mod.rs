/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 */

use std::sync::Arc;

use crate::protocols::dsp::http::dsp::DspRouter;
use crate::setup::context::ModuleCtx;
use axum::Router;
use common::config::services::traits::TransferConfigTrait;
use common::facades::ssi_auth_facade::ssi_auth_facade::SSIAuthFacadeService;
use common::http_client::HttpClient;
use common::module_loader::service_module::ServiceModuleTrait;

const DSP_BASE_PATH: &str = "/dsp/current/transfers";

pub(crate) struct DspModule;

impl ServiceModuleTrait<ModuleCtx> for DspModule {
    fn name(&self) -> &'static str {
        "dsp-transfers"
    }

    fn http(&self, ctx: &ModuleCtx) -> Option<(String, Router)> {
        let http_client = Arc::new(HttpClient::new(10, 10));
        let ssi_auth = Arc::new(SSIAuthFacadeService::new(
            Arc::new(ctx.config.ssi_auth().clone()),
            http_client,
        ));
        Some((DSP_BASE_PATH.to_string(), DspRouter::new(ssi_auth).router()))
    }
}
