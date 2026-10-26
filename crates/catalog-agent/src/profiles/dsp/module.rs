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

//! DSP as a profile module: no public routes (the DSP routes are the protocol plane's, behind
//! peer authentication), only the adapter the registry and the protocol plane use.

use std::sync::Arc;

use common::module_loader::service_module::ServiceModuleTrait;

use crate::profiles::dsp::DspProfileAdapter;
use crate::profiles::{ProfileAdapterTrait, ProfileModuleTrait};

pub struct DspProfileModule {
    adapter: Arc<DspProfileAdapter>,
}

impl Default for DspProfileModule {
    fn default() -> Self {
        Self {
            adapter: Arc::new(DspProfileAdapter::new()),
        }
    }
}

impl ServiceModuleTrait for DspProfileModule {
    fn name(&self) -> &'static str {
        "catalog-profile-dsp"
    }
}

impl ProfileModuleTrait for DspProfileModule {
    fn adapter(&self) -> Arc<dyn ProfileAdapterTrait> {
        self.adapter.clone()
    }
}
