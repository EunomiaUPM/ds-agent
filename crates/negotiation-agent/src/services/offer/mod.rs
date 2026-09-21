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
use common::auth::access::AccessScope;
use common::batch_requests::BatchRequests;
use common::paginated_spec::{Page, Paginated, Sort};
use urn::Urn;
use ymir::errors::Outcome;

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait OfferServiceTrait: Send + Sync + 'static {
    async fn get_all(
        &self,
        scope: &AccessScope,
        filters: &OfferFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<OfferView>>;

    async fn get_one(&self, scope: &AccessScope, id: &Urn) -> Outcome<OfferView>;

    async fn get_by_negotiation_message(
        &self,
        scope: &AccessScope,
        message_id: &Urn,
    ) -> Outcome<OfferView>;

    async fn get_by_offer_id(&self, scope: &AccessScope, offer_id: &Urn) -> Outcome<OfferView>;

    async fn get_by_process(
        &self,
        scope: &AccessScope,
        process_id: &Urn,
    ) -> Outcome<Vec<OfferView>>;

    async fn batch(&self, scope: &AccessScope, req: &BatchRequests) -> Outcome<Vec<OfferView>>;

    async fn create(&self, scope: &AccessScope, cmd: &NewOfferDto) -> Outcome<OfferView>;

    async fn delete(&self, scope: &AccessScope, id: &Urn) -> Outcome<()>;
}
