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

//! ODRL offer repository.

use crate::data::entities::odrl_offer;
use common::oauth::OwnerScope;
use crate::data::entities::odrl_offer::NewOdrlOfferModel;
use crate::data::repo_traits::catalog_db_errors::CatalogAgentRepoErrors;
use crate::entities::filters::OdrlPolicyFilter;
use common::paginated_spec::{Page, Sort};
use urn::Urn;
use ymir::errors::Outcome;

/// Persistence of ODRL offers, within the owner scope each call gives.
#[mockall::automock]
#[async_trait::async_trait]
pub trait OdrlOfferRepositoryTrait: Send + Sync {
    /// Page of offers matching the filters, with the total.
    async fn get_all_odrl_offers(
        &self,
        scope: &OwnerScope,
        filters: &OdrlPolicyFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<(Vec<odrl_offer::Model>, Option<u64>)>;
    async fn get_batch_odrl_offers(
        &self,
        scope: &OwnerScope,
        ids: &[Urn],
    ) -> Outcome<Vec<odrl_offer::Model>>;
    /// Offers attached to the entity.
    async fn get_all_odrl_offers_by_entity(
        &self,
        scope: &OwnerScope,
        entity: &Urn,
    ) -> Outcome<Vec<odrl_offer::Model>>;
    async fn get_odrl_offer_by_id(
        &self,
        scope: &OwnerScope,
        odrl_offer_id: &Urn,
    ) -> Outcome<Option<odrl_offer::Model>>;
    async fn create_odrl_offer(
        &self,
        new_odrl_offer_model: &NewOdrlOfferModel,
    ) -> Outcome<odrl_offer::Model>;
    /// Deletes and returns the removed row so callers can evict derived caches.
    async fn delete_odrl_offer_by_id(
        &self,
        scope: &OwnerScope,
        odrl_offer_id: &Urn,
    ) -> Outcome<odrl_offer::Model>;
    /// Deletes every offer of an entity and returns the removed rows.
    async fn delete_odrl_offers_by_entity(
        &self,
        scope: &OwnerScope,
        entity_id: &Urn,
    ) -> Outcome<Vec<odrl_offer::Model>>;
}
