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
use crate::entities::distributions::{
    DistributionDto, DistributionEntityTrait, EditDistributionDto, NewDistributionDto,
};
use common::errors::{CommonErrors, ErrorLog};
use std::str::FromStr;
use std::sync::Arc;
use tracing::error;
use urn::Urn;
use ymir::errors::Outcome;

pub struct DistributionEntities {
    repo: Arc<dyn CatalogAgentRepoTrait>,
    cache: Arc<dyn CatalogAgentCacheTrait>,
}

impl DistributionEntities {
    pub fn new(
        repo: Arc<dyn CatalogAgentRepoTrait>,
        cache: Arc<dyn CatalogAgentCacheTrait>,
    ) -> Self {
        Self { repo, cache }
    }
}

#[async_trait::async_trait]
impl DistributionEntityTrait for DistributionEntities {
    async fn get_all_distributions(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Outcome<Vec<DistributionDto>> {
        // cache
        if let Ok(dtos) = self
            .cache
            .get_distribution_cache()
            .get_collection(limit, page)
            .await
        {
            if !dtos.is_empty() {
                return Ok(dtos);
            }
        }

        // db
        let distributions = self
            .repo
            .get_distribution_repo()
            .get_all_distributions(limit, page)
            .await?;

        let dtos: Vec<DistributionDto> = distributions.into_iter().map(Into::into).collect();

        // hydration
        let cache = self.cache.get_distribution_cache();
        for dto in &dtos {
            if let Ok(id) = Urn::from_str(dto.inner.id.as_str()) {
                let score = dto.inner.dct_issued.timestamp() as f64;
                let _ = cache.set_single(&id, dto).await;
                let _ = cache.add_to_collection(&id, score).await;
            }
        }
        Ok(dtos)
    }

    async fn get_batch_distributions(&self, ids: &Vec<Urn>) -> Outcome<Vec<DistributionDto>> {
        // cache
        if let Ok(dtos) = self.cache.get_distribution_cache().get_batch(ids).await {
            if !dtos.is_empty() {
                return Ok(dtos);
            }
        }

        // db
        let distributions = self
            .repo
            .get_distribution_repo()
            .get_batch_distributions(ids)
            .await?;

        let dtos: Vec<DistributionDto> = distributions.into_iter().map(Into::into).collect();

        // hydration
        let cache = self.cache.get_distribution_cache();
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

    async fn get_distributions_by_dataset_id(
        &self,
        dataset_id: &Urn,
    ) -> Outcome<Vec<DistributionDto>> {
        // cache
        if let Ok(dtos) = self
            .cache
            .get_distribution_cache()
            .get_by_relation("datasets", dataset_id, None, None)
            .await
        {
            if !dtos.is_empty() {
                return Ok(dtos);
            }
        }

        // Database fetch
        let distributions = self
            .repo
            .get_distribution_repo()
            .get_distributions_by_dataset_id(dataset_id)
            .await?;

        let dtos: Vec<DistributionDto> = distributions.into_iter().map(Into::into).collect();

        // hydration
        let cache = self.cache.get_distribution_cache();
        for dto in &dtos {
            if let Ok(id) = Urn::from_str(dto.inner.id.as_str()) {
                let score = dto.inner.dct_issued.timestamp() as f64;
                let _ = cache.set_single(&id, dto).await;
                let _ = cache.add_to_collection(&id, score).await;
                let _ = cache
                    .add_to_relation("datasets", dataset_id, &id, score)
                    .await;
            }
        }
        Ok(dtos)
    }

    async fn get_distribution_by_dataset_id_and_dct_format(
        &self,
        dataset_id: &Urn,
        dct_formats: &String,
    ) -> Outcome<DistributionDto> {
        let distribution = self
            .repo
            .get_distribution_repo()
            .get_distribution_by_dataset_id_and_dct_format(dataset_id, dct_formats)
            .await?;

        let dto: DistributionDto = distribution.into();

        // Hydrate single
        if let Ok(id) = Urn::from_str(dto.inner.id.as_str()) {
            let _ = self
                .cache
                .get_distribution_cache()
                .set_single(&id, &dto)
                .await;
        }

        Ok(dto)
    }

    async fn get_distribution_by_id(
        &self,
        distribution_id: &Urn,
    ) -> Outcome<Option<DistributionDto>> {
        // Cache
        if let Ok(Some(dto)) = self
            .cache
            .get_distribution_cache()
            .get_single(distribution_id)
            .await
        {
            return Ok(Some(dto));
        }

        // Database
        let distribution = self
            .repo
            .get_distribution_repo()
            .get_distribution_by_id(distribution_id)
            .await?;

        let dto: Option<DistributionDto> = distribution.map(Into::into);

        // Safe hydration
        if let Some(dto) = &dto {
            let cache = self.cache.get_distribution_cache();
            let _ = cache.set_single(distribution_id, dto).await;
            let _ = cache
                .add_to_collection(distribution_id, dto.inner.dct_issued.timestamp() as f64)
                .await;
        }
        Ok(dto)
    }

    async fn put_distribution_by_id(
        &self,
        distribution_id: &Urn,
        edit_distribution_model: &EditDistributionDto,
    ) -> Outcome<DistributionDto> {
        let edit_model = edit_distribution_model.clone().into();
        let distribution = self
            .repo
            .get_distribution_repo()
            .put_distribution_by_id(distribution_id, &edit_model)
            .await?;

        let dto: DistributionDto = distribution.into();
        let dist_urn = Urn::from_str(dto.inner.id.as_str())?;

        // Update single and score
        let cache = self.cache.get_distribution_cache();
        let _ = cache.set_single(&dist_urn, &dto).await;
        let _ = cache
            .add_to_collection(&dist_urn, dto.inner.dct_issued.timestamp() as f64)
            .await;

        Ok(dto)
    }

    async fn create_distribution(
        &self,
        new_distribution_model: &NewDistributionDto,
    ) -> Outcome<DistributionDto> {
        let new_model = new_distribution_model.clone().into();
        let distribution = self
            .repo
            .get_distribution_repo()
            .create_distribution(&new_model)
            .await?;

        let dto: DistributionDto = distribution.into();
        let dist_urn = Urn::from_str(dto.inner.id.as_str())?;
        let score = dto.inner.dct_issued.timestamp() as f64;

        // Proactive hydration
        let cache = self.cache.get_distribution_cache();
        let _ = cache.set_single(&dist_urn, &dto).await;
        let _ = cache.add_to_collection(&dist_urn, score).await;

        // Lookup hydration (Distribution -> Dataset)
        if let Ok(dataset_id) = Urn::from_str(&*dto.inner.dataset_id) {
            let _ = cache
                .add_to_relation("datasets", &dataset_id, &dist_urn, score)
                .await;
        }

        Ok(dto)
    }

    async fn delete_distribution_by_id(&self, distribution_id: &Urn) -> Outcome<()> {
        let current = self.get_distribution_by_id(distribution_id).await?;

        // db
        self.repo
            .get_distribution_repo()
            .delete_distribution_by_id(distribution_id)
            .await?;

        // cache invalidation
        let cache = self.cache.get_distribution_cache();
        let _ = cache.delete_single(distribution_id).await;
        let _ = cache.remove_from_collection(distribution_id).await;

        // lookup invalidation
        if let Some(dto) = current {
            if let Ok(dataset_id) = Urn::from_str(&*dto.inner.dataset_id) {
                let _ = cache
                    .remove_from_relation("datasets", &dataset_id, distribution_id)
                    .await;
            }
        }

        Ok(())
    }
}
