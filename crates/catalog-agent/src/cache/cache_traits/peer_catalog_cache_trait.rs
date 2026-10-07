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

use crate::protocols::dsp::types::catalog_definition::Catalog;
use ymir::errors::Outcome;

#[mockall::automock]
#[async_trait::async_trait]
pub trait PeerCatalogCacheTrait: Sync + Send {
    /// Catalogs are cached per user: a peer may expose a different catalog to each (each one
    /// calls it with its own token).
    async fn get_catalog(&self, user_id: &str, participant_id: &str) -> Outcome<Option<Catalog>>;
    async fn set_catalog(
        &self,
        user_id: &str,
        participant_id: &str,
        catalog: &Catalog,
    ) -> Outcome<()>;
}
