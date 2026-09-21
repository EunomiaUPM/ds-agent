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

use crate::data::entities::odrl_offer;
use crate::data::entities::odrl_offer::NewOdrlOfferModel;
use crate::data::repo_traits::catalog_db_errors::CatalogAgentRepoErrors;
use crate::entities::filters::OdrlPolicyFilter;
use common::paginated_spec::{Page, Sort};
use urn::Urn;
use ymir::errors::Outcome;

#[mockall::automock]
#[async_trait::async_trait]
pub trait OdrlOfferRepositoryTrait: Send + Sync {
    async fn get_all_odrl_offers(
        &self,
        filters: &OdrlPolicyFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<(Vec<odrl_offer::Model>, Option<u64>)>;
    async fn get_batch_odrl_offers(
        &self,
        tenant_id: &str,
        ids: &[Urn],
    ) -> Outcome<Vec<odrl_offer::Model>>;
    async fn get_all_odrl_offers_by_entity(
        &self,
        tenant_id: &str,
        entity: &Urn,
    ) -> Outcome<Vec<odrl_offer::Model>>;
    async fn get_odrl_offer_by_id(
        &self,
        tenant_id: &str,
        odrl_offer_id: &Urn,
    ) -> Outcome<Option<odrl_offer::Model>>;
    async fn create_odrl_offer(
        &self,
        new_odrl_offer_model: &NewOdrlOfferModel,
    ) -> Outcome<odrl_offer::Model>;
    /// Deletes and returns the removed row so callers can evict derived caches.
    async fn delete_odrl_offer_by_id(
        &self,
        tenant_id: &str,
        odrl_offer_id: &Urn,
    ) -> Outcome<odrl_offer::Model>;
    /// Deletes every offer of an entity and returns the removed rows.
    async fn delete_odrl_offers_by_entity(
        &self,
        tenant_id: &str,
        entity_id: &Urn,
    ) -> Outcome<Vec<odrl_offer::Model>>;
}
