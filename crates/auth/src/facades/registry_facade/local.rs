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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

//! The catalog registering its datasets with the gatekeeper in-process.

use std::sync::Arc;

use async_trait::async_trait;
use common::facades::registry_facade::RegistryFacadeTrait;
use ymir::data::entities::shared::registered_dataset::Model as RegisteredDataset;
use ymir::errors::Outcome;

use crate::modules::GateKeeperModule;

/// Registry port served by the gatekeeper in the same process.
pub struct RegistryLocalFacade {
    gatekeeper: Arc<dyn GateKeeperModule>,
}

impl RegistryLocalFacade {
    pub fn new(gatekeeper: Arc<dyn GateKeeperModule>) -> Self {
        Self { gatekeeper }
    }
}

#[async_trait]
impl RegistryFacadeTrait for RegistryLocalFacade {
    async fn sync_datasets(&self, datasets: Vec<RegisteredDataset>) -> Outcome<()> {
        self.gatekeeper.register_datasets(datasets).await
    }
}
