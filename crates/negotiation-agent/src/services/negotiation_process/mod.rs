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

//! Negotiation process management service trait and submodule declarations.

pub mod service;
pub mod views;

use crate::entities::filters::NegotiationProcessFilter;
use crate::entities::negotiation_process::{EditNegotiationProcessDto, NewNegotiationProcessDto};
use crate::services::negotiation_process::views::NegotiationProcessView;
use common::auth::access::AccessScope;
use common::batch_requests::BatchRequests;
use common::paginated_spec::{Page, Paginated, Sort};
use urn::Urn;
use ymir::errors::Outcome;

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait NegotiationProcessServiceTrait: Send + Sync + 'static {
    async fn get_all(
        &self,
        scope: &AccessScope,
        filters: &NegotiationProcessFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<NegotiationProcessView>>;

    async fn get_one(&self, scope: &AccessScope, id: &Urn) -> Outcome<NegotiationProcessView>;

    async fn get_by_key_id(
        &self,
        scope: &AccessScope,
        key_id: &str,
        id: &Urn,
    ) -> Outcome<NegotiationProcessView>;

    async fn get_by_key_value(
        &self,
        scope: &AccessScope,
        value: &Urn,
    ) -> Outcome<NegotiationProcessView>;

    async fn batch(
        &self,
        scope: &AccessScope,
        req: &BatchRequests,
    ) -> Outcome<Vec<NegotiationProcessView>>;

    async fn create(
        &self,
        scope: &AccessScope,
        cmd: &NewNegotiationProcessDto,
    ) -> Outcome<NegotiationProcessView>;

    async fn edit(
        &self,
        scope: &AccessScope,
        id: &Urn,
        cmd: &EditNegotiationProcessDto,
    ) -> Outcome<NegotiationProcessView>;

    async fn delete(&self, scope: &AccessScope, id: &Urn) -> Outcome<()>;
}
