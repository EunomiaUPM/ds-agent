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

//! Registering the published datasets with the auth agent (GNAP RS): all of them, each time.

pub mod service;

use ymir::errors::Outcome;

/// Keeps auth's registry equal to what the catalog publishes.
#[mockall::automock]
#[async_trait::async_trait]
pub trait RegistryServiceTrait: Send + Sync {
    /// Sends auth every dataset of the connector's catalogs, with its catalog, location and owner.
    async fn sync(&self) -> Outcome<()>;
    /// [`sync`](Self::sync) after a write or at boot: a failure is logged and the next one repairs
    /// it.
    async fn resync(&self);
}
