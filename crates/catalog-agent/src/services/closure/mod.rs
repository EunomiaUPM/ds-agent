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

//! Loading a resource with what hangs from it and what it links to, in a number of queries
//! that does not grow with the catalog.

pub mod service;

use urn::Urn;
use ymir::errors::Outcome;
use ymir::oauth::OwnerScope;

use crate::entities::resource::CatalogResource;
use crate::entities::resource_set::ResourceSet;

/// The slices of the catalog graph a projection or a diagnosis reads.
#[mockall::automock]
#[async_trait::async_trait]
pub trait ClosureServiceTrait: Send + Sync {
    /// The connector's main catalog with every other catalog `scope` sees, their members,
    /// distributions, linked resources and offers.
    async fn main_catalog(&self, scope: &OwnerScope) -> Outcome<(CatalogResource, ResourceSet)>;
    /// The resource and its closure, if `scope` sees it: a catalog with its members, a dataset
    /// with its distributions, anything with what it links to and its offers.
    async fn resource(
        &self,
        scope: &OwnerScope,
        id: &Urn,
    ) -> Outcome<Option<(CatalogResource, ResourceSet)>>;
}
