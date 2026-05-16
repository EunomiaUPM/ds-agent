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

use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect,
};
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

use crate::data::repo::transfer_message::{TransferMessageRepoErrors, TransferMessageRepoTrait};
use crate::data::sea_orm::orm::transfer_message as orm;
use crate::entities::commands::NewTransferMessageCommand;
use crate::entities::query::{Page, Sort, TransferMessageFilter};
use crate::entities::transfer_message::TransferMessage;

pub(crate) struct SeaOrmTransferMessageRepo {
    db: Arc<DatabaseConnection>,
}

impl SeaOrmTransferMessageRepo {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn db_err(e: sea_orm::DbErr) -> ymir::errors::Errors {
        TransferMessageRepoErrors::ErrorFetchingTransferMessage(Box::new(e)).into_errors()
    }

    fn apply_message_filters(
        &self,
        mut q: sea_orm::Select<orm::Entity>,
        filters: &TransferMessageFilter,
        page: &Page,
        sort: &Sort,
    ) -> sea_orm::Select<orm::Entity> {
        use serde::Serialize;

        if let Some(dir) = &filters.direction {
            let s = serde_json::to_value(dir)
                .unwrap()
                .as_str()
                .unwrap_or("")
                .to_string();
            q = q.filter(orm::Column::Direction.eq(s));
        }
        if let Some(protocol) = &filters.protocol {
            let s = serde_json::to_value(protocol)
                .unwrap()
                .as_str()
                .unwrap_or("")
                .to_string();
            q = q.filter(orm::Column::Protocol.eq(s));
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
            if let Ok(bytes) =
                base64::Engine::decode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, cursor)
            {
                if let Ok(s) = String::from_utf8(bytes) {
                    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&s) {
                        q = match sort {
                            Sort::CreatedAtAsc => q.filter(orm::Column::OccurredAt.gt(dt)),
                            _ => q.filter(orm::Column::OccurredAt.lt(dt)),
                        };
                    }
                }
            }
        }

        match sort {
            Sort::CreatedAtAsc => q.order_by_asc(orm::Column::OccurredAt),
            _ => q.order_by_desc(orm::Column::OccurredAt),
        }
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
        q = q.filter(orm::Column::TenantId.eq(filters.tenant_id.as_str()));
        q = self.apply_message_filters(q, filters, page, sort);
        q.limit(page.limit as u64)
            .all(self.db.as_ref())
            .await
            .map_err(Self::db_err)?
            .into_iter()
            .map(orm::Model::into_domain)
            .collect()
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
        q = self.apply_message_filters(q, filters, page, sort);
        q.limit(page.limit as u64)
            .all(self.db.as_ref())
            .await
            .map_err(Self::db_err)?
            .into_iter()
            .map(orm::Model::into_domain)
            .collect()
    }

    async fn get_transfer_message_by_id(&self, id: &Urn) -> Outcome<Option<TransferMessage>> {
        orm::Entity::find_by_id(id.to_string())
            .one(self.db.as_ref())
            .await
            .map_err(Self::db_err)?
            .map(orm::Model::into_domain)
            .transpose()
    }

    async fn create_transfer_message(
        &self,
        cmd: &NewTransferMessageCommand,
    ) -> Outcome<TransferMessage> {
        orm::ActiveModel::from_domain(&TransferMessage::from_cmd(cmd))
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
