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

use crate::protocols::dsp::NegotiationDSP;
use crate::protocols::protocol::ProtocolPluginTrait;
use crate::setup::context::AppContext;
use axum::Router;
use common::module_loader::service_module::ServiceModuleTrait;
use ymir::errors::Outcome;

const DSP_BASE_PATH: &str = "/dsp/current/negotiations";

pub(crate) struct DspModule {
    router: Router,
}

impl DspModule {
    pub async fn build(ctx: Arc<AppContext>) -> Outcome<Self> {
        let router = NegotiationDSP::new(
            ctx.process_repo.clone(),
            ctx.process_svc.clone(),
            ctx.message_svc.clone(),
            ctx.offer_svc.clone(),
            ctx.agreement_svc.clone(),
            ctx.config.clone(),
            ctx.ssi_auth_facade.clone(),
            ctx.mates_facade.clone(),
            ctx.oauth_validator.clone(),
        )
        .build_router()
        .await?;
        Ok(Self { router })
    }
}

impl ServiceModuleTrait for DspModule {
    fn name(&self) -> &'static str {
        "dsp-negotiations"
    }

    fn http(&self) -> Option<(String, Router)> {
        Some((DSP_BASE_PATH.to_string(), self.router.clone()))
    }
}
