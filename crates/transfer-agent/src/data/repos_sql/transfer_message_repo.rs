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

use crate::data::entities::transfer_message;
use crate::data::entities::transfer_message::NewTransferMessageModel;
use crate::data::repo_traits::transfer_message_repo::{
    TransferMessageRepoErrors, TransferMessageRepoTrait,
};
use crate::entities::filters::TransferMessageFilter;
use common::paginated_spec::{Page, SelectCursorExt, Sort};
use common::query::FilterApplier;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Select,
};
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

impl FilterApplier<Select<transfer_message::Entity>> for TransferMessageFilter {
    fn apply_to(
        &self,
        mut select: Select<transfer_message::Entity>,
    ) -> Select<transfer_message::Entity> {
        if let Some(process_id) = &self.process_id {
            select = select.filter(transfer_message::Column::TransferAgentProcessId.eq(process_id));
        }
        if let Some(protocol) = &self.protocol {
            select = select.filter(transfer_message::Column::Protocol.eq(protocol));
        }
        if let Some(message_type) = &self.message_type {
            select = select.filter(transfer_message::Column::MessageType.eq(message_type));
        }
        if let Some(direction) = &self.direction {
            select = select.filter(transfer_message::Column::Direction.eq(direction));
        }
        if let Some(created_after) = self.created_after {
            select = select.filter(transfer_message::Column::CreatedAt.gte(created_after));
        }
        if let Some(created_before) = self.created_before {
            select = select.filter(transfer_message::Column::CreatedAt.lte(created_before));
        }
        select
    }
}

pub struct TransferMessageRepoForSql {
    db_connection: DatabaseConnection,
}

impl TransferMessageRepoForSql {
    pub fn new(db_connection: DatabaseConnection) -> Self {
        Self { db_connection }
    }
}

#[async_trait::async_trait]
impl TransferMessageRepoTrait for TransferMessageRepoForSql {
    async fn get_all_transfer_messages(
        &self,
        filters: &TransferMessageFilter,
        page: &Page,
        sort: Sort,
    ) -> Outcome<(Vec<transfer_message::Model>, Option<u64>)> {
        let q = filters.apply_to(transfer_message::Entity::find());
        let total = q.clone().count(&self.db_connection).await.map_err(|e| {
            TransferMessageRepoErrors::ErrorFetchingTransferMessage(e.into()).into_errors()
        })?;

        let items = q
            .apply_cursor_pagination_with_tie_break(
                page,
                sort,
                transfer_message::Column::CreatedAt,
                transfer_message::Column::Id,
            )
            .all(&self.db_connection)
            .await
            .map_err(|e| {
                TransferMessageRepoErrors::ErrorFetchingTransferMessage(e.into()).into_errors()
            })?;

        Ok((items, Some(total)))
    }

    async fn get_messages_by_process_id(
        &self,
        process_id: &Urn,
    ) -> Outcome<Vec<transfer_message::Model>> {
        let pid = process_id.to_string();
        let messages = transfer_message::Entity::find()
            .filter(transfer_message::Column::TransferAgentProcessId.eq(pid))
            .order_by_asc(transfer_message::Column::CreatedAt)
            .all(&self.db_connection)
            .await;

        match messages {
            Ok(messages) => Ok(messages),
            Err(e) => {
                Err(TransferMessageRepoErrors::ErrorFetchingTransferMessage(e.into()).into_errors())
            }
        }
    }

    async fn get_transfer_message_by_id(
        &self,
        id: &Urn,
    ) -> Outcome<Option<transfer_message::Model>> {
        let mid = id.to_string();
        let message = transfer_message::Entity::find_by_id(mid)
            .one(&self.db_connection)
            .await;
        match message {
            Ok(message) => Ok(message),
            Err(e) => {
                Err(TransferMessageRepoErrors::ErrorFetchingTransferMessage(e.into()).into_errors())
            }
        }
    }

    async fn create_transfer_message(
        &self,
        new_model: &NewTransferMessageModel,
    ) -> Outcome<transfer_message::Model> {
        let model: transfer_message::ActiveModel = new_model.clone().into();
        let result = transfer_message::Entity::insert(model)
            .exec_with_returning(&self.db_connection)
            .await;
        match result {
            Ok(message) => Ok(message),
            Err(e) => {
                Err(TransferMessageRepoErrors::ErrorCreatingTransferMessage(e.into()).into_errors())
            }
        }
    }

    async fn delete_transfer_message(&self, id: &Urn) -> Outcome<()> {
        let mid = id.to_string();
        let result = transfer_message::Entity::delete_by_id(mid)
            .exec(&self.db_connection)
            .await;

        match result {
            Ok(delete_result) => match delete_result.rows_affected {
                0 => Err(TransferMessageRepoErrors::TransferMessageNotFound.into_errors()),
                _ => Ok(()),
            },
            Err(e) => {
                Err(TransferMessageRepoErrors::ErrorDeletingTransferMessage(e.into()).into_errors())
            }
        }
    }
}
