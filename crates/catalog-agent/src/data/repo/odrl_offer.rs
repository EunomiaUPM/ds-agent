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

//! Persistence of ODRL offers.

use ymir::oauth::OwnerScope;
use ymir::paginated_spec::{Page, Sort};
use thiserror::Error;
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

use crate::entities::filters::OdrlPolicyFilter;
use crate::entities::odrl_offer::OdrlOffer;

/// Offers within the owner scope each call gives.
#[mockall::automock]
#[async_trait::async_trait]
pub trait OdrlOfferRepoTrait: Send + Sync {
    async fn get_page(
        &self,
        scope: &OwnerScope,
        filter: &OdrlPolicyFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Vec<OdrlOffer>>;
    async fn count(&self, scope: &OwnerScope, filter: &OdrlPolicyFilter) -> Outcome<u64>;
    async fn get_batch(&self, scope: &OwnerScope, ids: &[Urn]) -> Outcome<Vec<OdrlOffer>>;
    /// Offers attached to any of the entities, in one query.
    async fn get_by_entities(&self, scope: &OwnerScope, entities: &[Urn]) -> Outcome<Vec<OdrlOffer>>;
    async fn get_by_id(&self, scope: &OwnerScope, id: &Urn) -> Outcome<Option<OdrlOffer>>;
    /// Stores the offer and the template versions, `(id, version)`, it was composed from.
    async fn insert(&self, offer: &OdrlOffer, references: &[(String, String)]) -> Outcome<OdrlOffer>;
    /// Ids of the offers the scope sees that were composed from the template version.
    async fn using_template(&self, scope: &OwnerScope, id: &str, version: &str) -> Outcome<Vec<Urn>>;
    /// Removes the offer and returns it.
    async fn delete(&self, scope: &OwnerScope, id: &Urn) -> Outcome<OdrlOffer>;
    /// Removes every offer of the entity and returns them.
    async fn delete_by_entity(&self, scope: &OwnerScope, entity: &Urn) -> Outcome<Vec<OdrlOffer>>;
}

#[derive(Debug, Error)]
pub enum OdrlOfferRepoErrors {
    #[error("Offer not found")]
    NotFound,
    #[error("Invalid pagination cursor")]
    InvalidCursor,
    #[error("Error fetching offers. {0}")]
    Fetching(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error creating offer. {0}")]
    Creating(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error deleting offer. {0}")]
    Deleting(Box<dyn std::error::Error + Send + Sync>),
}

impl RepoIntoErrors for OdrlOfferRepoErrors {}
