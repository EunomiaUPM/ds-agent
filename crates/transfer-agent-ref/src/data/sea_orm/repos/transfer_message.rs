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

use std::sync::Arc;

use crate::data::repo::transfer_message::{TransferMessageRepoErrors, TransferMessageRepoTrait};
use crate::data::sea_orm::orm::ser_enum;
use crate::data::sea_orm::orm::transfer_message as orm;
use crate::entities::commands::NewTransferMessageCommand;
use crate::entities::filters::TransferMessageFilter;
use crate::entities::transfer_message::TransferMessage;
use base64::Engine;
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
        let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(cursor)
            .map_err(|_| TransferMessageRepoErrors::InvalidCursor.into_errors())?;
        let s = String::from_utf8(bytes)
            .map_err(|_| TransferMessageRepoErrors::InvalidCursor.into_errors())?;
        chrono::DateTime::parse_from_rfc3339(&s)
            .map_err(|_| TransferMessageRepoErrors::InvalidCursor.into_errors())
    }

    #[allow(clippy::result_large_err)]
    fn apply_message_filters(
        &self,
        mut q: sea_orm::Select<orm::Entity>,
        filters: &TransferMessageFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<sea_orm::Select<orm::Entity>> {
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
        let mut q = orm::Entity::find();
        if let Some(tid) = &filters.tenant_id {
            q = q.filter(orm::Column::TenantId.eq(tid.as_str()));
        }
        q = self.apply_message_filters(q, filters, page, sort)?;
        q.limit(page.limit as u64)
            .all(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)?
            .into_iter()
            .map(orm::Model::into_domain)
            .collect()
    }

    async fn count_transfer_messages(&self, filters: &TransferMessageFilter) -> Outcome<u64> {
        let mut q = orm::Entity::find();
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
        q.count(self.db.as_ref()).await.map_err(Self::fetch_err)
    }

    async fn get_messages_by_process_id(
        &self,
        process_id: &Urn,
        filters: &TransferMessageFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Vec<TransferMessage>> {
        let mut q = orm::Entity::find();
        q = q.filter(orm::Column::TransferProcessId.eq(process_id.to_string()));
        q = self.apply_message_filters(q, filters, page, sort)?;
        q.limit(page.limit as u64)
            .all(self.db.as_ref())
            .await
            .map_err(Self::fetch_err)?
            .into_iter()
            .map(orm::Model::into_domain)
            .collect()
    }

    async fn get_transfer_message_by_id(&self, id: &Urn) -> Outcome<Option<TransferMessage>> {
        orm::Entity::find_by_id(id.to_string())
            .one(self.db.as_ref())
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

    async fn delete_transfer_message(&self, id: &Urn) -> Outcome<()> {
        orm::Entity::delete_by_id(id.to_string())
            .exec(self.db.as_ref())
            .await
            .map_err(|e| {
                TransferMessageRepoErrors::ErrorDeletingTransferMessage(Box::new(e)).into_errors()
            })?;
        Ok(())
    }
}
