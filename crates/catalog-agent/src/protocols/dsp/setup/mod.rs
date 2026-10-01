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

use crate::protocols::dsp::CatalogDSP;
use crate::protocols::protocol::ProtocolPluginTrait;
use crate::setup::context::AppContext;
use axum::Router;
use common::module_loader::service_module::ServiceModuleTrait;
use ymir::errors::Outcome;

const DSP_BASE_PATH: &str = "/dsp/current/catalog";

pub(crate) struct DspModule {
    router: Router,
}

impl DspModule {
    pub async fn build(ctx: Arc<AppContext>) -> Outcome<Self> {
        let router = CatalogDSP::new(
            ctx.catalog_svc.clone(),
            ctx.data_service_svc.clone(),
            ctx.dataset_svc.clone(),
            ctx.odrl_policy_svc.clone(),
            ctx.distribution_svc.clone(),
            ctx.peer_catalog_svc.clone(),
            ctx.mates_facade.clone(),
            ctx.ssi_auth_facade.clone(),
            ctx.config.clone(),
            ctx.oauth_validator.clone(),
        )
        .build_router()
        .await?;
        Ok(Self { router })
    }
}

impl ServiceModuleTrait for DspModule {
    fn name(&self) -> &'static str {
        "dsp-catalog"
    }

    fn http(&self) -> Option<(String, Router)> {
        Some((DSP_BASE_PATH.to_string(), self.router.clone()))
    }
}
