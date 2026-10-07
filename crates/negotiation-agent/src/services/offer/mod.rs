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

//! Offer management service trait and submodule declarations.

pub mod service;
pub mod views;

use crate::entities::filters::OfferFilter;
use crate::entities::offer::NewOfferDto;
use crate::services::offer::views::OfferView;
use common::oauth::UserInfo;
use common::batch_requests::BatchRequests;
use common::paginated_spec::{Page, Paginated, Sort};
use urn::Urn;
use ymir::errors::Outcome;

/// Management of offers.
#[mockall::automock]
#[async_trait::async_trait]
pub trait OfferServiceTrait: Send + Sync + 'static {
    /// Page of offers visible to the caller.
    async fn get_all(
        &self,
        user: &UserInfo,
        filters: &OfferFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<OfferView>>;

    /// 404 when the offer is not visible to the caller.
    async fn get_one(&self, user: &UserInfo, id: &Urn) -> Outcome<OfferView>;

    /// Offer carried by the message.
    async fn get_by_negotiation_message(
        &self,
        user: &UserInfo,
        message_id: &Urn,
    ) -> Outcome<OfferView>;

    /// Offer by its ODRL `@id`.
    async fn get_by_offer_id(&self, user: &UserInfo, offer_id: &Urn) -> Outcome<OfferView>;

    /// Every offer made in the process.
    async fn get_by_process(
        &self,
        user: &UserInfo,
        process_id: &Urn,
    ) -> Outcome<Vec<OfferView>>;

    /// Most recent offer of the process.
    async fn get_last_by_process(
        &self,
        user: &UserInfo,
        process_id: &Urn,
    ) -> Outcome<OfferView>;

    /// Offers found among the requested ids.
    async fn batch(&self, user: &UserInfo, req: &BatchRequests) -> Outcome<Vec<OfferView>>;

    async fn create(&self, user: &UserInfo, cmd: &NewOfferDto) -> Outcome<OfferView>;

    async fn delete(&self, user: &UserInfo, id: &Urn) -> Outcome<()>;
}
