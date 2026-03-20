/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

use crate::data::entities::transfer_event::NewTransferEvent;
use crate::data::factory_trait::DataplaneRepoTrait;
use crate::entities::transfer_events::{
    NewTransferEventDto, TransferEventDto, TransferEventEntitiesTrait,
};
use common::errors::{CommonErrors, ErrorLog};
use std::str::FromStr;
use std::sync::Arc;
use tracing::error;
use urn::Urn;
use ymir::errors::Outcome;

pub struct TransferEventEntityService {
    pub data_plane_repo: Arc<dyn DataplaneRepoTrait>,
}

impl TransferEventEntityService {
    pub fn new(data_plane_repo: &std::sync::Arc<dyn DataplaneRepoTrait>) -> Self {
        Self { data_plane_repo: data_plane_repo.clone() }
    }
}

#[async_trait::async_trait]
impl TransferEventEntitiesTrait for TransferEventEntityService {
    async fn get_all_transfer_events(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Outcome<Vec<TransferEventDto>> {
        let events = self
            .data_plane_repo
            .get_transfer_events_repo()
            .get_all_transfer_events(limit, page)
            .await?;

        Ok(events.into_iter().map(|e| TransferEventDto { inner: e }).collect())
    }

    async fn get_batch_transfer_events(
        &self,
        ids: Vec<Urn>,
    ) -> Outcome<Vec<TransferEventDto>> {
        let events = self
            .data_plane_repo
            .get_transfer_events_repo()
            .get_batch_transfer_events(&ids)
            .await?;

        Ok(events.into_iter().map(|e| TransferEventDto { inner: e }).collect())
    }

    async fn get_transfer_event_by_id(&self, id: &Urn) -> Outcome<Option<TransferEventDto>> {
        let event = self
            .data_plane_repo
            .get_transfer_events_repo()
            .get_transfer_event_by_id(id)
            .await?;

        Ok(event.map(|e| TransferEventDto { inner: e }))
    }

    async fn get_transfer_events_by_process_id(
        &self,
        process_id: &Urn,
    ) -> Outcome<Vec<TransferEventDto>> {
        let events = self
            .data_plane_repo
            .get_transfer_events_repo()
            .get_all_transfer_events_by_process_id(process_id)
            .await?;

        Ok(events.into_iter().map(|e| TransferEventDto { inner: e }).collect())
    }

    async fn create_transfer_event(
        &self,
        new_transfer_event: &NewTransferEventDto,
    ) -> Outcome<TransferEventDto> {
        let new_model: NewTransferEvent = new_transfer_event.clone().into();

        let created_event = self
            .data_plane_repo
            .get_transfer_events_repo()
            .create_transfer_event(&new_model)
            .await?;

        Ok(TransferEventDto { inner: created_event })
    }
}
