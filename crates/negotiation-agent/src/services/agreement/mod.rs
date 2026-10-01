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

//! Agreement management service trait and submodule declarations.

pub mod service;
pub mod views;

use crate::entities::agreement::{EditAgreementDto, NewAgreementDto};
use crate::entities::filters::AgreementFilter;
use crate::services::agreement::views::AgreementView;
use common::auth::access::AccessScope;
use common::batch_requests::BatchRequests;
use common::paginated_spec::{Page, Paginated, Sort};
use urn::Urn;
use ymir::errors::Outcome;

#[mockall::automock]
#[async_trait::async_trait]
pub trait AgreementServiceTrait: Send + Sync + 'static {
    async fn get_all(
        &self,
        scope: &AccessScope,
        filters: &AgreementFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<AgreementView>>;

    async fn get_one(&self, scope: &AccessScope, id: &Urn) -> Outcome<AgreementView>;

    async fn get_by_process(&self, scope: &AccessScope, process_id: &Urn)
    -> Outcome<AgreementView>;

    async fn get_by_message(&self, scope: &AccessScope, message_id: &Urn)
    -> Outcome<AgreementView>;

    async fn get_by_assignee(
        &self,
        scope: &AccessScope,
        assignee: &str,
    ) -> Outcome<Vec<AgreementView>>;

    async fn get_by_assigner(
        &self,
        scope: &AccessScope,
        assigner: &str,
    ) -> Outcome<Vec<AgreementView>>;

    async fn batch(&self, scope: &AccessScope, req: &BatchRequests) -> Outcome<Vec<AgreementView>>;

    async fn create(&self, scope: &AccessScope, cmd: &NewAgreementDto) -> Outcome<AgreementView>;

    async fn edit(
        &self,
        scope: &AccessScope,
        id: &Urn,
        cmd: &EditAgreementDto,
    ) -> Outcome<AgreementView>;

    async fn delete(&self, scope: &AccessScope, id: &Urn) -> Outcome<()>;
}
