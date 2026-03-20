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
use crate::data::entities::catalog::{EditCatalogModel, NewCatalogModel};
use crate::data::factory_trait::CatalogAgentRepoTrait;
use crate::entities::catalogs::{CatalogDto, CatalogEntityTrait, EditCatalogDto, NewCatalogDto};
use common::errors::{CommonErrors, ErrorLog};
use log::error;
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::Outcome;

pub struct CatalogEntities {
    repo: Arc<dyn CatalogAgentRepoTrait>,
    cache: Arc<dyn CatalogAgentCacheTrait>,
}

impl CatalogEntities {
    pub fn new(
        repo: Arc<dyn CatalogAgentRepoTrait>,
        cache: Arc<dyn CatalogAgentCacheTrait>,
    ) -> Self {
        Self { repo, cache }
    }
}

#[async_trait::async_trait]
impl CatalogEntityTrait for CatalogEntities {
    async fn get_all_catalogs(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
        with_main_catalog: bool,
    ) -> Outcome<Vec<CatalogDto>> {
        // cache
        if !with_main_catalog {
            if let Ok(dtos) = self
                .cache
                .get_catalog_cache()
                .get_collection(limit, page)
                .await
            {
                if !dtos.is_empty() {
                    return Ok(dtos);
                }
            }
        }

        // db
        let catalogs = self
            .repo
            .get_catalog_repo()
            .get_all_catalogs(limit, page, with_main_catalog)
            .await?;

        let dtos: Vec<CatalogDto> = catalogs.into_iter().map(Into::into).collect();

        // hydration
        let cache = self.cache.get_catalog_cache();
        for dto in &dtos {
            if let Ok(id) = Urn::from_str(dto.inner.id.as_str()) {
                let score = dto.inner.dct_issued.timestamp() as f64;
                let _ = cache.set_single(&id, dto).await;
                let _ = cache.add_to_collection(&id, score).await;
            }
        }

        Ok(dtos)
    }

    async fn get_batch_catalogs(&self, ids: &Vec<Urn>) -> Outcome<Vec<CatalogDto>> {
        // cache
        if let Ok(dtos) = self.cache.get_catalog_cache().get_batch(ids).await {
            if !dtos.is_empty() {
                return Ok(dtos);
            }
        }

        // db
        let catalogs = self.repo.get_catalog_repo().get_batch_catalogs(ids).await?;

        let dtos: Vec<CatalogDto> = catalogs.into_iter().map(Into::into).collect();

        // hydration
        let cache = self.cache.get_catalog_cache();
        for dto in &dtos {
            if let Ok(id) = Urn::from_str(dto.inner.id.as_str()) {
                let _ = cache.set_single(&id, dto).await;
                let _ = cache
                    .add_to_collection(&id, dto.inner.dct_issued.timestamp() as f64)
                    .await;
            }
        }

        Ok(dtos)
    }

    async fn get_catalog_by_id(&self, catalog_id: &Urn) -> Outcome<Option<CatalogDto>> {
        // cache
        if let Ok(Some(catalog)) = self.cache.get_catalog_cache().get_single(catalog_id).await {
            return Ok(Some(catalog));
        }

        // db
        let catalog = self
            .repo
            .get_catalog_repo()
            .get_catalog_by_id(catalog_id)
            .await?;

        let dto: Option<CatalogDto> = catalog.map(Into::into);

        // hydration for single entry
        if let Some(dto) = &dto {
            let cache = self.cache.get_catalog_cache();
            let _ = cache.set_single(catalog_id, dto).await;
            let _ = cache
                .add_to_collection(catalog_id, dto.inner.dct_issued.timestamp() as f64)
                .await;
        }

        Ok(dto)
    }

    async fn get_main_catalog(&self) -> Outcome<Option<CatalogDto>> {
        // cache
        let catalog = self.cache.get_catalog_cache().get_main().await?;
        if let Some(catalog) = catalog {
            return Ok(Some(catalog));
        }

        let catalog = self.repo.get_catalog_repo().get_main_catalog().await?;
        let dto: Option<CatalogDto> = catalog.map(|c| c.into());

        // hydrate cache
        if let Some(dto) = &dto {
            let main_id = Urn::from_str(&*dto.inner.id)?;
            let catalog_timestamp = dto.inner.dct_issued.timestamp() as f64;
            self.cache.get_catalog_cache().set_main(&main_id, dto).await;
        }

        Ok(dto)
    }

    async fn put_catalog_by_id(
        &self,
        catalog_id: &Urn,
        edit_catalog_model: &EditCatalogDto,
    ) -> Outcome<CatalogDto> {
        let edit_model: EditCatalogModel = edit_catalog_model.clone().into();
        let catalog = self
            .repo
            .get_catalog_repo()
            .put_catalog_by_id(catalog_id, &edit_model)
            .await?;

        let dto: CatalogDto = catalog.into();
        let catalog_urn = Urn::from_str(dto.inner.id.as_str())?;

        // cache
        let cache = self.cache.get_catalog_cache();
        let _ = cache.set_single(&catalog_urn, &dto).await;
        let _ = cache
            .add_to_collection(&catalog_urn, dto.inner.dct_issued.timestamp() as f64)
            .await;

        Ok(dto)
    }

    async fn create_catalog(&self, new_catalog_model: &NewCatalogDto) -> Outcome<CatalogDto> {
        let new_model: NewCatalogModel = new_catalog_model.clone().into();
        let catalog = self
            .repo
            .get_catalog_repo()
            .create_catalog(&new_model)
            .await?;

        let dto: CatalogDto = catalog.into();
        let catalog_urn = Urn::from_str(dto.inner.id.as_str())?;

        // hydration
        let cache = self.cache.get_catalog_cache();
        let _ = cache.set_single(&catalog_urn, &dto).await;
        let _ = cache
            .add_to_collection(&catalog_urn, dto.inner.dct_issued.timestamp() as f64)
            .await;

        Ok(dto)
    }

    async fn create_main_catalog(&self, new_catalog_model: &NewCatalogDto) -> Outcome<CatalogDto> {
        let new_model: NewCatalogModel = new_catalog_model.clone().into();
        let catalog = self
            .repo
            .get_catalog_repo()
            .create_main_catalog(&new_model)
            .await?;
        let catalog_urn = Urn::from_str(&*catalog.id)?;
        let dto = catalog.into();

        // cache
        self.cache
            .get_catalog_cache()
            .set_main(&catalog_urn, &dto)
            .await;
        Ok(dto)
    }

    async fn delete_catalog_by_id(&self, catalog_id: &Urn) -> Outcome<()> {
        self.repo
            .get_catalog_repo()
            .delete_catalog_by_id(catalog_id)
            .await?;

        // invalidate cache
        let _ = self
            .cache
            .get_catalog_cache()
            .delete_single(catalog_id)
            .await;
        let _ = self
            .cache
            .get_catalog_cache()
            .remove_from_collection(catalog_id)
            .await;

        Ok(())
    }
}
