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

use crate::data::entities::connector_distro_relation;
use crate::data::repo_traits::connector_distro_relation_repo::ConnectorDistroRelationRepoTrait;
use crate::data::repo_traits::connector_repo_errors::{
    ConnectorAgentRepoErrors, ConnectorDistroRelationRepoErrors,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel,
    QueryFilter,
};
use ymir::errors::{Outcome, RepoIntoErrors};

pub struct ConnectorDistroRelationRepoForSql {
    db_connection: DatabaseConnection,
}

impl ConnectorDistroRelationRepoForSql {
    pub fn new(db_connection: DatabaseConnection) -> Self {
        Self { db_connection }
    }
}

#[async_trait::async_trait]
impl ConnectorDistroRelationRepoTrait for ConnectorDistroRelationRepoForSql {
    async fn create_relation(
        &self,
        distro: &String,
        instance: &String,
    ) -> Outcome<connector_distro_relation::Model> {
        let relation = connector_distro_relation::ActiveModel {
            distribution_id: ActiveValue::Set(distro.clone()),
            connector_instance_id: ActiveValue::Set(instance.clone()),
        };
        let instance = connector_distro_relation::Entity::insert(relation)
            .exec_with_returning(&self.db_connection)
            .await;
        match instance {
            Ok(instance) => Ok(instance),
            Err(err) => Err(ConnectorAgentRepoErrors::ConnectorDistroRelationRepoErrors(
                ConnectorDistroRelationRepoErrors::ErrorCreatingRelation(err.to_string()),
            )
            .into_errors()),
        }
    }

    async fn update_relation(
        &self,
        distro: &String,
        instance: &String,
    ) -> Outcome<connector_distro_relation::Model> {
        let relation = self.get_relation_by_distribution(distro).await?;
        if relation.is_none() {
            return Err(ConnectorAgentRepoErrors::ConnectorDistroRelationRepoErrors(
                ConnectorDistroRelationRepoErrors::RelationNotFound,
            )
            .into_errors());
        }
        let mut old_relation: connector_distro_relation::ActiveModel = relation.unwrap().into();
        old_relation.connector_instance_id = ActiveValue::Set(instance.clone());
        let model = old_relation.update(&self.db_connection).await;
        match model {
            Ok(model) => Ok(model),
            Err(err) => Err(ConnectorAgentRepoErrors::ConnectorDistroRelationRepoErrors(
                ConnectorDistroRelationRepoErrors::ErrorUpdatingRelation(err.to_string()),
            )
            .into_errors()),
        }
    }

    async fn get_relation_by_distribution(
        &self,
        distro: &String,
    ) -> Outcome<Option<connector_distro_relation::Model>> {
        let relation = connector_distro_relation::Entity::find()
            .filter(connector_distro_relation::Column::DistributionId.eq(distro))
            .one(&self.db_connection)
            .await;
        match relation {
            Ok(relation) => Ok(relation),
            Err(err) => Err(ConnectorAgentRepoErrors::ConnectorDistroRelationRepoErrors(
                ConnectorDistroRelationRepoErrors::ErrorFetchingRelation(err.to_string()),
            )
            .into_errors()),
        }
    }

    async fn get_relation_by_instance(
        &self,
        instance: &String,
    ) -> Outcome<Option<connector_distro_relation::Model>> {
        let relation = connector_distro_relation::Entity::find_by_id(instance)
            .one(&self.db_connection)
            .await;
        match relation {
            Ok(relation) => Ok(relation),
            Err(err) => Err(ConnectorAgentRepoErrors::ConnectorDistroRelationRepoErrors(
                ConnectorDistroRelationRepoErrors::ErrorFetchingRelation(err.to_string()),
            )
            .into_errors()),
        }
    }

    async fn delete_relation_by_distribution(&self, distro: &String) -> Outcome<()> {
        let relation = self.get_relation_by_distribution(distro).await?;
        if relation.is_none() {
            return Err(ConnectorAgentRepoErrors::ConnectorDistroRelationRepoErrors(
                ConnectorDistroRelationRepoErrors::RelationNotFound,
            )
            .into_errors());
        }
        let relation = relation.unwrap();

        let relation = connector_distro_relation::Entity::delete(relation.into_active_model())
            .exec(&self.db_connection)
            .await;
        match relation {
            Ok(delete_result) => match delete_result.rows_affected {
                0 => Err(ConnectorAgentRepoErrors::ConnectorDistroRelationRepoErrors(
                    ConnectorDistroRelationRepoErrors::RelationNotFound,
                )
                .into_errors()),
                _ => Ok(()),
            },
            Err(err) => Err(ConnectorAgentRepoErrors::ConnectorDistroRelationRepoErrors(
                ConnectorDistroRelationRepoErrors::ErrorDeletingRelation(err.to_string()),
            )
            .into_errors()),
        }
    }

    async fn delete_relation_by_instance(&self, distro: &String) -> Outcome<()> {
        let relation = connector_distro_relation::Entity::delete_by_id(distro)
            .exec(&self.db_connection)
            .await;
        match relation {
            Ok(delete_result) => match delete_result.rows_affected {
                0 => Err(ConnectorAgentRepoErrors::ConnectorDistroRelationRepoErrors(
                    ConnectorDistroRelationRepoErrors::RelationNotFound,
                )
                .into_errors()),
                _ => Ok(()),
            },
            Err(err) => Err(ConnectorAgentRepoErrors::ConnectorDistroRelationRepoErrors(
                ConnectorDistroRelationRepoErrors::ErrorDeletingRelation(err.to_string()),
            )
            .into_errors()),
        }
    }
}
