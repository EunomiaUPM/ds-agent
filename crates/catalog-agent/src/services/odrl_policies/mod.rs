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

//! ODRL policy management use cases.

pub mod service;

use crate::entities::filters::OdrlPolicyFilter;
use crate::entities::odrl_policies::{NewOdrlPolicyDto, OdrlPolicyDto};
use common::auth::AccessScope;
use common::paginated_spec::{Page, Paginated, Sort};
use urn::Urn;
use ymir::errors::Outcome;

#[mockall::automock]
#[async_trait::async_trait]
pub trait OdrlPolicyServiceTrait: Sync + Send {
    async fn get_all_odrl_offers(
        &self,
        scope: &AccessScope,
        filters: &OdrlPolicyFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<OdrlPolicyDto>>;
    async fn get_batch_odrl_offers(
        &self,
        scope: &AccessScope,
        ids: &[Urn],
    ) -> Outcome<Vec<OdrlPolicyDto>>;
    async fn get_all_odrl_offers_by_entity(
        &self,
        scope: &AccessScope,
        entity: &Urn,
    ) -> Outcome<Vec<OdrlPolicyDto>>;
    async fn get_odrl_offer_by_id(
        &self,
        scope: &AccessScope,
        odrl_offer_id: &Urn,
    ) -> Outcome<OdrlPolicyDto>;
    async fn create_odrl_offer(
        &self,
        scope: &AccessScope,
        new_odrl_offer_model: &NewOdrlPolicyDto,
    ) -> Outcome<OdrlPolicyDto>;
    async fn delete_odrl_offer_by_id(
        &self,
        scope: &AccessScope,
        odrl_offer_id: &Urn,
    ) -> Outcome<()>;
    async fn delete_odrl_offers_by_entity(
        &self,
        scope: &AccessScope,
        entity_id: &Urn,
    ) -> Outcome<()>;
}
