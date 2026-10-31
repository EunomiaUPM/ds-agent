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

//! DCAT-AP as a module: its public read routes and its adapter.

use std::sync::Arc;

use axum::Router;
use ymir::config::traits::CommonConfigTrait;
use ymir::module_loader::service_module::ServiceModuleTrait;
use ymir::config::traits::ApiConfigTrait;

use crate::profiles::dcat_ap::DcatAp;
use crate::profiles::shared::dcat::{DcatAdapter, DcatProfileSpec};
use crate::profiles::shared::public::PublicProfileRouter;
use crate::profiles::{ProfileAdapterTrait, ProfileModuleTrait};
use crate::setup::context::AppContext;
use crate::SERVICE_NAME;

pub struct DcatApProfileModule {
    ctx: Arc<AppContext>,
    adapter: Arc<DcatAdapter<DcatAp>>,
}

impl DcatApProfileModule {
    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self {
            ctx,
            adapter: Arc::new(DcatAdapter::default()),
        }
    }
}

impl ServiceModuleTrait for DcatApProfileModule {
    fn name(&self) -> &'static str {
        "catalog-profile-dcat-ap"
    }

    fn http(&self) -> Option<(String, Router)> {
        let base = format!(
            "{}/{SERVICE_NAME}/public/{}",
            self.ctx.config.common().get_api_version(),
            DcatAp::KEY
        );
        Some((base, PublicProfileRouter::new(self.ctx.clone(), self.adapter()).router()))
    }
}

impl ProfileModuleTrait for DcatApProfileModule {
    fn adapter(&self) -> Arc<dyn ProfileAdapterTrait> {
        self.adapter.clone()
    }
}
