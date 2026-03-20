/*
 *
 *  * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
 *  *
 *  * This program is free software: you can redistribute it and/or modify
 *  * it under the terms of the GNU General Public License as published by
 *  * the Free Software Foundation, either version 3 of the License, or
 *  * (at your option) any later version.
 *  *
 *  * This program is distributed in the hope that it will be useful,
 *  * but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  * GNU General Public License for more details.
 *  *
 *  * You should have received a copy of the GNU General Public License
 *  * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */
use crate::protocols::dsp::facades::well_known_rpc_facade::WellKnownRPCFacadeTrait;
use std::sync::Arc;

pub(crate) mod well_known_rpc_facade;

#[async_trait::async_trait]
pub trait FacadeTrait: Send + Sync {
    async fn get_catalog_rpc_path_facade(&self) -> Arc<dyn WellKnownRPCFacadeTrait>;
}

pub struct FacadeService {
    catalog_rpc_path_facade: Arc<dyn WellKnownRPCFacadeTrait>,
}

impl FacadeService {
    pub fn new(catalog_rpc_path_facade: Arc<dyn WellKnownRPCFacadeTrait>) -> FacadeService {
        Self { catalog_rpc_path_facade }
    }
}

#[async_trait::async_trait]
impl FacadeTrait for FacadeService {
    async fn get_catalog_rpc_path_facade(&self) -> Arc<dyn WellKnownRPCFacadeTrait> {
        self.catalog_rpc_path_facade.clone()
    }
}
