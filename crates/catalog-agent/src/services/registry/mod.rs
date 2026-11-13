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

//! Registering the published datasets with the auth agent (GNAP RS).

pub mod service;

use ymir::errors::Outcome;

use crate::entities::resource::CatalogResource;

/// Keeps auth's registry equal to what the catalog publishes.
#[mockall::automock]
#[async_trait::async_trait]
pub trait RegistryServiceTrait: Send + Sync {
    /// Sends auth every dataset of the connector's catalogs, with its catalog, location and owner.
    async fn sync(&self) -> Outcome<()>;
    /// [`sync`](Self::sync) after boot or a catalog-wide change: failures are logged and repaired later.
    async fn resync(&self);
    /// Incrementally upserts or unregisters a saved `Dataset` in auth's registry; skips other kinds.
    async fn sync_saved(&self, resource: &CatalogResource);
    /// Unregisters a deleted `Dataset`, or runs [`resync`](Self::resync) if a `Catalog` cascaded.
    async fn sync_deleted(&self, resource: &CatalogResource);
    /// Syncs datasets written by a catalog import without reloading the entire catalog closure.
    async fn sync_imported(&self, resources: &[CatalogResource]);
}
