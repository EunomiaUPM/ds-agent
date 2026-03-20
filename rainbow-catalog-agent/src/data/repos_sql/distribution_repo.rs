use crate::data::entities::distribution::{EditDistributionModel, NewDistributionModel};
use crate::data::entities::{dataservice, dataset, distribution};
use crate::data::repo_traits::catalog_db_errors::{
    CatalogAgentRepoErrors, DataServiceRepoErrors, DatasetRepoErrors, DistributionRepoErrors,
};
use crate::data::repo_traits::distribution_repo::DistributionRepositoryTrait;
use rainbow_common::dcat_formats::DctFormats;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QuerySelect,
};
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

pub struct DistributionRepositoryForSql {
    db_connection: DatabaseConnection,
}

impl DistributionRepositoryForSql {
    pub fn new(db_connection: DatabaseConnection) -> Self {
        Self { db_connection }
    }
}

#[async_trait::async_trait]
impl DistributionRepositoryTrait for DistributionRepositoryForSql {
    async fn get_all_distributions(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Outcome<Vec<distribution::Model>> {
        let page_limit = limit.unwrap_or(25);
        let page_number = page.unwrap_or(1);
        let calculated_offset = (page_number.max(1) - 1) * page_limit;
        let distributions = distribution::Entity::find()
            .limit(page_limit)
            .offset(calculated_offset)
            .all(&self.db_connection)
            .await;
        match distributions {
            Ok(distributions) => Ok(distributions),
            Err(err) => Err(CatalogAgentRepoErrors::DistributionRepoErrors(
                DistributionRepoErrors::ErrorFetchingDistribution(err.into()),
            ).into_errors()),
        }
    }

    async fn get_batch_distributions(
        &self,
        ids: &Vec<Urn>,
    ) -> Outcome<Vec<distribution::Model>> {
        let distribution_ids = ids.iter().map(|t| t.to_string()).collect::<Vec<_>>();
        let distribution_process = distribution::Entity::find()
            .filter(distribution::Column::Id.is_in(distribution_ids))
            .all(&self.db_connection)
            .await;
        match distribution_process {
            Ok(dataset_process) => Ok(dataset_process),
            Err(err) => Err(CatalogAgentRepoErrors::DistributionRepoErrors(
                DistributionRepoErrors::ErrorFetchingDistribution(err.into()),
            ).into_errors()),
        }
    }

    async fn get_distributions_by_dataset_id(
        &self,
        dataset_id: &Urn,
    ) -> Outcome<Vec<distribution::Model>> {
        let dataset_id = dataset_id.to_string();
        let dataset = dataset::Entity::find_by_id(dataset_id).one(&self.db_connection).await;
        match dataset {
            Ok(dataset) => match dataset {
                Some(dataset) => {
                    let distributions = distribution::Entity::find()
                        .filter(distribution::Column::DatasetId.eq(dataset.id))
                        .all(&self.db_connection)
                        .await;
                    match distributions {
                        Ok(distributions) => Ok(distributions),
                        Err(err) => Err(CatalogAgentRepoErrors::DistributionRepoErrors(
                            DistributionRepoErrors::ErrorFetchingDistribution(err.into()),
                        ).into_errors()),
                    }
                }
                None => Err(CatalogAgentRepoErrors::DatasetRepoErrors(
                    DatasetRepoErrors::DatasetNotFound,
                ).into_errors()),
            },
            Err(err) => Err(CatalogAgentRepoErrors::DistributionRepoErrors(
                DistributionRepoErrors::ErrorFetchingDistribution(err.into()),
            ).into_errors()),
        }
    }

    async fn get_distribution_by_dataset_id_and_dct_format(
        &self,
        dataset_id: &Urn,
        dct_formats: &String,
    ) -> Outcome<distribution::Model> {
        let dataset_id = dataset_id.to_string();
        let _ = dataset::Entity::find_by_id(dataset_id.clone())
            .one(&self.db_connection)
            .await
            .map_err(|err| {
                CatalogAgentRepoErrors::DistributionRepoErrors(
                    DistributionRepoErrors::ErrorFetchingDistribution(err.into()),
                ).into_errors()
            })?
            .ok_or(CatalogAgentRepoErrors::DatasetRepoErrors(
                DatasetRepoErrors::DatasetNotFound,
            ).into_errors())?;
        let distribution = distribution::Entity::find()
            .filter(distribution::Column::DatasetId.eq(dataset_id.clone()))
            .filter(distribution::Column::DctFormat.eq(dct_formats.to_string()))
            .one(&self.db_connection)
            .await
            .map_err(|err| {
                CatalogAgentRepoErrors::DistributionRepoErrors(
                    DistributionRepoErrors::ErrorFetchingDistribution(err.into()),
                ).into_errors()
            })?
            .ok_or(CatalogAgentRepoErrors::DistributionRepoErrors(
                DistributionRepoErrors::DistributionNotFound,
            ).into_errors())?;
        Ok(distribution)
    }

    async fn get_distribution_by_id(
        &self,
        distribution_id: &Urn,
    ) -> Outcome<Option<distribution::Model>> {
        let distribution_id = distribution_id.to_string();
        let distribution =
            distribution::Entity::find_by_id(distribution_id).one(&self.db_connection).await;
        match distribution {
            Ok(distribution) => Ok(distribution),
            Err(err) => Err(CatalogAgentRepoErrors::DistributionRepoErrors(
                DistributionRepoErrors::ErrorFetchingDistribution(err.into()),
            ).into_errors()),
        }
    }

    async fn put_distribution_by_id(
        &self,
        distribution_id: &Urn,
        edit_distribution_model: &EditDistributionModel,
    ) -> Outcome<distribution::Model> {
        let distribution_id = distribution_id.to_string();

        if let Some(ds) = edit_distribution_model.dcat_access_service.clone() {
            let data_service = dataservice::Entity::find_by_id(ds)
                .one(&self.db_connection)
                .await
                .map_err(|e| {
                CatalogAgentRepoErrors::DistributionRepoErrors(
                    DistributionRepoErrors::ErrorFetchingDistribution(e.into()),
                ).into_errors()
            })?;
            if data_service.is_none() {
                return Err(CatalogAgentRepoErrors::DistributionRepoErrors(
                    DistributionRepoErrors::DistributionNotFound,
                ).into_errors());
            }
        }

        let old_model =
            distribution::Entity::find_by_id(distribution_id).one(&self.db_connection).await;
        let old_model = match old_model {
            Ok(old_model) => match old_model {
                Some(old_model) => old_model,
                None => {
                    return Err(CatalogAgentRepoErrors::DistributionRepoErrors(
                        DistributionRepoErrors::DistributionNotFound,
                    ).into_errors())
                }
            },
            Err(err) => {
                return Err(CatalogAgentRepoErrors::DistributionRepoErrors(
                    DistributionRepoErrors::ErrorFetchingDistribution(err.into()),
                ).into_errors());
            }
        };
        let mut old_active_model: distribution::ActiveModel = old_model.into();
        if let Some(dct_title) = &edit_distribution_model.dct_title {
            old_active_model.dct_title = ActiveValue::Set(Some(dct_title.clone()));
        }
        if let Some(dct_description) = &edit_distribution_model.dct_description {
            old_active_model.dct_description = ActiveValue::Set(Some(dct_description.clone()));
        }
        if let Some(dcat_access_service) = &edit_distribution_model.dcat_access_service {
            old_active_model.dcat_access_service = ActiveValue::Set(dcat_access_service.clone());
        }
        old_active_model.dct_modified = ActiveValue::Set(Some(chrono::Utc::now().into()));
        let model = old_active_model.update(&self.db_connection).await;
        match model {
            Ok(model) => Ok(model),
            Err(err) => Err(CatalogAgentRepoErrors::DistributionRepoErrors(
                DistributionRepoErrors::ErrorUpdatingDistribution(err.into()),
            ).into_errors()),
        }
    }

    async fn create_distribution(
        &self,
        new_distribution_model: &NewDistributionModel,
    ) -> Outcome<distribution::Model> {
        let dataset =
            dataset::Entity::find_by_id(new_distribution_model.dataset_id.clone().to_string())
                .one(&self.db_connection)
                .await
                .map_err(|err| {
                    CatalogAgentRepoErrors::DistributionRepoErrors(
                        DistributionRepoErrors::ErrorFetchingDistribution(err.into()),
                    ).into_errors()
                })?;
        if dataset.is_none() {
            return Err(CatalogAgentRepoErrors::DatasetRepoErrors(
                DatasetRepoErrors::DatasetNotFound,
            ).into_errors());
        }

        let data_service =
            dataservice::Entity::find_by_id(new_distribution_model.dcat_access_service.clone())
                .one(&self.db_connection)
                .await
                .map_err(|err| {
                    CatalogAgentRepoErrors::DistributionRepoErrors(
                        DistributionRepoErrors::ErrorFetchingDistribution(err.into()),
                    ).into_errors()
                })?;
        if data_service.is_none() {
            return Err(CatalogAgentRepoErrors::DataServiceRepoErrors(
                DataServiceRepoErrors::DataServiceNotFound,
            ).into_errors());
        }

        let model: distribution::ActiveModel = new_distribution_model.into();
        let distribution =
            distribution::Entity::insert(model).exec_with_returning(&self.db_connection).await;
        match distribution {
            Ok(distribution) => Ok(distribution),
            Err(err) => Err(CatalogAgentRepoErrors::DistributionRepoErrors(
                DistributionRepoErrors::ErrorCreatingDistribution(err.into()),
            ).into_errors()),
        }
    }

    async fn delete_distribution_by_id(
        &self,
        distribution_id: &Urn,
    ) -> Outcome<()> {
        let distribution_id = distribution_id.to_string();
        let distribution =
            distribution::Entity::delete_by_id(distribution_id).exec(&self.db_connection).await;
        match distribution {
            Ok(delete_result) => match delete_result.rows_affected {
                0 => Err(CatalogAgentRepoErrors::DistributionRepoErrors(
                    DistributionRepoErrors::DistributionNotFound,
                ).into_errors()),
                _ => Ok(()),
            },
            Err(err) => Err(CatalogAgentRepoErrors::DistributionRepoErrors(
                DistributionRepoErrors::ErrorDeletingDistribution(err.into()),
            ).into_errors()),
        }
    }
}
