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

use crate::data::entities::catalog;
use crate::data::entities::catalog::{EditCatalogModel, NewCatalogModel};
use crate::data::repo_traits::catalog_db_errors::{CatalogAgentRepoErrors, CatalogRepoErrors};
use crate::data::repo_traits::catalog_repo::CatalogRepositoryTrait;
use crate::entities::filters::CatalogFilter;
use common::paginated_spec::{Page, SelectCursorExt, Sort};
use common::query::FilterApplier;
use sea_orm::QueryTrait;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, Select,
};
use urn::Urn;
use ymir::errors::{Outcome, RepoIntoErrors};

impl FilterApplier<Select<catalog::Entity>> for CatalogFilter {
    fn apply_to(&self, mut q: Select<catalog::Entity>) -> Select<catalog::Entity> {
        if let Some(ref tenant_id) = self.tenant_id {
            q = q.filter(catalog::Column::TenantId.eq(tenant_id));
        }
        if let Some(ref title) = self.title {
            q = q.filter(catalog::Column::DctTitle.contains(title));
        }
        if let Some(ref creator) = self.creator {
            q = q.filter(catalog::Column::DctCreator.eq(creator));
        }
        if let Some(ref participant_id) = self.participant_id {
            q = q.filter(catalog::Column::DspaceParticipantId.eq(participant_id));
        }
        if let Some(false) = self.with_main_catalog {
            q = q.filter(catalog::Column::DspaceMainCatalog.eq(false));
        }
        if let Some(after) = self.created_after {
            q = q.filter(catalog::Column::DctIssued.gte(after));
        }
        if let Some(before) = self.created_before {
            q = q.filter(catalog::Column::DctIssued.lte(before));
        }
        q
    }
}

pub struct CatalogRepositoryForSql {
    db_connection: DatabaseConnection,
}

impl CatalogRepositoryForSql {
    pub fn new(db_connection: DatabaseConnection) -> Self {
        Self { db_connection }
    }
}

#[async_trait::async_trait]
impl CatalogRepositoryTrait for CatalogRepositoryForSql {
    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_all_catalogs(
        &self,
        filters: &CatalogFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<(Vec<catalog::Model>, Option<u64>)> {
        let q = filters.apply_to(catalog::Entity::find());

        let total = q.clone().count(&self.db_connection).await.map_err(|err| {
            CatalogAgentRepoErrors::CatalogRepoErrors(CatalogRepoErrors::ErrorFetchingCatalog(
                err.into(),
            ))
            .into_errors()
        })?;

        let items = q
            .apply_cursor_pagination_with_tie_break(
                page,
                sort,
                catalog::Column::DctIssued,
                catalog::Column::Id,
            )
            .all(&self.db_connection)
            .await
            .map_err(|err| {
                CatalogAgentRepoErrors::CatalogRepoErrors(CatalogRepoErrors::ErrorFetchingCatalog(
                    err.into(),
                ))
                .into_errors()
            })?;

        Ok((items, Some(total)))
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_batch_catalogs(
        &self,
        tenant_id: Option<String>,
        ids: &[Urn],
    ) -> Outcome<Vec<catalog::Model>> {
        let catalog_ids = ids.iter().map(|t| t.to_string()).collect::<Vec<_>>();
        let catalog_process = catalog::Entity::find()
            .apply_if(tenant_id, |q, t| q.filter(catalog::Column::TenantId.eq(t)))
            .filter(catalog::Column::Id.is_in(catalog_ids))
            .all(&self.db_connection)
            .await;
        match catalog_process {
            Ok(catalog_process) => Ok(catalog_process),
            Err(err) => Err(CatalogAgentRepoErrors::CatalogRepoErrors(
                CatalogRepoErrors::ErrorFetchingCatalog(err.into()),
            )
            .into_errors()),
        }
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_catalog_by_id(
        &self,
        tenant_id: Option<String>,
        catalog_id: &Urn,
    ) -> Outcome<Option<catalog::Model>> {
        let catalog_id = catalog_id.to_string();
        let catalog = catalog::Entity::find_by_id(catalog_id)
            .apply_if(tenant_id, |q, t| q.filter(catalog::Column::TenantId.eq(t)))
            .one(&self.db_connection)
            .await;
        match catalog {
            Ok(catalog) => Ok(catalog),
            Err(err) => Err(CatalogAgentRepoErrors::CatalogRepoErrors(
                CatalogRepoErrors::ErrorFetchingCatalog(err.into()),
            )
            .into_errors()),
        }
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn get_main_catalog(&self, tenant_id: &str) -> Outcome<Option<catalog::Model>> {
        let catalog = catalog::Entity::find()
            .filter(catalog::Column::TenantId.eq(tenant_id))
            .filter(catalog::Column::DspaceMainCatalog.eq(true))
            .one(&self.db_connection)
            .await
            .map_err(|err| {
                CatalogAgentRepoErrors::CatalogRepoErrors(CatalogRepoErrors::ErrorFetchingCatalog(
                    err.into(),
                ))
                .into_errors()
            })?;
        Ok(catalog)
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn put_catalog_by_id(
        &self,
        tenant_id: Option<String>,
        catalog_id: &Urn,
        edit_catalog_model: &EditCatalogModel,
    ) -> Outcome<catalog::Model> {
        let catalog_id = catalog_id.to_string();
        let old_model = catalog::Entity::find_by_id(catalog_id)
            .apply_if(tenant_id, |q, t| q.filter(catalog::Column::TenantId.eq(t)))
            .one(&self.db_connection)
            .await;
        let old_model = match old_model {
            Ok(old_model) => match old_model {
                Some(old_model) => old_model,
                None => {
                    return Err(CatalogAgentRepoErrors::CatalogRepoErrors(
                        CatalogRepoErrors::CatalogNotFound,
                    )
                    .into_errors())
                }
            },
            Err(err) => {
                return Err(CatalogAgentRepoErrors::CatalogRepoErrors(
                    CatalogRepoErrors::ErrorFetchingCatalog(err.into()),
                )
                .into_errors())
            }
        };

        let mut old_active_model: catalog::ActiveModel = old_model.into();
        if let Some(foaf_home_page) = &edit_catalog_model.foaf_home_page {
            old_active_model.foaf_home_page = ActiveValue::Set(Some(foaf_home_page.clone()));
        }
        if let Some(dct_conforms_to) = &edit_catalog_model.dct_conforms_to {
            old_active_model.dct_conforms_to = ActiveValue::Set(Some(dct_conforms_to.clone()));
        }
        if let Some(dct_creator) = &edit_catalog_model.dct_creator {
            old_active_model.dct_creator = ActiveValue::Set(Some(dct_creator.clone()));
        }
        if let Some(dct_title) = &edit_catalog_model.dct_title {
            old_active_model.dct_title = ActiveValue::Set(Some(dct_title.clone()));
        }
        old_active_model.dct_modified = ActiveValue::Set(Some(chrono::Utc::now().into()));

        let model = old_active_model.update(&self.db_connection).await;
        match model {
            Ok(model) => Ok(model),
            Err(err) => Err(CatalogAgentRepoErrors::CatalogRepoErrors(
                CatalogRepoErrors::ErrorUpdatingCatalog(err.into()),
            )
            .into_errors()),
        }
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn create_catalog(&self, new_catalog_model: &NewCatalogModel) -> Outcome<catalog::Model> {
        let main_catalog = self.get_main_catalog(&new_catalog_model.tenant_id).await?;
        if main_catalog.is_none() {
            return Err(CatalogAgentRepoErrors::CatalogRepoErrors(
                CatalogRepoErrors::ErrorCreatingCatalog(
                    "Main Catalog must be created first".into(),
                ),
            )
            .into_errors());
        }
        let model: catalog::ActiveModel = new_catalog_model.clone().into();
        let catalog = catalog::Entity::insert(model)
            .exec_with_returning(&self.db_connection)
            .await;
        match catalog {
            Ok(catalog) => Ok(catalog),
            Err(err) => Err(CatalogAgentRepoErrors::CatalogRepoErrors(
                CatalogRepoErrors::ErrorCreatingCatalog(err.into()),
            )
            .into_errors()),
        }
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn create_main_catalog(
        &self,
        new_catalog_model: &NewCatalogModel,
    ) -> Outcome<catalog::Model> {
        let main_catalog = self.get_main_catalog(&new_catalog_model.tenant_id).await?;
        if main_catalog.is_some() {
            return Ok(main_catalog.unwrap());
        }

        let mut model: catalog::ActiveModel = new_catalog_model.into();
        model.dspace_main_catalog = ActiveValue::Set(true);
        let catalog = catalog::Entity::insert(model)
            .exec_with_returning(&self.db_connection)
            .await;
        match catalog {
            Ok(catalog) => Ok(catalog),
            Err(err) => Err(CatalogAgentRepoErrors::CatalogRepoErrors(
                CatalogRepoErrors::ErrorCreatingCatalog(err.into()),
            )
            .into_errors()),
        }
    }

    #[tracing::instrument(level = "debug", skip_all, err)]
    async fn delete_catalog_by_id(
        &self,
        tenant_id: Option<String>,
        catalog_id: &Urn,
    ) -> Outcome<catalog::Model> {
        let deleted = catalog::Entity::delete_many()
            .filter(catalog::Column::Id.eq(catalog_id.to_string()))
            .apply_if(tenant_id, |q, t| q.filter(catalog::Column::TenantId.eq(t)))
            .exec_with_returning(&self.db_connection)
            .await
            .map_err(|err| {
                CatalogAgentRepoErrors::CatalogRepoErrors(CatalogRepoErrors::ErrorDeletingCatalog(
                    err.into(),
                ))
                .into_errors()
            })?;
        deleted.into_iter().next().ok_or_else(|| {
            CatalogAgentRepoErrors::CatalogRepoErrors(CatalogRepoErrors::CatalogNotFound)
                .into_errors()
        })
    }
}
