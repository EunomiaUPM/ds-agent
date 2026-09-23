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
use crate::entities::filters::DataServiceFilter;
use common::paginated_spec::{Page, SelectCursorExt, Sort};
use common::query::FilterApplier;
use sea_orm::QueryTrait;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, Select,
};
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

impl FilterApplier<Select<dataservice::Entity>> for DataServiceFilter {
    fn apply_to(&self, mut q: Select<dataservice::Entity>) -> Select<dataservice::Entity> {
        if let Some(ref tenant_id) = self.tenant_id {
            q = q.filter(dataservice::Column::TenantId.eq(tenant_id));
        }
        if let Some(ref catalog_id) = self.catalog_id {
            q = q.filter(dataservice::Column::CatalogId.eq(catalog_id));
        }
        if let Some(ref endpoint_url) = self.endpoint_url {
            q = q.filter(dataservice::Column::DcatEndpointUrl.contains(endpoint_url));
        }
        if let Some(ref title) = self.title {
            q = q.filter(dataservice::Column::DctTitle.contains(title));
        }
        if let Some(ref creator) = self.creator {
            q = q.filter(dataservice::Column::DctCreator.eq(creator));
        }
        if let Some(main) = self.main_data_service {
            q = q.filter(dataservice::Column::DspaceMainDataService.eq(main));
        }
        if let Some(after) = self.created_after {
            q = q.filter(dataservice::Column::DctIssued.gte(after));
        }
        if let Some(before) = self.created_before {
            q = q.filter(dataservice::Column::DctIssued.lte(before));
        }
        q
    }
}

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
        filters: &DataServiceFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<(Vec<dataservice::Model>, Option<u64>)> {
        let mut q = dataservice::Entity::find();
        q = filters.apply_to(q);

        let total = q.clone().count(&self.db_connection).await.map_err(|err| {
            CatalogAgentRepoErrors::DataServiceRepoErrors(
                DataServiceRepoErrors::ErrorFetchingDataService(err.into()),
            )
            .into_errors()
        })?;

        let items = q
            .apply_cursor_pagination_with_tie_break(
                page,
                sort,
                dataservice::Column::DctIssued,
                dataservice::Column::Id,
            )
            .all(&self.db_connection)
            .await
            .map_err(|err| {
                CatalogAgentRepoErrors::DataServiceRepoErrors(
                    DataServiceRepoErrors::ErrorFetchingDataService(err.into()),
                )
                .into_errors()
            })?;

        Ok((items, Some(total)))
    }

    async fn get_batch_data_services(
        &self,
        tenant_id: Option<String>,
        ids: &[Urn],
    ) -> Outcome<Vec<dataservice::Model>> {
        let dataset_ids = ids.iter().map(|t| t.to_string()).collect::<Vec<_>>();
        let dataset_process = dataservice::Entity::find()
            .apply_if(tenant_id, |q, t| {
                q.filter(dataservice::Column::TenantId.eq(t))
            })
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
        tenant_id: Option<String>,
        catalog_id: &Urn,
    ) -> Outcome<Vec<dataservice::Model>> {
        let catalog_id = catalog_id.to_string();
        let data_services = dataservice::Entity::find()
            .apply_if(tenant_id, |q, t| {
                q.filter(dataservice::Column::TenantId.eq(t))
            })
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

    async fn get_main_data_service(&self, tenant_id: &str) -> Outcome<Option<dataservice::Model>> {
        let data_service = dataservice::Entity::find()
            .filter(dataservice::Column::TenantId.eq(tenant_id))
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
        tenant_id: Option<String>,
        data_service_id: &Urn,
    ) -> Outcome<Option<dataservice::Model>> {
        let data_service_id = data_service_id.to_string();
        let data_service = dataservice::Entity::find_by_id(data_service_id)
            .apply_if(tenant_id, |q, t| {
                q.filter(dataservice::Column::TenantId.eq(t))
            })
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
        tenant_id: Option<String>,
        data_service_id: &Urn,
        edit_data_service_model: &EditDataServiceModel,
    ) -> Outcome<dataservice::Model> {
        let data_service_id = data_service_id.to_string();
        let old_model = dataservice::Entity::find_by_id(data_service_id)
            .apply_if(tenant_id, |q, t| {
                q.filter(dataservice::Column::TenantId.eq(t))
            })
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
                .filter(catalog::Column::TenantId.eq(&new_data_service_model.tenant_id))
                .one(&self.db_connection)
                .await
                .map_err(|err| {
                    CatalogAgentRepoErrors::CatalogRepoErrors(
                        CatalogRepoErrors::ErrorFetchingCatalog(err.into()),
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
                .filter(catalog::Column::TenantId.eq(&new_data_service_model.tenant_id))
                .one(&self.db_connection)
                .await
                .map_err(|err| {
                    CatalogAgentRepoErrors::CatalogRepoErrors(
                        CatalogRepoErrors::ErrorFetchingCatalog(err.into()),
                    )
                    .into_errors()
                })?;
        if catalog.is_none() {
            return Err(CatalogAgentRepoErrors::CatalogRepoErrors(
                CatalogRepoErrors::CatalogNotFound,
            )
            .into_errors());
        }

        let main_dataservice = self
            .get_main_data_service(&new_data_service_model.tenant_id)
            .await?;
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

    async fn delete_data_service_by_id(
        &self,
        tenant_id: Option<String>,
        data_service_id: &Urn,
    ) -> Outcome<dataservice::Model> {
        // Single round-trip: DELETE ... RETURNING, tenant-scoped; empty result means not found.
        let deleted = dataservice::Entity::delete_many()
            .filter(dataservice::Column::Id.eq(data_service_id.to_string()))
            .apply_if(tenant_id, |q, t| {
                q.filter(dataservice::Column::TenantId.eq(t))
            })
            .exec_with_returning(&self.db_connection)
            .await
            .map_err(|err| {
                CatalogAgentRepoErrors::DataServiceRepoErrors(
                    DataServiceRepoErrors::ErrorDeletingDataService(err.into()),
                )
                .into_errors()
            })?;
        deleted.into_iter().next().ok_or_else(|| {
            CatalogAgentRepoErrors::DataServiceRepoErrors(
                DataServiceRepoErrors::DataServiceNotFound,
            )
            .into_errors()
        })
    }
}
