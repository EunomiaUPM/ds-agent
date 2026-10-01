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

use crate::cache::factory_trait::CatalogAgentCacheTrait;
use crate::data::entities::dataset::NewDatasetModel;
use crate::data::factory_trait::CatalogAgentRepoTrait;
use crate::entities::datasets::{DatasetDto, EditDatasetDto, NewDatasetDto};
use crate::entities::filters::DatasetFilter;
use crate::services::datasets::DatasetServiceTrait;
use common::auth::AccessScope;
use common::errors::NotFoundExt;
use common::paginated_spec::{Cursor, Page, Paginated, Sort};
use common::query::QueryFilter;
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::Outcome;

pub struct DatasetService {
    repo: Arc<dyn CatalogAgentRepoTrait>,
    cache: Arc<dyn CatalogAgentCacheTrait>,
    event_bus: Option<events::EventBus>,
}

impl DatasetService {
    pub fn new(
        repo: Arc<dyn CatalogAgentRepoTrait>,
        cache: Arc<dyn CatalogAgentCacheTrait>,
    ) -> Self {
        Self {
            repo,
            cache,
            event_bus: None,
        }
    }

    pub fn with_event_bus(mut self, event_bus: Option<events::EventBus>) -> Self {
        self.event_bus = event_bus;
        self
    }
}

#[async_trait::async_trait]
impl DatasetServiceTrait for DatasetService {
    #[tracing::instrument(level = "info", skip_all, err, fields(tenant = %scope.acting_tenant()))]
    async fn get_all_datasets(
        &self,
        scope: &AccessScope,
        filters: &DatasetFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<DatasetDto>> {
        scope.require_read()?;
        filters.validate()?;
        let mut filters = filters.clone();
        filters.tenant_id = scope.resolve_query_tenant(filters.tenant_id.as_deref())?;
        let page = page.clamped();

        let (datasets, total) = self
            .repo
            .get_dataset_repo()
            .get_all_datasets(&filters, &page, sort)
            .await?;

        let dtos: Vec<DatasetDto> = datasets.into_iter().map(Into::into).collect();

        Ok(Paginated::from_page(dtos, &page, total, |d| {
            Cursor::encode_composite(&d.inner.dct_issued, &d.inner.id)
        }))
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(tenant = %scope.acting_tenant()))]
    async fn get_batch_datasets(
        &self,
        scope: &AccessScope,
        ids: &[Urn],
    ) -> Outcome<Vec<DatasetDto>> {
        scope.require_read()?;
        let datasets = self
            .repo
            .get_dataset_repo()
            .get_batch_datasets(scope.tenant_filter().map(str::to_string), ids)
            .await?;

        let mut dtos: Vec<DatasetDto> = Vec::new();
        let cache = self.cache.get_dataset_cache();
        for ds in datasets {
            let dto: DatasetDto = ds.into();
            if let Ok(id) = Urn::from_str(dto.inner.id.as_str()) {
                let _ = cache.set_single(&id, &dto).await;
                let _ = cache
                    .add_to_collection(&id, dto.inner.dct_issued.timestamp() as f64)
                    .await;
            }
            dtos.push(dto);
        }
        Ok(dtos)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(tenant = %scope.acting_tenant()))]
    async fn get_datasets_by_catalog_id(
        &self,
        scope: &AccessScope,
        catalog_id: &Urn,
    ) -> Outcome<Vec<DatasetDto>> {
        scope.require_read()?;
        let datasets = self
            .repo
            .get_dataset_repo()
            .get_datasets_by_catalog_id(scope.tenant_filter().map(str::to_string), catalog_id)
            .await?;

        let mut dtos: Vec<DatasetDto> = Vec::new();
        let cache = self.cache.get_dataset_cache();
        for ds in datasets {
            let dto: DatasetDto = ds.into();
            if let Ok(id) = Urn::from_str(dto.inner.id.as_str()) {
                let score = dto.inner.dct_issued.timestamp() as f64;
                let _ = cache.set_single(&id, &dto).await;
                let _ = cache.add_to_collection(&id, score).await;
                let _ = cache
                    .add_to_relation("catalogs", catalog_id, &id, score)
                    .await;
            }
            dtos.push(dto);
        }
        Ok(dtos)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(tenant = %scope.acting_tenant()))]
    async fn get_dataset_by_id(
        &self,
        scope: &AccessScope,
        dataset_id: &Urn,
    ) -> Outcome<DatasetDto> {
        scope.require_read()?;
        let dataset = self
            .repo
            .get_dataset_repo()
            .get_dataset_by_id(scope.tenant_filter().map(str::to_string), dataset_id)
            .await?
            .or_not_found(dataset_id, "dataset")?;

        let dto: DatasetDto = dataset.into();

        let cache = self.cache.get_dataset_cache();
        let _ = cache.set_single(dataset_id, &dto).await;
        let _ = cache
            .add_to_collection(dataset_id, dto.inner.dct_issued.timestamp() as f64)
            .await;
        Ok(dto)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(tenant = %scope.acting_tenant()))]
    async fn put_dataset_by_id(
        &self,
        scope: &AccessScope,
        dataset_id: &Urn,
        edit_dataset_model: &EditDatasetDto,
    ) -> Outcome<DatasetDto> {
        scope.require_write()?;
        let edit_model = edit_dataset_model.clone().into();
        let dataset = self
            .repo
            .get_dataset_repo()
            .put_dataset_by_id(
                scope.tenant_filter().map(str::to_string),
                dataset_id,
                &edit_model,
            )
            .await?;

        let dto: DatasetDto = dataset.into();
        let ds_urn = Urn::from_str(dto.inner.id.as_str())?;

        let cache = self.cache.get_dataset_cache();
        let _ = cache.set_single(&ds_urn, &dto).await;
        let _ = cache
            .add_to_collection(&ds_urn, dto.inner.dct_issued.timestamp() as f64)
            .await;

        events::emit_action!(
            self.event_bus,
            &dto.inner.tenant_id,
            crate::EVENT_PREFIX,
            "dataset",
            "edit",
            &dto
        );
        Ok(dto)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(tenant = %scope.acting_tenant()))]
    async fn create_dataset(
        &self,
        scope: &AccessScope,
        new_dataset_model: &NewDatasetDto,
    ) -> Outcome<DatasetDto> {
        let mut new_dataset_model = new_dataset_model.clone();
        let tenant_id = scope.resolve_create_tenant(new_dataset_model.tenant_id.as_deref())?;
        new_dataset_model.tenant_id = Some(tenant_id.clone());
        let new_model: NewDatasetModel = new_dataset_model.into_model(tenant_id);
        let dataset = self
            .repo
            .get_dataset_repo()
            .create_dataset(&new_model)
            .await?;

        let dto: DatasetDto = dataset.into();
        let ds_urn = Urn::from_str(dto.inner.id.as_str())?;
        let score = dto.inner.dct_issued.timestamp() as f64;

        let cache = self.cache.get_dataset_cache();
        let _ = cache.set_single(&ds_urn, &dto).await;
        let _ = cache.add_to_collection(&ds_urn, score).await;

        if let Ok(catalog_id) = Urn::from_str(&dto.inner.catalog_id) {
            let _ = cache
                .add_to_relation("catalogs", &catalog_id, &ds_urn, score)
                .await;
        }

        events::emit_action!(
            self.event_bus,
            &dto.inner.tenant_id,
            crate::EVENT_PREFIX,
            "dataset",
            "create",
            &dto
        );
        Ok(dto)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(tenant = %scope.acting_tenant()))]
    async fn delete_dataset_by_id(&self, scope: &AccessScope, dataset_id: &Urn) -> Outcome<()> {
        scope.require_write()?;
        let deleted = self
            .repo
            .get_dataset_repo()
            .delete_dataset_by_id(scope.tenant_filter().map(str::to_string), dataset_id)
            .await?;

        let cache = self.cache.get_dataset_cache();
        let _ = cache.delete_single(dataset_id).await;
        let _ = cache.remove_from_collection(dataset_id).await;
        if let Ok(catalog_id) = Urn::from_str(&deleted.catalog_id) {
            let _ = cache
                .remove_from_relation("catalogs", &catalog_id, dataset_id)
                .await;
        }

        events::emit_action!(
            self.event_bus,
            &deleted.tenant_id,
            crate::EVENT_PREFIX,
            "dataset",
            "delete",
            &events::EntityDeletedDto::new(dataset_id)
        );
        Ok(())
    }
}
