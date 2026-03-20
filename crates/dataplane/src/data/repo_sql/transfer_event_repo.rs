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

use crate::data::entities::transfer_event;
use crate::data::entities::transfer_event::NewTransferEvent;
use crate::data::repo_traits::transfer_event_repo::{TransferEventRepo, TransferEventRepoErrors};
use sea_orm::{
    ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect,
};
use urn::{Urn, UrnBuilder};
use uuid::Uuid;
use ymir::errors::{Outcome, RepoIntoErrors};

pub struct TransferEventRepoForSql {
    db_connection: DatabaseConnection,
}
impl TransferEventRepoForSql {
    pub fn new(db_connection: DatabaseConnection) -> Self {
        Self { db_connection }
    }
}

#[async_trait::async_trait]
impl TransferEventRepo for TransferEventRepoForSql {
    async fn get_all_transfer_events(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Outcome<Vec<transfer_event::Model>> {
        let events = transfer_event::Entity::find()
            .limit(limit.unwrap_or(100))
            .offset(page.map(|p| p * limit.unwrap_or(100)).unwrap_or(0))
            .all(&self.db_connection)
            .await;
        match events {
            Ok(events) => Ok(events),
            Err(e) => Err(TransferEventRepoErrors::ErrorFetchingTransferEvent(e.into()).into_errors()),
        }
    }

    async fn get_batch_transfer_events(
        &self,
        ids: &Vec<Urn>,
    ) -> Outcome<Vec<transfer_event::Model>> {
        let ids: Vec<String> = ids.iter().map(|urn| urn.to_string()).collect();

        let events = transfer_event::Entity::find()
            .filter(transfer_event::Column::Id.is_in(ids))
            .all(&self.db_connection)
            .await;

        match events {
            Ok(events) => Ok(events),
            Err(e) => Err(TransferEventRepoErrors::ErrorFetchingTransferEvent(e.into()).into_errors()),
        }
    }

    async fn get_all_transfer_events_by_process_id(
        &self,
        process_id: &Urn,
    ) -> Outcome<Vec<transfer_event::Model>> {
        let process_id = process_id.to_string();
        let events = transfer_event::Entity::find()
            .filter(transfer_event::Column::TransferId.eq(process_id))
            .all(&self.db_connection)
            .await;

        match events {
            Ok(events) => Ok(events),
            Err(e) => Err(TransferEventRepoErrors::ErrorFetchingTransferEvent(e.into()).into_errors()),
        }
    }

    async fn get_transfer_event_by_id(
        &self,
        transfer_event_urn: &Urn,
    ) -> Outcome<Option<transfer_event::Model>> {
        let transfer_event_urn = transfer_event_urn.to_string();
        let event =
            transfer_event::Entity::find_by_id(transfer_event_urn).one(&self.db_connection).await;

        match event {
            Ok(event) => Ok(event),
            Err(e) => Err(TransferEventRepoErrors::ErrorFetchingTransferEvent(e.into()).into_errors()),
        }
    }

    async fn create_transfer_event(
        &self,
        new_transfer_event: &NewTransferEvent,
    ) -> Outcome<transfer_event::Model> {
        let model: transfer_event::ActiveModel = new_transfer_event.clone().into();

        let event =
            transfer_event::Entity::insert(model).exec_with_returning(&self.db_connection).await;
        match event {
            Ok(event) => Ok(event),
            Err(e) => return Err(TransferEventRepoErrors::ErrorCreatingTransferEvent(e.into()).into_errors()),
        }
    }
}
