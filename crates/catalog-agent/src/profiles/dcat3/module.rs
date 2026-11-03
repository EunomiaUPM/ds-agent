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

//! DCAT 3 as a profile module: no routes of its own, only the adapter the registry, the
//! exports and the projections use.

use std::sync::Arc;

use ymir::module_loader::service_module::ServiceModuleTrait;

use crate::profiles::dcat3::Dcat3;
use crate::profiles::shared::dcat::DcatAdapter;
use crate::profiles::{ProfileAdapterTrait, ProfileModuleTrait};

pub struct Dcat3ProfileModule {
    adapter: Arc<DcatAdapter<Dcat3>>,
}

impl Default for Dcat3ProfileModule {
    fn default() -> Self {
        Self {
            adapter: Arc::new(DcatAdapter::default()),
        }
    }
}

impl ServiceModuleTrait for Dcat3ProfileModule {
    fn name(&self) -> &'static str {
        "catalog-profile-dcat3"
    }
}

impl ProfileModuleTrait for Dcat3ProfileModule {
    fn adapter(&self) -> Arc<dyn ProfileAdapterTrait> {
        self.adapter.clone()
    }
}
