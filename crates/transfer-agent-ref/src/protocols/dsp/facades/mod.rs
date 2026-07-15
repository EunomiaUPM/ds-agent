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

use crate::protocols::dsp::facades::data_service_resolver_facade::DataServiceFacadeTrait;
use crate::protocols::dsp::facades::dataplane_facade::DataPlaneFacadeTrait;
use std::sync::Arc;

pub mod data_service_resolver_facade;
pub mod dataplane_facade;

#[async_trait::async_trait]
pub trait FacadeTrait: Send + Sync {
    async fn get_data_service_facade(&self) -> Arc<dyn DataServiceFacadeTrait>;
    async fn get_data_plane_facade(&self) -> Arc<dyn DataPlaneFacadeTrait>;
}

pub struct FacadeService {
    data_service_resolver_facade: Arc<dyn DataServiceFacadeTrait>,
    data_plane_facade: Arc<dyn DataPlaneFacadeTrait>,
}

impl FacadeService {
    pub fn new(
        data_service_resolver_facade: Arc<dyn DataServiceFacadeTrait>,
        data_plane_facade: Arc<dyn DataPlaneFacadeTrait>,
    ) -> FacadeService {
        Self {
            data_service_resolver_facade,
            data_plane_facade,
        }
    }
}

#[async_trait::async_trait]
impl FacadeTrait for FacadeService {
    async fn get_data_service_facade(&self) -> Arc<dyn DataServiceFacadeTrait> {
        self.data_service_resolver_facade.clone()
    }

    async fn get_data_plane_facade(&self) -> Arc<dyn DataPlaneFacadeTrait> {
        self.data_plane_facade.clone()
    }
}
