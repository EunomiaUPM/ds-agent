use crate::data::entities::connector_instances;
use crate::data::entities::connector_instances::NewConnectorInstanceModel;
use crate::data::repo_traits::connector_instance_repo::ConnectorInstanceRepoTrait;
use crate::data::repo_traits::connector_repo_errors::{
    ConnectorAgentRepoErrors, ConnectorInstanceRepoErrors,
};
use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, RuntimeErr, SqlxError,
};
use ymir::errors::{Outcome, RepoIntoErrors};

pub struct ConnectorInstanceRepoForSql {
    db_connection: DatabaseConnection,
}

impl ConnectorInstanceRepoForSql {
    pub fn new(db_connection: DatabaseConnection) -> Self {
        Self { db_connection }
    }
}

#[async_trait::async_trait]
impl ConnectorInstanceRepoTrait for ConnectorInstanceRepoForSql {
    async fn create_instance(
        &self,
        new_instance_model: &NewConnectorInstanceModel,
    ) -> Outcome<connector_instances::Model> {
        let model: connector_instances::ActiveModel = new_instance_model.clone().into();
        let instance = connector_instances::Entity::insert(model)
            .exec_with_returning(&self.db_connection)
            .await;

        match instance {
            Ok(instance) => Ok(instance),
            Err(err) => match err {
                DbErr::Query(RuntimeErr::SqlxError(SqlxError::Database(ref db_err)))
                | DbErr::Exec(RuntimeErr::SqlxError(SqlxError::Database(ref db_err))) => {
                    if let Some(code) = db_err.code() {
                        if code == "23505"
                            && db_err.message().contains("idx_unique_distribution_connector")
                        {
                            Err(ConnectorAgentRepoErrors::ConnectorInstanceRepoErrors(
                                ConnectorInstanceRepoErrors::ErrorCreatingTemplateByDuplication(
                                    err.into(),
                                ),
                            )
                            .into_errors())
                        } else {
                            Err(ConnectorAgentRepoErrors::ConnectorInstanceRepoErrors(
                                ConnectorInstanceRepoErrors::ErrorCreatingInstance(err.into()),
                            )
                            .into_errors())
                        }
                    } else {
                        Err(ConnectorAgentRepoErrors::ConnectorInstanceRepoErrors(
                            ConnectorInstanceRepoErrors::ErrorCreatingInstance(err.into()),
                        )
                        .into_errors())
                    }
                }
                _ => Err(ConnectorAgentRepoErrors::ConnectorInstanceRepoErrors(
                    ConnectorInstanceRepoErrors::ErrorCreatingInstance(err.into()),
                )
                .into_errors()),
            },
        }
    }

    async fn get_instance_by_id(
        &self,
        instance_id: &String,
    ) -> Outcome<Option<connector_instances::Model>> {
        let result =
            connector_instances::Entity::find_by_id(instance_id).one(&self.db_connection).await;
        match result {
            Ok(opt) => Ok(opt),
            Err(err) => Err(ConnectorAgentRepoErrors::ConnectorInstanceRepoErrors(
                ConnectorInstanceRepoErrors::ErrorFetchingInstance(err.into()),
            )
            .into_errors()),
        }
    }

    async fn get_instance_by_name_and_version(
        &self,
        name: &String,
        version: &String,
    ) -> Outcome<Option<connector_instances::Model>> {
        let result = connector_instances::Entity::find()
            .filter(connector_instances::Column::TemplateName.eq(name))
            .filter(connector_instances::Column::TemplateVersion.eq(version))
            .one(&self.db_connection)
            .await;
        match result {
            Ok(opt) => Ok(opt),
            Err(err) => Err(ConnectorAgentRepoErrors::ConnectorInstanceRepoErrors(
                ConnectorInstanceRepoErrors::ErrorFetchingInstance(err.into()),
            )
            .into_errors()),
        }
    }

    async fn get_instances_by_distribution(
        &self,
        distribution_id: &String,
    ) -> Outcome<Option<connector_instances::Model>> {
        let result = connector_instances::Entity::find()
            .filter(connector_instances::Column::DistributionId.eq(distribution_id))
            .one(&self.db_connection)
            .await;
        match result {
            Ok(list) => Ok(list),
            Err(err) => Err(ConnectorAgentRepoErrors::ConnectorInstanceRepoErrors(
                ConnectorInstanceRepoErrors::ErrorFetchingInstance(err.into()),
            )
            .into_errors()),
        }
    }

    async fn delete_instance_by_name_and_version(
        &self,
        name: &String,
        version: &String,
    ) -> Outcome<()> {
        let instance = self.get_instance_by_name_and_version(name, &version).await?;
        if instance.is_none() {
            return Err(ConnectorAgentRepoErrors::ConnectorInstanceRepoErrors(
                ConnectorInstanceRepoErrors::InstanceNotFound,
            )
            .into_errors());
        }
        let instance = instance.unwrap();
        let result =
            connector_instances::Entity::delete_by_id(instance.id).exec(&self.db_connection).await;

        match result {
            Ok(delete_result) => match delete_result.rows_affected {
                0 => Err(ConnectorAgentRepoErrors::ConnectorInstanceRepoErrors(
                    ConnectorInstanceRepoErrors::InstanceNotFound,
                )
                .into_errors()),
                _ => Ok(()),
            },
            Err(err) => Err(ConnectorAgentRepoErrors::ConnectorInstanceRepoErrors(
                ConnectorInstanceRepoErrors::ErrorDeletingInstance(err.into()),
            )
            .into_errors()),
        }
    }

    async fn delete_instance_by_id(&self, instance_id: &String) -> Outcome<()> {
        let result =
            connector_instances::Entity::delete_by_id(instance_id).exec(&self.db_connection).await;

        match result {
            Ok(delete_result) => match delete_result.rows_affected {
                0 => Err(ConnectorAgentRepoErrors::ConnectorInstanceRepoErrors(
                    ConnectorInstanceRepoErrors::InstanceNotFound,
                )
                .into_errors()),
                _ => Ok(()),
            },
            Err(err) => Err(ConnectorAgentRepoErrors::ConnectorInstanceRepoErrors(
                ConnectorInstanceRepoErrors::ErrorDeletingInstance(err.into()),
            )
            .into_errors()),
        }
    }
}
