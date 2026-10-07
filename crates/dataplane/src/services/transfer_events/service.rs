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

//! Event service over the repository.

use std::sync::Arc;

use common::oauth::{Owner, OwnerScope, UserInfo};
use common::batch_requests::BatchRequests;
use common::errors::NotFoundExt;
use common::paginated_spec::Cursor;
use common::query::{Page, Paginated, QueryFilter, Sort, MAX_BATCH_IDS};
use urn::Urn;
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::data::factory_trait::DataplaneRepoTrait;
use crate::data::repo::transfer_event::TransferEventRepo;
use crate::data::sea_orm::orm::transfer_event::NewTransferEvent;
use crate::entities::filters::TransferEventFilter;
use crate::entities::transfer_events::{NewTransferEventDto, TransferEventDto};
use crate::services::transfer_events::TransferEventServiceTrait;

pub struct TransferEventsService {
    data_plane_repo: Arc<dyn DataplaneRepoTrait>,
}

impl TransferEventsService {
    pub fn new(data_plane_repo: Arc<dyn DataplaneRepoTrait>) -> Self {
        Self { data_plane_repo }
    }

    fn repo(&self) -> Arc<dyn TransferEventRepo> {
        self.data_plane_repo.get_transfer_events_repo()
    }
}

#[async_trait::async_trait]
impl TransferEventServiceTrait for TransferEventsService {
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_all(
        &self,
        user: &UserInfo,
        filters: &TransferEventFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<TransferEventDto>> {
        filters.validate()?;

        let scope = OwnerScope::seeing(user);
        let page = page.clamped();

        let repo = self.repo();
        let (events, total) = tokio::try_join!(
            repo.get_all_transfer_events(&scope, filters, &page, sort),
            repo.count_transfer_events(&scope, filters),
        )?;

        let items: Vec<TransferEventDto> = events
            .into_iter()
            .map(|e| TransferEventDto { inner: e })
            .collect();

        Ok(Paginated::from_page(items, &page, Some(total), |e| {
            Cursor::encode_timestamp(&e.inner.created_at)
        }))
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_one(&self, user: &UserInfo, id: &Urn) -> Outcome<TransferEventDto> {

        let event = self
            .repo()
            .get_transfer_event_by_id(&OwnerScope::seeing(user), id)
            .await?
            .or_not_found(id, "transfer event")?;

        Ok(TransferEventDto { inner: event })
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_by_process_id(
        &self,
        user: &UserInfo,
        process_id: &Urn,
    ) -> Outcome<Vec<TransferEventDto>> {

        let events = self
            .repo()
            .get_all_transfer_events_by_process_id(
                &OwnerScope::seeing(user),
                process_id,
            )
            .await?;

        Ok(events
            .into_iter()
            .map(|e| TransferEventDto { inner: e })
            .collect())
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn batch(
        &self,
        user: &UserInfo,
        req: &BatchRequests,
    ) -> Outcome<Vec<TransferEventDto>> {

        if req.ids.len() > MAX_BATCH_IDS {
            return Err(Errors::format(
                BadFormat::Received,
                format!("batch request exceeds maximum of {MAX_BATCH_IDS} ids"),
                None,
            ));
        }

        if req.ids.is_empty() {
            return Ok(vec![]);
        }

        let events = self
            .repo()
            .get_batch_transfer_events(&OwnerScope::seeing(user), &req.ids)
            .await?;

        Ok(events
            .into_iter()
            .map(|e| TransferEventDto { inner: e })
            .collect())
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn create(
        &self,
        user: &UserInfo,
        cmd: &NewTransferEventDto,
    ) -> Outcome<TransferEventDto> {
        let owner = Owner::requested_by(user, cmd.owner.clone());
        let new_model: NewTransferEvent = cmd.clone().into_model(owner);
        let event = self.repo().create_transfer_event(&new_model).await?;

        Ok(TransferEventDto { inner: event })
    }
}
