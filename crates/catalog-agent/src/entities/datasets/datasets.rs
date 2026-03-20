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

use crate::cache::factory_trait::CatalogAgentCacheTrait;
use crate::data::factory_trait::CatalogAgentRepoTrait;
use crate::entities::datasets::{DatasetDto, DatasetEntityTrait, EditDatasetDto, NewDatasetDto};
use log::error;
use common::errors::{CommonErrors, ErrorLog};
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::Outcome;

pub struct DatasetEntities {
    repo: Arc<dyn CatalogAgentRepoTrait>,
    cache: Arc<dyn CatalogAgentCacheTrait>,
}

impl DatasetEntities {
    pub fn new(
        repo: Arc<dyn CatalogAgentRepoTrait>,
        cache: Arc<dyn CatalogAgentCacheTrait>,
    ) -> Self {
        Self { repo, cache }
    }
}

#[async_trait::async_trait]
impl DatasetEntityTrait for DatasetEntities {
    async fn get_all_datasets(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Outcome<Vec<DatasetDto>> {
        // cache
        if let Ok(dtos) = self.cache.get_dataset_cache().get_collection(limit, page).await {
            if !dtos.is_empty() {
                return Ok(dtos);
            }
        }

        // db
        let datasets = self
            .repo
            .get_dataset_repo()
            .get_all_datasets(limit, page)
            .await?;

        let dtos: Vec<DatasetDto> = datasets.into_iter().map(Into::into).collect();

        // hydration
        let cache = self.cache.get_dataset_cache();
        for dto in &dtos {
            if let Ok(id) = Urn::from_str(dto.inner.id.as_str()) {
                let score = dto.inner.dct_issued.timestamp() as f64;
                let _ = cache.set_single(&id, dto).await;
                let _ = cache.add_to_collection(&id, score).await;
            }
        }
        Ok(dtos)
    }

    async fn get_batch_datasets(&self, ids: &Vec<Urn>) -> Outcome<Vec<DatasetDto>> {
        // cache
        if let Ok(dtos) = self.cache.get_dataset_cache().get_batch(ids).await {
            if !dtos.is_empty() {
                return Ok(dtos);
            }
        }

        // db
        let datasets = self
            .repo
            .get_dataset_repo()
            .get_batch_datasets(ids)
            .await?;

        let dtos: Vec<DatasetDto> = datasets.into_iter().map(Into::into).collect();

        // hydration
        let cache = self.cache.get_dataset_cache();
        for dto in &dtos {
            if let Ok(id) = Urn::from_str(dto.inner.id.as_str()) {
                let _ = cache.set_single(&id, dto).await;
                let _ = cache.add_to_collection(&id, dto.inner.dct_issued.timestamp() as f64).await;
            }
        }
        Ok(dtos)
    }

    async fn get_datasets_by_catalog_id(
        &self,
        catalog_id: &Urn,
    ) -> Outcome<Vec<DatasetDto>> {
        // cache
        if let Ok(dtos) =
            self.cache.get_dataset_cache().get_by_relation("catalogs", catalog_id, None, None).await
        {
            if !dtos.is_empty() {
                return Ok(dtos);
            }
        }

        // db fetch
        let datasets = self
            .repo
            .get_dataset_repo()
            .get_datasets_by_catalog_id(catalog_id)
            .await?;

        let dtos: Vec<DatasetDto> = datasets.into_iter().map(Into::into).collect();

        //  hydration
        let cache = self.cache.get_dataset_cache();
        for dto in &dtos {
            if let Ok(id) = Urn::from_str(dto.inner.id.as_str()) {
                let score = dto.inner.dct_issued.timestamp() as f64;
                let _ = cache.set_single(&id, dto).await;
                let _ = cache.add_to_collection(&id, score).await;
                let _ = cache.add_to_relation("catalogs", catalog_id, &id, score).await;
            }
        }
        Ok(dtos)
    }

    async fn get_dataset_by_id(&self, dataset_id: &Urn) -> Outcome<Option<DatasetDto>> {
        // Try cache
        if let Ok(Some(dto)) = self.cache.get_dataset_cache().get_single(dataset_id).await {
            return Ok(Some(dto));
        }

        // Database
        let dataset = self
            .repo
            .get_dataset_repo()
            .get_dataset_by_id(dataset_id)
            .await?;

        let dto: Option<DatasetDto> = dataset.map(Into::into);

        // Safe hydration
        if let Some(dto) = &dto {
            let cache = self.cache.get_dataset_cache();
            let _ = cache.set_single(dataset_id, dto).await;
            let _ =
                cache.add_to_collection(dataset_id, dto.inner.dct_issued.timestamp() as f64).await;
        }
        Ok(dto)
    }

    async fn put_dataset_by_id(
        &self,
        dataset_id: &Urn,
        edit_dataset_model: &EditDatasetDto,
    ) -> Outcome<DatasetDto> {
        // db
        let edit_model = edit_dataset_model.clone().into();
        let dataset = self
            .repo
            .get_dataset_repo()
            .put_dataset_by_id(dataset_id, &edit_model)
            .await?;

        let dto: DatasetDto = dataset.into();
        let ds_urn = Urn::from_str(dto.inner.id.as_str())?;

        // hydration
        let cache = self.cache.get_dataset_cache();
        let _ = cache.set_single(&ds_urn, &dto).await;
        let _ = cache.add_to_collection(&ds_urn, dto.inner.dct_issued.timestamp() as f64).await;

        Ok(dto)
    }

    async fn create_dataset(
        &self,
        new_dataset_model: &NewDatasetDto,
    ) -> Outcome<DatasetDto> {
        // db
        let new_model = new_dataset_model.clone().into();
        let dataset = self
            .repo
            .get_dataset_repo()
            .create_dataset(&new_model)
            .await?;

        let dto: DatasetDto = dataset.into();
        let ds_urn = Urn::from_str(dto.inner.id.as_str())?;
        let score = dto.inner.dct_issued.timestamp() as f64;

        // hydration
        let cache = self.cache.get_dataset_cache();
        let _ = cache.set_single(&ds_urn, &dto).await;
        let _ = cache.add_to_collection(&ds_urn, score).await;

        // lookup cache hydration
        if let Ok(catalog_id) = Urn::from_str(&*dto.inner.catalog_id) {
            let _ = cache.add_to_relation("catalogs", &catalog_id, &ds_urn, score).await;
        }

        Ok(dto)
    }

    async fn delete_dataset_by_id(&self, dataset_id: &Urn) -> Outcome<()> {
        // 1. Get current for parent URN lookup
        let current = self.get_dataset_by_id(dataset_id).await?;

        // 2. Database
        self.repo
            .get_dataset_repo()
            .delete_dataset_by_id(dataset_id)
            .await?;

        // 3. Invalidation
        let cache = self.cache.get_dataset_cache();
        let _ = cache.delete_single(dataset_id).await;
        let _ = cache.remove_from_collection(dataset_id).await;

        // Relation invalidation
        if let Some(dto) = current {
            if let Ok(catalog_id) = Urn::from_str(&*dto.inner.catalog_id) {
                let _ = cache.remove_from_relation("catalogs", &catalog_id, dataset_id).await;
            }
        }

        Ok(())
    }
}
