/*
 *
 * * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 * *
 * * This program is free software: you can redistribute it and/or modify
 * * it under the terms of the GNU General Public License as published by
 * * the Free Software Foundation, either version 3 of the License, or
 * * (at your option) any later version.
 * *
 * * This program is distributed in the hope that it will be useful,
 * * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * * GNU General Public License for more details.
 * *
 * * You should have received a copy of the GNU General Public License
 * * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use crate::data::entities::negotiation_message;
use crate::data::entities::negotiation_message::{Model, NewNegotiationMessageModel};
use crate::data::repo_traits::negotiation_message_repo::{
    NegotiationMessageRepoErrors, NegotiationMessageRepoTrait,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

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
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Outcome<Vec<Model>> {
        let messages = negotiation_message::Entity::find()
            .limit(limit.unwrap_or(20))
            .offset(page.map(|p| p * limit.unwrap_or(20)).unwrap_or(0))
            .order_by_desc(negotiation_message::Column::CreatedAt)
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
