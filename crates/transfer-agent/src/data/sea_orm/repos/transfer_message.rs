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

use crate::data::repo::transfer_message::{TransferMessageRepoErrors, TransferMessageRepoTrait};
use crate::data::sea_orm::orm::ser_enum;
use crate::data::sea_orm::orm::transfer_message as orm;
use crate::entities::commands::NewTransferMessageCommand;
use crate::entities::filters::TransferMessageFilter;
use crate::entities::transfer_message::TransferMessage;
use common::paginated_spec::Cursor;
use common::query::{Page, Sort};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect,
};
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

pub(crate) struct SeaOrmTransferMessageRepo {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmTransferMessageRepo {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn fetch_err(e: sea_orm::DbErr) -> ymir::errors::Errors {
        TransferMessageRepoErrors::ErrorFetchingTransferMessage(Box::new(e)).into_errors()
    }

    #[allow(clippy::result_large_err)]
    fn decode_cursor(&self, cursor: &str) -> Outcome<chrono::DateTime<chrono::FixedOffset>> {
        Cursor::decode_timestamp(cursor)
            .map_err(|_| TransferMessageRepoErrors::InvalidCursor.into_errors())
    }

    fn apply_base_filters(
        mut q: sea_orm::Select<orm::Entity>,
        filters: &TransferMessageFilter,
    ) -> sea_orm::Select<orm::Entity> {
        if let Some(tid) = &filters.tenant_id {
            q = q.filter(orm::Column::TenantId.eq(tid.as_str()));
        }
        if let Some(dir) = &filters.direction {
            q = q.filter(orm::Column::Direction.eq(ser_enum(dir)));
        }
        if let Some(protocol) = &filters.protocol {
            q = q.filter(orm::Column::Protocol.eq(ser_enum(protocol)));
        }
        if let Some(state) = &filters.state_transition_to {
            q = q.filter(orm::Column::StateTransitionTo.eq(state.0.as_str()));
        }
        if let Some(after) = filters.created_after {
            q = q.filter(orm::Column::OccurredAt.gt(after));
        }
        if let Some(before) = filters.created_before {
            q = q.filter(orm::Column::OccurredAt.lt(before));
        }
        q
    }

    #[allow(clippy::result_large_err)]
    fn apply_page_and_sort(
        &self,
        mut q: sea_orm::Select<orm::Entity>,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<sea_orm::Select<orm::Entity>> {
        if let Some(cursor) = &page.cursor {
            let dt = self.decode_cursor(cursor)?;
            q = match sort {
                Sort::CreatedAtAsc => q.filter(orm::Column::OccurredAt.gt(dt)),
                _ => q.filter(orm::Column::OccurredAt.lt(dt)),
            };
        }
        Ok(match sort {
            Sort::CreatedAtAsc => q
                .order_by_asc(orm::Column::OccurredAt)
                .order_by_asc(orm::Column::Id),
            _ => q
                .order_by_desc(orm::Column::OccurredAt)
                .order_by_desc(orm::Column::Id),
        })
    }
}

#[async_trait::async_trait]
impl TransferMessageRepoTrait for SeaOrmTransferMessageRepo {
    async fn get_all_transfer_messages(
        &self,
        filters: &TransferMessageFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Vec<TransferMessage>> {
        let q = Self::apply_base_filters(orm::Entity::find(), filters);
        let q = self.apply_page_and_sort(q, page, sort)?;
        q.limit(page.limit as u64)
            .all(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)?
            .into_iter()
            .map(orm::Model::into_domain)
            .collect()
    }

    async fn count_transfer_messages(&self, filters: &TransferMessageFilter) -> Outcome<u64> {
        Self::apply_base_filters(orm::Entity::find(), filters)
            .count(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)
    }

    async fn get_messages_by_process_id(
        &self,
        process_id: &Urn,
        filters: &TransferMessageFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Vec<TransferMessage>> {
        let q =
            orm::Entity::find().filter(orm::Column::TransferProcessId.eq(process_id.to_string()));
        let q = Self::apply_base_filters(q, filters);
        let q = self.apply_page_and_sort(q, page, sort)?;
        q.limit(page.limit as u64)
            .all(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)?
            .into_iter()
            .map(orm::Model::into_domain)
            .collect()
    }

    async fn get_transfer_message_by_id(
        &self,
        tenant_id: Option<String>,
        id: &Urn,
    ) -> Outcome<Option<TransferMessage>> {
        let q = orm::Entity::find_by_id(id.to_string())
            .apply_if(tenant_id, |q, t| q.filter(orm::Column::TenantId.eq(t)));
        q.one(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)?
            .map(orm::Model::into_domain)
            .transpose()
    }

    async fn create_transfer_message(
        &self,
        cmd: &NewTransferMessageCommand,
    ) -> Outcome<TransferMessage> {
        orm::ActiveModel::from_domain(&TransferMessage::from_cmd(cmd)?)
            .insert(self.db.as_ref())
            .await
            .map_err(|e| {
                TransferMessageRepoErrors::ErrorCreatingTransferMessage(Box::new(e)).into_errors()
            })
            .and_then(orm::Model::into_domain)
    }

    async fn delete_transfer_message(&self, tenant_id: Option<String>, id: &Urn) -> Outcome<()> {
        let q = orm::Entity::delete_many()
            .filter(orm::Column::Id.eq(id.to_string()))
            .apply_if(tenant_id, |q, t| q.filter(orm::Column::TenantId.eq(t)));
        let res = q.exec(self.db.as_ref()).await.map_err(|e| {
            TransferMessageRepoErrors::ErrorDeletingTransferMessage(Box::new(e)).into_errors()
        })?;
        if res.rows_affected == 0 {
            return Err(TransferMessageRepoErrors::TransferMessageNotFound.into_errors());
        }
        Ok(())
    }
}
