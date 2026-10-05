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

//! Distribution service over the repository and the cache.

use crate::cache::factory_trait::CatalogAgentCacheTrait;
use crate::data::entities::distribution::NewDistributionModel;
use crate::data::factory_trait::CatalogAgentRepoTrait;
use crate::entities::distributions::{DistributionDto, EditDistributionDto, NewDistributionDto};
use crate::entities::filters::DistributionFilter;
use crate::services::distributions::DistributionServiceTrait;
use common::oauth::{OwnedTrait, Owner, OwnerScope, UserInfo};
use common::errors::NotFoundExt;
use common::paginated_spec::{Cursor, Page, Paginated, Sort};
use common::query::QueryFilter;
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::Outcome;

/// Distribution service that writes through to the cache and emits `catalog:` events.
pub struct DistributionService {
    repo: Arc<dyn CatalogAgentRepoTrait>,
    cache: Arc<dyn CatalogAgentCacheTrait>,
    event_bus: Option<events::EventBus>,
}

impl DistributionService {
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

    /// Publishes create, edit and delete events on `event_bus`.
    pub fn with_event_bus(mut self, event_bus: Option<events::EventBus>) -> Self {
        self.event_bus = event_bus;
        self
    }
}

#[async_trait::async_trait]
impl DistributionServiceTrait for DistributionService {
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_all_distributions(
        &self,
        user: &UserInfo,
        filters: &DistributionFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<DistributionDto>> {
        filters.validate()?;
        let page = page.clamped();

        let (distributions, total) = self
            .repo
            .get_distribution_repo()
            .get_all_distributions(&OwnerScope::seeing(user), filters, &page, sort)
            .await?;

        let dtos: Vec<DistributionDto> = distributions.into_iter().map(Into::into).collect();

        Ok(Paginated::from_page(dtos, &page, total, |d| {
            Cursor::encode_composite(&d.inner.dct_issued, &d.inner.id)
        }))
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_batch_distributions(
        &self,
        user: &UserInfo,
        ids: &[Urn],
    ) -> Outcome<Vec<DistributionDto>> {
        let distributions = self
            .repo
            .get_distribution_repo()
            .get_batch_distributions(&OwnerScope::seeing(user), ids)
            .await?;

        let mut dtos: Vec<DistributionDto> = Vec::new();
        let cache = self.cache.get_distribution_cache();
        for d in distributions {
            let dto: DistributionDto = d.into();
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

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_distributions_by_dataset_id(
        &self,
        user: &UserInfo,
        dataset_id: &Urn,
    ) -> Outcome<Vec<DistributionDto>> {
        let distributions = self
            .repo
            .get_distribution_repo()
            .get_distributions_by_dataset_id(&OwnerScope::seeing(user), dataset_id)
            .await?;

        let mut dtos: Vec<DistributionDto> = Vec::new();
        let cache = self.cache.get_distribution_cache();
        for d in distributions {
            let dto: DistributionDto = d.into();
            if let Ok(id) = Urn::from_str(dto.inner.id.as_str()) {
                let score = dto.inner.dct_issued.timestamp() as f64;
                let _ = cache.set_single(&id, &dto).await;
                let _ = cache.add_to_collection(&id, score).await;
                let _ = cache
                    .add_to_relation("datasets", dataset_id, &id, score)
                    .await;
            }
            dtos.push(dto);
        }
        Ok(dtos)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_distribution_by_dataset_id_and_dct_format(
        &self,
        user: &UserInfo,
        dataset_id: &Urn,
        dct_formats: &str,
    ) -> Outcome<DistributionDto> {
        let distribution = self
            .repo
            .get_distribution_repo()
            .get_distribution_by_dataset_id_and_dct_format(
                &OwnerScope::seeing(user),
                dataset_id,
                dct_formats,
            )
            .await?
            .or_not_found(dataset_id, "distribution")?;

        let dto: DistributionDto = distribution.into();

        if let Ok(id) = Urn::from_str(dto.inner.id.as_str()) {
            let _ = self
                .cache
                .get_distribution_cache()
                .set_single(&id, &dto)
                .await;
        }

        Ok(dto)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_distribution_by_id(
        &self,
        user: &UserInfo,
        distribution_id: &Urn,
    ) -> Outcome<DistributionDto> {
        let distribution = self
            .repo
            .get_distribution_repo()
            .get_distribution_by_id(&OwnerScope::seeing(user), distribution_id)
            .await?
            .or_not_found(distribution_id, "distribution")?;

        let dto: DistributionDto = distribution.into();

        let cache = self.cache.get_distribution_cache();
        let _ = cache.set_single(distribution_id, &dto).await;
        let _ = cache
            .add_to_collection(distribution_id, dto.inner.dct_issued.timestamp() as f64)
            .await;
        Ok(dto)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn put_distribution_by_id(
        &self,
        user: &UserInfo,
        distribution_id: &Urn,
        edit_distribution_model: &EditDistributionDto,
    ) -> Outcome<DistributionDto> {
        let edit_model = edit_distribution_model.clone().into();
        let distribution = self
            .repo
            .get_distribution_repo()
            .put_distribution_by_id(
                &OwnerScope::acting(user),
                distribution_id,
                &edit_model,
            )
            .await?;

        let dto: DistributionDto = distribution.into();
        let dist_urn = Urn::from_str(dto.inner.id.as_str())?;

        let cache = self.cache.get_distribution_cache();
        let _ = cache.set_single(&dist_urn, &dto).await;
        let _ = cache
            .add_to_collection(&dist_urn, dto.inner.dct_issued.timestamp() as f64)
            .await;

        events::emit_action!(
            self.event_bus,
            &dto.inner.owner(),
            crate::EVENT_PREFIX,
            "distribution",
            "edit",
            &dto
        );
        Ok(dto)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn create_distribution(
        &self,
        user: &UserInfo,
        new_distribution_model: &NewDistributionDto,
    ) -> Outcome<DistributionDto> {
        let mut new_distribution_model = new_distribution_model.clone();
        let owner =
            Owner::for_new(user, new_distribution_model.owner.take(), new_distribution_model.visibility.clone());
        let new_model: NewDistributionModel = new_distribution_model.into_model(owner);
        let distribution = self
            .repo
            .get_distribution_repo()
            .create_distribution(&new_model)
            .await?;

        let dto: DistributionDto = distribution.into();
        let dist_urn = Urn::from_str(dto.inner.id.as_str())?;
        let score = dto.inner.dct_issued.timestamp() as f64;

        let cache = self.cache.get_distribution_cache();
        let _ = cache.set_single(&dist_urn, &dto).await;
        let _ = cache.add_to_collection(&dist_urn, score).await;

        if let Ok(dataset_id) = Urn::from_str(&dto.inner.dataset_id) {
            let _ = cache
                .add_to_relation("datasets", &dataset_id, &dist_urn, score)
                .await;
        }

        events::emit_action!(
            self.event_bus,
            &dto.inner.owner(),
            crate::EVENT_PREFIX,
            "distribution",
            "create",
            &dto
        );
        Ok(dto)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn delete_distribution_by_id(
        &self,
        user: &UserInfo,
        distribution_id: &Urn,
    ) -> Outcome<()> {
        let deleted = self
            .repo
            .get_distribution_repo()
            .delete_distribution_by_id(&OwnerScope::acting(user), distribution_id)
            .await?;

        let cache = self.cache.get_distribution_cache();
        let _ = cache.delete_single(distribution_id).await;
        let _ = cache.remove_from_collection(distribution_id).await;
        if let Ok(dataset_id) = Urn::from_str(&deleted.dataset_id) {
            let _ = cache
                .remove_from_relation("datasets", &dataset_id, distribution_id)
                .await;
        }

        events::emit_action!(
            self.event_bus,
            &deleted.owner(),
            crate::EVENT_PREFIX,
            "distribution",
            "delete",
            &events::EntityDeletedDto::new(distribution_id)
        );
        Ok(())
    }
}
