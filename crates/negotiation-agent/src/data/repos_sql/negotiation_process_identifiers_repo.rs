/*
 *
 * * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

use crate::data::entities::negotiation_process_identifier;
use crate::data::entities::negotiation_process_identifier::{
    EditNegotiationIdentifierModel, Model, NewNegotiationIdentifierModel,
};
use crate::data::repo_traits::negotiation_process_identifiers_repo::{
    NegotiationIdentifierRepoErrors, NegotiationIdentifierRepoTrait,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QuerySelect,
};
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

pub struct NegotiationProcessIdentifierRepoForSql {
    db_connection: DatabaseConnection,
}

impl NegotiationProcessIdentifierRepoForSql {
    pub fn new(db_connection: DatabaseConnection) -> Self {
        Self { db_connection }
    }
}

#[async_trait::async_trait]
impl NegotiationIdentifierRepoTrait for NegotiationProcessIdentifierRepoForSql {
    async fn get_all_identifiers(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Outcome<Vec<Model>> {
        let identifiers = negotiation_process_identifier::Entity::find()
            .limit(limit.unwrap_or(20))
            .offset(page.map(|p| p * limit.unwrap_or(20)).unwrap_or(0))
            .all(&self.db_connection)
            .await;

        match identifiers {
            Ok(identifiers) => Ok(identifiers),
            Err(e) => Err(
                NegotiationIdentifierRepoErrors::ErrorFetchingNegotiationIdentifier(e.into())
                    .into_errors(),
            ),
        }
    }

    async fn get_identifiers_by_process_id(&self, process_id: &Urn) -> Outcome<Vec<Model>> {
        let pid = process_id.to_string();
        let identifiers = negotiation_process_identifier::Entity::find()
            .filter(negotiation_process_identifier::Column::NegotiationAgentProcessId.eq(pid))
            .all(&self.db_connection)
            .await;

        match identifiers {
            Ok(identifiers) => Ok(identifiers),
            Err(e) => Err(
                NegotiationIdentifierRepoErrors::ErrorFetchingNegotiationIdentifier(e.into())
                    .into_errors(),
            ),
        }
    }

    async fn get_identifier_by_id(&self, id: &Urn) -> Outcome<Option<Model>> {
        let iid = id.to_string();
        let identifier = negotiation_process_identifier::Entity::find_by_id(iid)
            .one(&self.db_connection)
            .await;

        match identifier {
            Ok(identifier) => Ok(identifier),
            Err(e) => Err(
                NegotiationIdentifierRepoErrors::ErrorFetchingNegotiationIdentifier(e.into())
                    .into_errors(),
            ),
        }
    }

    async fn get_identifier_by_key(&self, process_id: &Urn, key: &str) -> Outcome<Option<Model>> {
        let pid = process_id.to_string();
        let identifier = negotiation_process_identifier::Entity::find()
            .filter(negotiation_process_identifier::Column::NegotiationAgentProcessId.eq(pid))
            .filter(negotiation_process_identifier::Column::IdKey.eq(key))
            .one(&self.db_connection)
            .await;

        match identifier {
            Ok(identifier) => Ok(identifier),
            Err(e) => Err(
                NegotiationIdentifierRepoErrors::ErrorFetchingNegotiationIdentifier(e.into())
                    .into_errors(),
            ),
        }
    }

    async fn create_identifier(&self, new_model: &NewNegotiationIdentifierModel) -> Outcome<Model> {
        let model: negotiation_process_identifier::ActiveModel = new_model.clone().into();
        let result = negotiation_process_identifier::Entity::insert(model)
            .exec_with_returning(&self.db_connection)
            .await;
        match result {
            Ok(identifier) => Ok(identifier),
            Err(e) => Err(
                NegotiationIdentifierRepoErrors::ErrorCreatingNegotiationIdentifier(e.into())
                    .into_errors(),
            ),
        }
    }

    async fn put_identifier(
        &self,
        id: &Urn,
        edit_model: &EditNegotiationIdentifierModel,
    ) -> Outcome<Model> {
        let iid = id.to_string();
        let old_model = negotiation_process_identifier::Entity::find_by_id(&iid)
            .one(&self.db_connection)
            .await;
        let old_model = match old_model {
            Ok(Some(model)) => model,
            Ok(None) => {
                return Err(
                    NegotiationIdentifierRepoErrors::NegotiationIdentifierNotFound.into_errors(),
                );
            }
            Err(e) => {
                return Err(
                    NegotiationIdentifierRepoErrors::ErrorFetchingNegotiationIdentifier(e.into())
                        .into_errors(),
                );
            }
        };

        let mut active_model: negotiation_process_identifier::ActiveModel = old_model.into();
        if let Some(key) = &edit_model.id_key {
            active_model.id_key = ActiveValue::Set(key.clone());
        }
        if let Some(value) = &edit_model.id_value {
            active_model.id_value = ActiveValue::Set(Some(value.clone()));
        }

        let result = active_model.update(&self.db_connection).await;
        match result {
            Ok(updated_model) => Ok(updated_model),
            Err(e) => Err(
                NegotiationIdentifierRepoErrors::ErrorUpdatingNegotiationIdentifier(e.into())
                    .into_errors(),
            ),
        }
    }

    async fn delete_identifier(&self, id: &Urn) -> Outcome<()> {
        let iid = id.to_string();
        let result = negotiation_process_identifier::Entity::delete_by_id(iid)
            .exec(&self.db_connection)
            .await;

        match result {
            Ok(delete_result) => match delete_result.rows_affected {
                0 => Err(
                    NegotiationIdentifierRepoErrors::NegotiationIdentifierNotFound.into_errors(),
                ),
                _ => Ok(()),
            },
            Err(e) => Err(
                NegotiationIdentifierRepoErrors::ErrorDeletingNegotiationIdentifier(e.into())
                    .into_errors(),
            ),
        }
    }
}
