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

//! ODRL offer management use cases.

pub mod service;
pub mod views;

use urn::Urn;
use ymir::errors::Outcome;
use ymir::paginated_spec::{Page, Paginated, Sort};
use ymir::types::oauth::UserInfo;

use crate::entities::commands::NewOdrlOfferCommand;
use crate::entities::filters::OdrlPolicyFilter;
use crate::entities::odrl_offer::OdrlOffer;

/// Management of the ODRL offers attached to catalog resources.
#[mockall::automock]
#[async_trait::async_trait]
pub trait OdrlPolicyServiceTrait: Sync + Send {
    /// Page of offers visible to the caller.
    async fn list(
        &self,
        user: &UserInfo,
        filter: &OdrlPolicyFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<OdrlOffer>>;
    async fn batch(&self, user: &UserInfo, ids: &[Urn]) -> Outcome<Vec<OdrlOffer>>;
    /// Offers attached to any of the entities, in one query.
    async fn by_entities(&self, user: &UserInfo, entities: &[Urn]) -> Outcome<Vec<OdrlOffer>>;
    /// 404 when the offer is not visible to the caller.
    async fn get(&self, user: &UserInfo, id: &Urn) -> Outcome<OdrlOffer>;
    /// Stores the offer on a resource the caller sees, of the kind it names.
    async fn create(&self, user: &UserInfo, command: &NewOdrlOfferCommand) -> Outcome<OdrlOffer>;
    async fn delete(&self, user: &UserInfo, id: &Urn) -> Outcome<()>;
    /// Deletes every offer attached to the entity.
    async fn delete_by_entity(&self, user: &UserInfo, entity: &Urn) -> Outcome<()>;
}
