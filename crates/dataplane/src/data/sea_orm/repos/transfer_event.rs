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

use sea_orm::QueryTrait;
use std::sync::Arc;

use crate::data::repo::transfer_event::{TransferEventRepo, TransferEventRepoErrors};
use crate::data::sea_orm::orm::transfer_event::{
    self, Column, Entity as TransferEventEntity, NewTransferEvent,
};
use crate::entities::filters::TransferEventFilter;
use common::paginated_spec::Cursor;
use common::query::{Page, Sort};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect,
};
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

pub struct TransferEventRepoForSql {
    db: Arc<DatabaseConnection>,
}

impl TransferEventRepoForSql {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub fn new_with_raw_db(db: DatabaseConnection) -> Self {
        Self { db: Arc::new(db) }
    }

    fn fetch_err(e: sea_orm::DbErr) -> ymir::errors::Errors {
        TransferEventRepoErrors::ErrorFetchingTransferEvent(Box::new(e)).into_errors()
    }

    fn decode_cursor(&self, cursor: &str) -> Outcome<chrono::DateTime<chrono::FixedOffset>> {
        Cursor::decode_timestamp(cursor)
            .map_err(|_| TransferEventRepoErrors::InvalidCursor.into_errors())
    }

    fn apply_base_filters(
        mut q: sea_orm::Select<TransferEventEntity>,
        filters: &TransferEventFilter,
    ) -> sea_orm::Select<TransferEventEntity> {
        if let Some(tid) = &filters.tenant_id {
            q = q.filter(Column::TenantId.eq(tid.as_str()));
        }
        if let Some(trid) = &filters.transfer_id {
            q = q.filter(Column::TransferId.eq(trid.as_str()));
        }
        if let Some(level) = &filters.level {
            q = q.filter(Column::Level.eq(level.clone()));
        }
        if let Some(component) = &filters.component {
            q = q.filter(Column::Component.eq(component.as_str()));
        }
        if let Some(after) = filters.created_after {
            q = q.filter(Column::CreatedAt.gt(after));
        }
        if let Some(before) = filters.created_before {
            q = q.filter(Column::CreatedAt.lt(before));
        }
        q
    }
}

#[async_trait::async_trait]
impl TransferEventRepo for TransferEventRepoForSql {
    async fn get_all_transfer_events(
        &self,
        filters: &TransferEventFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Vec<transfer_event::Model>> {
        let mut q = Self::apply_base_filters(TransferEventEntity::find(), filters);

        if let Some(cursor) = &page.cursor {
            let cursor_dt = self.decode_cursor(cursor)?;
            q = match sort {
                Sort::CreatedAtAsc => q.filter(Column::CreatedAt.gt(cursor_dt)),
                _ => q.filter(Column::CreatedAt.lt(cursor_dt)),
            };
        }

        q = match sort {
            Sort::CreatedAtAsc => q.order_by_asc(Column::CreatedAt).order_by_asc(Column::Id),
            _ => q.order_by_desc(Column::CreatedAt).order_by_desc(Column::Id),
        };

        let events = q
            .limit(page.limit as u64)
            .all(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)?;

        Ok(events)
    }

    async fn count_transfer_events(&self, filters: &TransferEventFilter) -> Outcome<u64> {
        Self::apply_base_filters(TransferEventEntity::find(), filters)
            .count(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)
    }

    async fn get_batch_transfer_events(
        &self,
        tenant_id: Option<String>,
        ids: &[Urn],
    ) -> Outcome<Vec<transfer_event::Model>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let ids: Vec<String> = ids.iter().map(|urn| urn.to_string()).collect();
        let events = TransferEventEntity::find()
            .filter(Column::Id.is_in(ids))
            .apply_if(tenant_id, |q, t| q.filter(Column::TenantId.eq(t)))
            .all(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)?;

        Ok(events)
    }

    async fn get_all_transfer_events_by_process_id(
        &self,
        tenant_id: Option<String>,
        process_id: &Urn,
    ) -> Outcome<Vec<transfer_event::Model>> {
        let events = TransferEventEntity::find()
            .filter(Column::TransferId.eq(process_id.to_string()))
            .apply_if(tenant_id, |q, t| q.filter(Column::TenantId.eq(t)))
            .all(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)?;

        Ok(events)
    }

    async fn get_transfer_event_by_id(
        &self,
        tenant_id: Option<String>,
        transfer_event_urn: &Urn,
    ) -> Outcome<Option<transfer_event::Model>> {
        let event = TransferEventEntity::find_by_id(transfer_event_urn.to_string())
            .apply_if(tenant_id, |q, t| q.filter(Column::TenantId.eq(t)))
            .one(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)?;

        Ok(event)
    }

    async fn create_transfer_event(
        &self,
        new_transfer_event: &NewTransferEvent,
    ) -> Outcome<transfer_event::Model> {
        let model: transfer_event::ActiveModel = new_transfer_event.clone().into();
        let event = TransferEventEntity::insert(model)
            .exec_with_returning(self.db.as_ref())
            .await
            .map_err(|e| {
                TransferEventRepoErrors::ErrorCreatingTransferEvent(Box::new(e)).into_errors()
            })?;
        Ok(event)
    }
}
