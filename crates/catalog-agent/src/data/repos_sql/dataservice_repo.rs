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

use crate::data::entities::dataservice::{EditDataServiceModel, NewDataServiceModel};
use crate::data::entities::{catalog, dataservice};
use crate::data::repo_traits::catalog_db_errors::{
    CatalogAgentRepoErrors, CatalogRepoErrors, DataServiceRepoErrors, DistributionRepoErrors,
};
use crate::data::repo_traits::dataservice_repo::DataServiceRepositoryTrait;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QuerySelect,
};
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

pub struct DataServiceRepositoryForSql {
    db_connection: DatabaseConnection,
}

impl DataServiceRepositoryForSql {
    pub fn new(db_connection: DatabaseConnection) -> Self {
        Self { db_connection }
    }
}

#[async_trait::async_trait]
impl DataServiceRepositoryTrait for DataServiceRepositoryForSql {
    async fn get_all_data_services(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Outcome<Vec<dataservice::Model>> {
        let page_limit = limit.unwrap_or(25);
        let page_number = page.unwrap_or(1);
        let calculated_offset = (page_number.max(1) - 1) * page_limit;
        let data_services = dataservice::Entity::find()
            .limit(page_limit)
            .offset(calculated_offset)
            .all(&self.db_connection)
            .await;
        match data_services {
            Ok(data_services) => Ok(data_services),
            Err(err) => Err(CatalogAgentRepoErrors::DataServiceRepoErrors(
                DataServiceRepoErrors::ErrorFetchingDataService(err.into()),
            )
            .into_errors()),
        }
    }

    async fn get_batch_data_services(&self, ids: &Vec<Urn>) -> Outcome<Vec<dataservice::Model>> {
        let dataset_ids = ids.iter().map(|t| t.to_string()).collect::<Vec<_>>();
        let dataset_process = dataservice::Entity::find()
            .filter(dataservice::Column::Id.is_in(dataset_ids))
            .all(&self.db_connection)
            .await;
        match dataset_process {
            Ok(dataset_process) => Ok(dataset_process),
            Err(err) => Err(CatalogAgentRepoErrors::DataServiceRepoErrors(
                DataServiceRepoErrors::ErrorFetchingDataService(err.into()),
            )
            .into_errors()),
        }
    }

    async fn get_data_services_by_catalog_id(
        &self,
        catalog_id: &Urn,
    ) -> Outcome<Vec<dataservice::Model>> {
        let catalog_id = catalog_id.to_string();

        let catalog = catalog::Entity::find_by_id(catalog_id.clone())
            .one(&self.db_connection)
            .await
            .map_err(|err| {
                CatalogAgentRepoErrors::DataServiceRepoErrors(
                    DataServiceRepoErrors::ErrorFetchingDataService(err.into()),
                )
                .into_errors()
            })?;
        if catalog.is_none() {
            return Err(CatalogAgentRepoErrors::CatalogRepoErrors(
                CatalogRepoErrors::CatalogNotFound,
            )
            .into_errors());
        }

        let data_services = dataservice::Entity::find()
            .filter(dataservice::Column::CatalogId.eq(catalog_id))
            .all(&self.db_connection)
            .await;
        match data_services {
            Ok(data_services) => Ok(data_services),
            Err(err) => Err(CatalogAgentRepoErrors::DataServiceRepoErrors(
                DataServiceRepoErrors::ErrorFetchingDataService(err.into()),
            )
            .into_errors()),
        }
    }

    async fn get_main_data_service(&self) -> Outcome<Option<dataservice::Model>> {
        let data_service = dataservice::Entity::find()
            .filter(dataservice::Column::DspaceMainDataService.eq(true))
            .one(&self.db_connection)
            .await
            .map_err(|err| {
                CatalogAgentRepoErrors::DataServiceRepoErrors(
                    DataServiceRepoErrors::ErrorFetchingDataService(err.into()),
                )
                .into_errors()
            })?;
        Ok(data_service)
    }

    async fn get_data_service_by_id(
        &self,
        data_service_id: &Urn,
    ) -> Outcome<Option<dataservice::Model>> {
        let data_service_id = data_service_id.to_string();
        let data_service = dataservice::Entity::find_by_id(data_service_id)
            .one(&self.db_connection)
            .await;
        match data_service {
            Ok(data_service) => Ok(data_service),
            Err(err) => Err(CatalogAgentRepoErrors::DataServiceRepoErrors(
                DataServiceRepoErrors::ErrorFetchingDataService(err.into()),
            )
            .into_errors()),
        }
    }

    async fn put_data_service_by_id(
        &self,
        data_service_id: &Urn,
        edit_data_service_model: &EditDataServiceModel,
    ) -> Outcome<dataservice::Model> {
        let data_service_id = data_service_id.to_string();
        let old_model = dataservice::Entity::find_by_id(data_service_id)
            .one(&self.db_connection)
            .await;
        let old_model = match old_model {
            Ok(old_model) => match old_model {
                Some(old_model) => old_model,
                None => {
                    return Err(CatalogAgentRepoErrors::DataServiceRepoErrors(
                        DataServiceRepoErrors::DataServiceNotFound,
                    )
                    .into_errors())
                }
            },
            Err(err) => {
                return Err(CatalogAgentRepoErrors::DataServiceRepoErrors(
                    DataServiceRepoErrors::ErrorFetchingDataService(err.into()),
                )
                .into_errors())
            }
        };
        let mut old_active_model: dataservice::ActiveModel = old_model.into();
        if let Some(dcat_endpoint_description) = &edit_data_service_model.dcat_endpoint_description
        {
            old_active_model.dcat_endpoint_description =
                ActiveValue::Set(Some(dcat_endpoint_description.clone()));
        }
        if let Some(dcat_endpoint_url) = &edit_data_service_model.dcat_endpoint_url {
            old_active_model.dcat_endpoint_url = ActiveValue::Set(dcat_endpoint_url.clone());
        }
        if let Some(dct_conforms_to) = &edit_data_service_model.dct_conforms_to {
            old_active_model.dct_conforms_to = ActiveValue::Set(Some(dct_conforms_to.clone()));
        }
        if let Some(dct_creator) = &edit_data_service_model.dct_creator {
            old_active_model.dct_creator = ActiveValue::Set(Some(dct_creator.clone()));
        }
        if let Some(dct_title) = &edit_data_service_model.dct_title {
            old_active_model.dct_title = ActiveValue::Set(Some(dct_title.clone()));
        }
        if let Some(dct_description) = &edit_data_service_model.dct_description {
            old_active_model.dct_description = ActiveValue::Set(Some(dct_description.clone()));
        }

        old_active_model.dct_modified = ActiveValue::Set(Some(chrono::Utc::now().into()));
        let model = old_active_model.update(&self.db_connection).await;
        match model {
            Ok(model) => Ok(model),
            Err(err) => Err(CatalogAgentRepoErrors::DataServiceRepoErrors(
                DataServiceRepoErrors::ErrorUpdatingDataService(err.into()),
            )
            .into_errors()),
        }
    }

    async fn create_data_service(
        &self,
        new_data_service_model: &NewDataServiceModel,
    ) -> Outcome<dataservice::Model> {
        let catalog =
            catalog::Entity::find_by_id(new_data_service_model.catalog_id.clone().to_string())
                .one(&self.db_connection)
                .await
                .map_err(|err| {
                    CatalogAgentRepoErrors::DistributionRepoErrors(
                        DistributionRepoErrors::ErrorFetchingDistribution(err.into()),
                    )
                    .into_errors()
                })?;
        if catalog.is_none() {
            return Err(CatalogAgentRepoErrors::CatalogRepoErrors(
                CatalogRepoErrors::CatalogNotFound,
            )
            .into_errors());
        }
        let model: dataservice::ActiveModel = new_data_service_model.into();
        let data_service = dataservice::Entity::insert(model)
            .exec_with_returning(&self.db_connection)
            .await;
        match data_service {
            Ok(data_service) => Ok(data_service),
            Err(err) => Err(CatalogAgentRepoErrors::DataServiceRepoErrors(
                DataServiceRepoErrors::ErrorCreatingDataService(err.into()),
            )
            .into_errors()),
        }
    }

    async fn create_main_data_service(
        &self,
        new_data_service_model: &NewDataServiceModel,
    ) -> Outcome<dataservice::Model> {
        let catalog =
            catalog::Entity::find_by_id(new_data_service_model.catalog_id.clone().to_string())
                .one(&self.db_connection)
                .await
                .map_err(|err| {
                    CatalogAgentRepoErrors::DistributionRepoErrors(
                        DistributionRepoErrors::ErrorFetchingDistribution(err.into()),
                    )
                    .into_errors()
                })?;
        if catalog.is_none() {
            return Err(CatalogAgentRepoErrors::CatalogRepoErrors(
                CatalogRepoErrors::CatalogNotFound,
            )
            .into_errors());
        }

        let main_dataservice = self.get_main_data_service().await?;
        if main_dataservice.is_some() {
            return Ok(main_dataservice.unwrap());
        }

        let mut model: dataservice::ActiveModel = new_data_service_model.into();
        model.dspace_main_data_service = ActiveValue::Set(true);
        let data_service = dataservice::Entity::insert(model)
            .exec_with_returning(&self.db_connection)
            .await;
        match data_service {
            Ok(data_service) => Ok(data_service),
            Err(err) => Err(CatalogAgentRepoErrors::DataServiceRepoErrors(
                DataServiceRepoErrors::ErrorCreatingDataService(err.into()),
            )
            .into_errors()),
        }
    }

    async fn delete_data_service_by_id(&self, data_service_id: &Urn) -> Outcome<()> {
        let data_service_id = data_service_id.to_string();
        let data_service = dataservice::Entity::delete_by_id(data_service_id)
            .exec(&self.db_connection)
            .await;
        match data_service {
            Ok(delete_result) => match delete_result.rows_affected {
                0 => Err(CatalogAgentRepoErrors::DataServiceRepoErrors(
                    DataServiceRepoErrors::DataServiceNotFound,
                )
                .into_errors()),
                _ => Ok(()),
            },
            Err(err) => Err(CatalogAgentRepoErrors::DataServiceRepoErrors(
                DataServiceRepoErrors::ErrorDeletingDataService(err.into()),
            )
            .into_errors()),
        }
    }
}
