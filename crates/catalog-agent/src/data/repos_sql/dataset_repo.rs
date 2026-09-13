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

use crate::data::entities::dataset::{EditDatasetModel, NewDatasetModel};
use crate::data::entities::{catalog, dataset};
use crate::data::repo_traits::catalog_db_errors::{
    CatalogAgentRepoErrors, CatalogRepoErrors, DatasetRepoErrors, DistributionRepoErrors,
};
use crate::data::repo_traits::dataset_repo::DatasetRepositoryTrait;
use crate::entities::filters::DatasetFilter;
use common::paginated_spec::{Page, SelectCursorExt, Sort};
use common::query::FilterApplier;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect,
};
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

impl FilterApplier<sea_orm::Select<dataset::Entity>> for DatasetFilter {
    fn apply_to(&self, mut q: sea_orm::Select<dataset::Entity>) -> sea_orm::Select<dataset::Entity> {
        if let Some(catalog_id) = &self.catalog_id {
            q = q.filter(dataset::Column::CatalogId.eq(catalog_id));
        }
        if let Some(title) = &self.title {
            q = q.filter(dataset::Column::DctTitle.contains(title));
        }
        if let Some(creator) = &self.creator {
            q = q.filter(dataset::Column::DctCreator.eq(creator));
        }
        if let Some(conforms_to) = &self.conforms_to {
            q = q.filter(dataset::Column::DctConformsTo.eq(conforms_to));
        }
        if let Some(after) = self.created_after {
            q = q.filter(dataset::Column::DctIssued.gt(after));
        }
        if let Some(before) = self.created_before {
            q = q.filter(dataset::Column::DctIssued.lt(before));
        }
        q
    }
}

pub struct DatasetRepositoryForSql {
    db_connection: DatabaseConnection,
}

impl DatasetRepositoryForSql {
    pub fn new(db_connection: DatabaseConnection) -> Self {
        Self { db_connection }
    }
}

#[async_trait::async_trait]
impl DatasetRepositoryTrait for DatasetRepositoryForSql {
    async fn get_all_datasets(
        &self,
        filters: &DatasetFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<(Vec<dataset::Model>, Option<u64>)> {
        let mut q = filters.apply_to(dataset::Entity::find());
        let total = q
            .clone()
            .count(&self.db_connection)
            .await
            .map_err(|err| {
                CatalogAgentRepoErrors::DatasetRepoErrors(
                    DatasetRepoErrors::ErrorFetchingDataset(err.into()),
                )
                .into_errors()
            })?;

        let datasets = q
            .apply_cursor_pagination_with_tie_break(
                page,
                sort,
                dataset::Column::DctIssued,
                dataset::Column::Id,
            )
            .all(&self.db_connection)
            .await
            .map_err(|err| {
                CatalogAgentRepoErrors::DatasetRepoErrors(
                    DatasetRepoErrors::ErrorFetchingDataset(err.into()),
                )
                .into_errors()
            })?;

        Ok((datasets, Some(total)))
    }

    async fn get_batch_datasets(&self, ids: &Vec<Urn>) -> Outcome<Vec<dataset::Model>> {
        let dataset_ids = ids.iter().map(|t| t.to_string()).collect::<Vec<_>>();
        let dataset_process = dataset::Entity::find()
            .filter(dataset::Column::Id.is_in(dataset_ids))
            .all(&self.db_connection)
            .await;
        match dataset_process {
            Ok(dataset_process) => Ok(dataset_process),
            Err(err) => Err(CatalogAgentRepoErrors::DatasetRepoErrors(
                DatasetRepoErrors::ErrorFetchingDataset(err.into()),
            )
            .into_errors()),
        }
    }

    async fn get_datasets_by_catalog_id(&self, catalog_id: &Urn) -> Outcome<Vec<dataset::Model>> {
        let catalog_id = catalog_id.to_string();

        let catalog = catalog::Entity::find_by_id(catalog_id.clone())
            .one(&self.db_connection)
            .await
            .map_err(|err| {
                CatalogAgentRepoErrors::DatasetRepoErrors(DatasetRepoErrors::ErrorFetchingDataset(
                    err.into(),
                ))
                .into_errors()
            })?;
        if catalog.is_none() {
            return Err(CatalogAgentRepoErrors::CatalogRepoErrors(
                CatalogRepoErrors::CatalogNotFound,
            )
            .into_errors());
        }

        let datasets = dataset::Entity::find()
            .filter(dataset::Column::CatalogId.eq(catalog_id))
            .all(&self.db_connection)
            .await;
        match datasets {
            Ok(datasets) => Ok(datasets),
            Err(err) => Err(CatalogAgentRepoErrors::DatasetRepoErrors(
                DatasetRepoErrors::ErrorFetchingDataset(err.into()),
            )
            .into_errors()),
        }
    }

    async fn get_dataset_by_id(&self, dataset_id: &Urn) -> Outcome<Option<dataset::Model>> {
        let dataset_id = dataset_id.to_string();
        let dataset = dataset::Entity::find_by_id(dataset_id)
            .one(&self.db_connection)
            .await;
        match dataset {
            Ok(dataset) => Ok(dataset),
            Err(err) => Err(CatalogAgentRepoErrors::DatasetRepoErrors(
                DatasetRepoErrors::ErrorFetchingDataset(err.into()),
            )
            .into_errors()),
        }
    }

    async fn put_dataset_by_id(
        &self,
        dataset_id: &Urn,
        edit_dataset_model: &EditDatasetModel,
    ) -> Outcome<dataset::Model> {
        let dataset_id = dataset_id.to_string();

        let old_model = dataset::Entity::find_by_id(dataset_id)
            .one(&self.db_connection)
            .await;
        let old_model = match old_model {
            Ok(old_model) => match old_model {
                Some(old_model) => old_model,
                None => {
                    return Err(CatalogAgentRepoErrors::DatasetRepoErrors(
                        DatasetRepoErrors::DatasetNotFound,
                    )
                    .into_errors())
                }
            },
            Err(err) => {
                return Err(CatalogAgentRepoErrors::DatasetRepoErrors(
                    DatasetRepoErrors::ErrorFetchingDataset(err.into()),
                )
                .into_errors())
            }
        };

        let mut old_active_model: dataset::ActiveModel = old_model.into();
        if let Some(dct_conforms_to) = &edit_dataset_model.dct_conforms_to {
            old_active_model.dct_conforms_to = ActiveValue::Set(Some(dct_conforms_to.clone()));
        }
        if let Some(dct_creator) = &edit_dataset_model.dct_creator {
            old_active_model.dct_creator = ActiveValue::Set(Some(dct_creator.clone()));
        }
        if let Some(dct_title) = &edit_dataset_model.dct_title {
            old_active_model.dct_title = ActiveValue::Set(Some(dct_title.clone()));
        }
        if let Some(dct_description) = &edit_dataset_model.dct_description {
            old_active_model.dct_description = ActiveValue::Set(Some(dct_description.clone()));
        }
        old_active_model.dct_modified = ActiveValue::Set(Some(chrono::Utc::now().into()));

        let model = old_active_model.update(&self.db_connection).await;
        match model {
            Ok(model) => Ok(model),
            Err(err) => Err(CatalogAgentRepoErrors::DatasetRepoErrors(
                DatasetRepoErrors::ErrorUpdatingDataset(err.into()),
            )
            .into_errors()),
        }
    }

    async fn create_dataset(&self, new_dataset_model: &NewDatasetModel) -> Outcome<dataset::Model> {
        let catalog = catalog::Entity::find_by_id(new_dataset_model.catalog_id.clone().to_string())
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

        let model: dataset::ActiveModel = new_dataset_model.into();
        let dataset = dataset::Entity::insert(model)
            .exec_with_returning(&self.db_connection)
            .await;
        match dataset {
            Ok(dataset) => Ok(dataset),
            Err(err) => Err(CatalogAgentRepoErrors::DatasetRepoErrors(
                DatasetRepoErrors::ErrorCreatingDataset(err.into()),
            )
            .into_errors()),
        }
    }

    async fn delete_dataset_by_id(&self, dataset_id: &Urn) -> Outcome<()> {
        let dataset_id = dataset_id.to_string();
        let dataset = dataset::Entity::delete_by_id(dataset_id)
            .exec(&self.db_connection)
            .await;
        match dataset {
            Ok(delete_result) => match delete_result.rows_affected {
                0 => Err(CatalogAgentRepoErrors::DatasetRepoErrors(
                    DatasetRepoErrors::DatasetNotFound,
                )
                .into_errors()),
                _ => Ok(()),
            },
            Err(err) => Err(CatalogAgentRepoErrors::DatasetRepoErrors(
                DatasetRepoErrors::ErrorDeletingDataset(err.into()),
            )
            .into_errors()),
        }
    }
}
