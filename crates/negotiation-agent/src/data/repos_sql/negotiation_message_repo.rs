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

use crate::data::entities::negotiation_message;
use crate::data::entities::negotiation_message::{Model, NewNegotiationMessageModel};
use crate::data::repo_traits::negotiation_message_repo::{
    NegotiationMessageRepoErrors, NegotiationMessageRepoTrait,
};
use crate::entities::filters::NegotiationMessageFilter;
use common::paginated_spec::{Page, SelectCursorExt, Sort};
use common::query::FilterApplier;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Select,
};
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

impl FilterApplier<Select<negotiation_message::Entity>> for NegotiationMessageFilter {
    fn apply_to(&self, mut q: Select<negotiation_message::Entity>) -> Select<negotiation_message::Entity> {
        if let Some(ref process_id) = self.process_id {
            q = q.filter(negotiation_message::Column::NegotiationAgentProcessId.eq(process_id));
        }
        if let Some(ref protocol) = self.protocol {
            q = q.filter(negotiation_message::Column::Protocol.eq(protocol));
        }
        if let Some(ref message_type) = self.message_type {
            q = q.filter(negotiation_message::Column::MessageType.eq(message_type));
        }
        if let Some(ref direction) = self.direction {
            q = q.filter(negotiation_message::Column::Direction.eq(direction));
        }
        if let Some(after) = self.created_after {
            q = q.filter(negotiation_message::Column::CreatedAt.gte(after));
        }
        if let Some(before) = self.created_before {
            q = q.filter(negotiation_message::Column::CreatedAt.lte(before));
        }
        q
    }
}

pub struct NegotiationMessageRepoForSql {
    db_connection: DatabaseConnection,
}

impl NegotiationMessageRepoForSql {
    pub fn new(db_connection: DatabaseConnection) -> Self {
        Self { db_connection }
    }
}

#[async_trait::async_trait]
impl NegotiationMessageRepoTrait for NegotiationMessageRepoForSql {
    async fn get_all_negotiation_messages(
        &self,
        filters: &NegotiationMessageFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<(Vec<Model>, Option<u64>)> {
        let mut q = negotiation_message::Entity::find();
        q = filters.apply_to(q);

        let total = q
            .clone()
            .count(&self.db_connection)
            .await
            .map_err(|e| {
                NegotiationMessageRepoErrors::ErrorFetchingNegotiationMessage(e.into())
                    .into_errors()
            })?;

        let items = q
            .apply_cursor_pagination_with_tie_break(
                page,
                sort,
                negotiation_message::Column::CreatedAt,
                negotiation_message::Column::Id,
            )
            .all(&self.db_connection)
            .await
            .map_err(|e| {
                NegotiationMessageRepoErrors::ErrorFetchingNegotiationMessage(e.into())
                    .into_errors()
            })?;

        Ok((items, Some(total)))
    }

    async fn get_messages_by_process_id(&self, process_id: &Urn) -> Outcome<Vec<Model>> {
        let pid = process_id.to_string();
        let messages = negotiation_message::Entity::find()
            .filter(negotiation_message::Column::NegotiationAgentProcessId.eq(pid))
            .order_by_asc(negotiation_message::Column::CreatedAt)
            .all(&self.db_connection)
            .await;

        match messages {
            Ok(messages) => Ok(messages),
            Err(e) => Err(
                NegotiationMessageRepoErrors::ErrorFetchingNegotiationMessage(e.into())
                    .into_errors(),
            ),
        }
    }

    async fn get_negotiation_message_by_id(&self, id: &Urn) -> Outcome<Option<Model>> {
        let mid = id.to_string();
        let message = negotiation_message::Entity::find_by_id(mid)
            .one(&self.db_connection)
            .await;
        match message {
            Ok(message) => Ok(message),
            Err(e) => Err(
                NegotiationMessageRepoErrors::ErrorFetchingNegotiationMessage(e.into())
                    .into_errors(),
            ),
        }
    }

    async fn create_negotiation_message(
        &self,
        new_model: &NewNegotiationMessageModel,
    ) -> Outcome<Model> {
        let model: negotiation_message::ActiveModel = new_model.clone().into();
        let result = negotiation_message::Entity::insert(model)
            .exec_with_returning(&self.db_connection)
            .await;
        match result {
            Ok(message) => Ok(message),
            Err(e) => Err(
                NegotiationMessageRepoErrors::ErrorCreatingNegotiationMessage(e.into())
                    .into_errors(),
            ),
        }
    }

    async fn delete_negotiation_message(&self, id: &Urn) -> Outcome<()> {
        let mid = id.to_string();
        let result = negotiation_message::Entity::delete_by_id(mid)
            .exec(&self.db_connection)
            .await;

        match result {
            Ok(delete_result) => match delete_result.rows_affected {
                0 => Err(NegotiationMessageRepoErrors::NegotiationMessageNotFound.into_errors()),
                _ => Ok(()),
            },
            Err(e) => Err(
                NegotiationMessageRepoErrors::ErrorDeletingNegotiationMessage(e.into())
                    .into_errors(),
            ),
        }
    }
}
