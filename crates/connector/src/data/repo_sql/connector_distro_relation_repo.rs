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
        tenant_id: &str,
        distro: &str,
        instance: &str,
    ) -> Outcome<connector_distro_relation::Model> {
        let relation = connector_distro_relation::ActiveModel {
            distribution_id: ActiveValue::Set(distro.to_string()),
            tenant_id: ActiveValue::Set(tenant_id.to_string()),
            connector_instance_id: ActiveValue::Set(instance.to_string()),
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
        tenant_id: &str,
        distro: &str,
        instance: &str,
    ) -> Outcome<connector_distro_relation::Model> {
        let existing = connector_distro_relation::Entity::find_by_id(distro)
            .filter(connector_distro_relation::Column::TenantId.eq(tenant_id))
            .one(&self.db_connection)
            .await
            .map_err(|e| {
                ConnectorAgentRepoErrors::ConnectorDistroRelationRepoErrors(
                    ConnectorDistroRelationRepoErrors::ErrorUpdatingRelation(e.to_string()),
                )
                .into_errors()
            })?
            .ok_or_else(|| {
                ConnectorAgentRepoErrors::ConnectorDistroRelationRepoErrors(
                    ConnectorDistroRelationRepoErrors::RelationNotFound,
                )
                .into_errors()
            })?;

        let mut active: connector_distro_relation::ActiveModel = existing.into();
        active.connector_instance_id = ActiveValue::Set(instance.to_string());
        active.update(&self.db_connection).await.map_err(|err| {
            ConnectorAgentRepoErrors::ConnectorDistroRelationRepoErrors(
                ConnectorDistroRelationRepoErrors::ErrorUpdatingRelation(err.to_string()),
            )
            .into_errors()
        })
    }

    async fn get_relation_by_distribution(
        &self,
        tenant_id: &str,
        distro: &str,
    ) -> Outcome<Option<connector_distro_relation::Model>> {
        let relation = connector_distro_relation::Entity::find()
            .filter(connector_distro_relation::Column::DistributionId.eq(distro))
            .filter(connector_distro_relation::Column::TenantId.eq(tenant_id))
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
        tenant_id: &str,
        instance: &str,
    ) -> Outcome<Option<connector_distro_relation::Model>> {
        let relation = connector_distro_relation::Entity::find()
            .filter(connector_distro_relation::Column::ConnectorInstanceId.eq(instance))
            .filter(connector_distro_relation::Column::TenantId.eq(tenant_id))
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

    async fn delete_relation_by_distribution(&self, tenant_id: &str, distro: &str) -> Outcome<()> {
        let result = connector_distro_relation::Entity::delete_many()
            .filter(connector_distro_relation::Column::DistributionId.eq(distro))
            .filter(connector_distro_relation::Column::TenantId.eq(tenant_id))
            .exec(&self.db_connection)
            .await;
        match result {
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

    async fn delete_relation_by_instance(&self, tenant_id: &str, instance: &str) -> Outcome<()> {
        let result = connector_distro_relation::Entity::delete_many()
            .filter(connector_distro_relation::Column::ConnectorInstanceId.eq(instance))
            .filter(connector_distro_relation::Column::TenantId.eq(tenant_id))
            .exec(&self.db_connection)
            .await;
        match result {
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
