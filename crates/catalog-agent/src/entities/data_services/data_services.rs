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

use crate::cache::cache_traits::lookup_cache_trait::LookupCacheTrait;
use crate::cache::factory_trait::CatalogAgentCacheTrait;
use crate::data::factory_trait::CatalogAgentRepoTrait;
use crate::entities::data_services::{
    DataServiceDto, DataServiceEntityTrait, EditDataServiceDto, NewDataServiceDto,
};
use common::errors::{CommonErrors, ErrorLog};
use log::error;
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::Outcome;

pub struct DataServiceEntities {
    repo: Arc<dyn CatalogAgentRepoTrait>,
    cache: Arc<dyn CatalogAgentCacheTrait>,
}

impl DataServiceEntities {
    pub fn new(
        repo: Arc<dyn CatalogAgentRepoTrait>,
        cache: Arc<dyn CatalogAgentCacheTrait>,
    ) -> Self {
        Self { repo, cache }
    }
}

#[async_trait::async_trait]
impl DataServiceEntityTrait for DataServiceEntities {
    async fn get_all_data_services(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Outcome<Vec<DataServiceDto>> {
        // cache
        if let Ok(dtos) = self
            .cache
            .get_dataservice_cache()
            .get_collection(limit, page)
            .await
        {
            if !dtos.is_empty() {
                return Ok(dtos);
            }
        }

        // db
        let data_services = self
            .repo
            .get_dataservice_repo()
            .get_all_data_services(limit, page)
            .await?;

        let dtos: Vec<DataServiceDto> = data_services.into_iter().map(Into::into).collect();

        // hydration
        let cache = self.cache.get_dataservice_cache();
        for dto in &dtos {
            if let Ok(id) = Urn::from_str(dto.inner.id.as_str()) {
                let score = dto.inner.dct_issued.timestamp() as f64;
                let _ = cache.set_single(&id, dto).await;
                let _ = cache.add_to_collection(&id, score).await;
            }
        }

        Ok(dtos)
    }

    async fn get_batch_data_services(&self, ids: &Vec<Urn>) -> Outcome<Vec<DataServiceDto>> {
        // cache
        if let Ok(dtos) = self.cache.get_dataservice_cache().get_batch(ids).await {
            if !dtos.is_empty() {
                return Ok(dtos);
            }
        }

        // db
        let data_services = self
            .repo
            .get_dataservice_repo()
            .get_batch_data_services(ids)
            .await?;

        let dtos: Vec<DataServiceDto> = data_services.into_iter().map(Into::into).collect();

        // cache hydration
        let cache = self.cache.get_dataservice_cache();
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

    async fn get_data_services_by_catalog_id(
        &self,
        catalog_id: &Urn,
    ) -> Outcome<Vec<DataServiceDto>> {
        // cache hit using LookupCacheTrait
        if let Ok(dtos) = self
            .cache
            .get_dataservice_cache()
            .get_by_relation("catalogs", catalog_id, None, None)
            .await
        {
            if !dtos.is_empty() {
                return Ok(dtos);
            }
        }

        // database fetch
        let data_services = self
            .repo
            .get_dataservice_repo()
            .get_data_services_by_catalog_id(catalog_id)
            .await?;

        let dtos: Vec<DataServiceDto> = data_services.into_iter().map(Into::into).collect();

        // hydration of relation index
        let cache = self.cache.get_dataservice_cache();
        for dto in &dtos {
            if let Ok(id) = Urn::from_str(dto.inner.id.as_str()) {
                let score = dto.inner.dct_issued.timestamp() as f64;
                let _ = cache.set_single(&id, dto).await;
                // Hydrate both global collection and catalog-specific relation
                let _ = cache.add_to_collection(&id, score).await;
                let _ = cache
                    .add_to_relation("catalogs", catalog_id, &id, score)
                    .await;
            }
        }

        Ok(dtos)
    }

    async fn get_main_data_service(&self) -> Outcome<Option<DataServiceDto>> {
        let cache = self.cache.get_dataservice_cache();
        // cache
        if let Ok(Some(dto)) = cache.get_main().await {
            return Ok(Some(dto));
        }
        // db
        let data_service = self
            .repo
            .get_dataservice_repo()
            .get_main_data_service()
            .await?;

        let dto: Option<DataServiceDto> = data_service.map(Into::into);

        // cache hydration
        if let Some(dto) = &dto {
            if let Ok(id) = Urn::from_str(dto.inner.id.as_str()) {
                let _ = cache.set_main(&id, dto).await;
            }
        }
        Ok(dto)
    }

    async fn get_data_service_by_id(
        &self,
        data_service_id: &Urn,
    ) -> Outcome<Option<DataServiceDto>> {
        // cache hit
        if let Ok(Some(dto)) = self
            .cache
            .get_dataservice_cache()
            .get_single(data_service_id)
            .await
        {
            return Ok(Some(dto));
        }

        // db
        let data_service = self
            .repo
            .get_dataservice_repo()
            .get_data_service_by_id(data_service_id)
            .await?;

        let dto: Option<DataServiceDto> = data_service.map(Into::into);

        // cache hydration
        if let Some(dto) = &dto {
            let cache = self.cache.get_dataservice_cache();
            let _ = cache.set_single(data_service_id, dto).await;
            let _ = cache
                .add_to_collection(data_service_id, dto.inner.dct_issued.timestamp() as f64)
                .await;
        }

        Ok(dto)
    }

    async fn put_data_service_by_id(
        &self,
        data_service_id: &Urn,
        edit_data_service_model: &EditDataServiceDto,
    ) -> Outcome<DataServiceDto> {
        let edit_model = edit_data_service_model.clone().into();
        let data_service = self
            .repo
            .get_dataservice_repo()
            .put_data_service_by_id(data_service_id, &edit_model)
            .await?;

        let dto: DataServiceDto = data_service.into();
        let ds_urn = Urn::from_str(dto.inner.id.as_str())?;

        // hydration
        let cache = self.cache.get_dataservice_cache();
        let _ = cache.set_single(&ds_urn, &dto).await;
        let _ = cache
            .add_to_collection(&ds_urn, dto.inner.dct_issued.timestamp() as f64)
            .await;

        Ok(dto)
    }

    async fn create_data_service(
        &self,
        new_data_service_model: &NewDataServiceDto,
    ) -> Outcome<DataServiceDto> {
        // db
        let new_model = new_data_service_model.clone().into();
        let data_service = self
            .repo
            .get_dataservice_repo()
            .create_data_service(&new_model)
            .await?;

        let dto: DataServiceDto = data_service.into();
        let ds_urn = Urn::from_str(dto.inner.id.as_str())?;
        let score = dto.inner.dct_issued.timestamp() as f64;

        // hydration
        let cache = self.cache.get_dataservice_cache();
        let _ = cache.set_single(&ds_urn, &dto).await;
        let _ = cache.add_to_collection(&ds_urn, score).await;

        // lookup cache hydration
        if let Ok(catalog_id) = Urn::from_str(&*dto.inner.catalog_id) {
            let _ = cache
                .add_to_relation("catalogs", &catalog_id, &ds_urn, score)
                .await;
        }

        Ok(dto)
    }

    async fn create_main_data_service(
        &self,
        new_data_service_model: &NewDataServiceDto,
    ) -> Outcome<DataServiceDto> {
        // db
        let new_model = new_data_service_model.clone().into();
        let data_service = self
            .repo
            .get_dataservice_repo()
            .create_main_data_service(&new_model)
            .await?;
        let dto: DataServiceDto = data_service.into();

        // cache
        if let Ok(id) = Urn::from_str(dto.inner.id.as_str()) {
            let _ = self.cache.get_dataservice_cache().set_main(&id, &dto).await;
        }
        Ok(dto)
    }

    async fn delete_data_service_by_id(&self, data_service_id: &Urn) -> Outcome<()> {
        // db self
        let current = self.get_data_service_by_id(data_service_id).await?;

        // db
        self.repo
            .get_dataservice_repo()
            .delete_data_service_by_id(data_service_id)
            .await?;

        // invalidation
        let cache = self.cache.get_dataservice_cache();
        let _ = cache.delete_single(data_service_id).await;
        let _ = cache.remove_from_collection(data_service_id).await;

        // lookup invalidation
        if let Some(dto) = current {
            if let Ok(catalog_id) = Urn::from_str(&*dto.inner.catalog_id) {
                let _ = cache
                    .remove_from_relation("catalogs", &catalog_id, data_service_id)
                    .await;
            }
        }

        Ok(())
    }
}
