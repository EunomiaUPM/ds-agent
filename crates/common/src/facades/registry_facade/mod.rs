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

//! The catalog registering its datasets with the auth agent, so the AS decides what a grant gives
//! without asking the catalog (GNAP RS).

pub mod remote;

use async_trait::async_trait;
use ymir::data::entities::shared::registered_dataset::Model as RegisteredDataset;
use ymir::errors::Outcome;

/// What the catalog tells the auth agent about the datasets it publishes.
#[mockall::automock]
#[async_trait]
pub trait RegistryFacadeTrait: Send + Sync {
    /// Leaves exactly `datasets` registered: every published dataset, each time.
    async fn sync_datasets(&self, datasets: Vec<RegisteredDataset>) -> Outcome<()>;
}
