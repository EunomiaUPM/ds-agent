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
use crate::entities::odrl_policies::{NewOdrlPolicyDto, OdrlPolicyDto, OdrlPolicyEntityTrait};
use common::errors::{CommonErrors, ErrorLog};
use std::str::FromStr;
use std::sync::Arc;
use tracing::error;
use urn::Urn;
use ymir::errors::Outcome;

pub struct OdrlPolicyEntities {
    repo: Arc<dyn CatalogAgentRepoTrait>,
    cache: Arc<dyn CatalogAgentCacheTrait>,
}

impl OdrlPolicyEntities {
    pub fn new(
        repo: Arc<dyn CatalogAgentRepoTrait>,
        cache: Arc<dyn CatalogAgentCacheTrait>,
    ) -> Self {
        Self { repo, cache }
    }
}

#[async_trait::async_trait]
impl OdrlPolicyEntityTrait for OdrlPolicyEntities {
    async fn get_all_odrl_offers(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Outcome<Vec<OdrlPolicyDto>> {
        // 1. Cache hit
        if let Ok(dtos) = self.cache.get_odrl_offer_cache().get_collection(limit, page).await {
            if !dtos.is_empty() {
                return Ok(dtos);
            }
        }

        // 2. Database fetch
        let odrl_policies = self
            .repo
            .get_odrl_offer_repo()
            .get_all_odrl_offers(limit, page)
            .await?;

        let dtos: Vec<OdrlPolicyDto> = odrl_policies.into_iter().map(Into::into).collect();

        // 3. Safe hydration
        let cache = self.cache.get_odrl_offer_cache();
        for dto in &dtos {
            if let Ok(id) = Urn::from_str(dto.inner.id.as_str()) {
                // We use a default score or timestamp if available
                let _ = cache.set_single(&id, dto).await;
                let _ = cache.add_to_collection(&id, 0.0).await;
            }
        }
        Ok(dtos)
    }

    async fn get_batch_odrl_offers(&self, ids: &Vec<Urn>) -> Outcome<Vec<OdrlPolicyDto>> {
        //  cache
        if let Ok(dtos) = self.cache.get_odrl_offer_cache().get_batch(ids).await {
            if !dtos.is_empty() {
                return Ok(dtos);
            }
        }

        // db
        let odrl_policies = self
            .repo
            .get_odrl_offer_repo()
            .get_batch_odrl_offers(ids)
            .await?;

        let dtos: Vec<OdrlPolicyDto> = odrl_policies.into_iter().map(Into::into).collect();

        // hydration
        let cache = self.cache.get_odrl_offer_cache();
        for dto in &dtos {
            if let Ok(id) = Urn::from_str(dto.inner.id.as_str()) {
                let _ = cache.set_single(&id, dto).await;
            }
        }

        Ok(dtos)
    }

    async fn get_all_odrl_offers_by_entity(
        &self,
        entity: &Urn,
    ) -> Outcome<Vec<OdrlPolicyDto>> {
        // cache
        if let Ok(dtos) =
            self.cache.get_odrl_offer_cache().get_by_relation("target", entity, None, None).await
        {
            if !dtos.is_empty() {
                return Ok(dtos);
            }
        }

        // db
        let odrl_policies = self
            .repo
            .get_odrl_offer_repo()
            .get_all_odrl_offers_by_entity(entity)
            .await?;

        let dtos: Vec<OdrlPolicyDto> = odrl_policies.into_iter().map(Into::into).collect();

        // hydration
        let cache = self.cache.get_odrl_offer_cache();
        for dto in &dtos {
            if let Ok(id) = Urn::from_str(dto.inner.id.as_str()) {
                let _ = cache.set_single(&id, dto).await;
                // Index under the specific entity URN
                let _ = cache.add_to_relation("target", entity, &id, 0.0).await;
            }
        }
        Ok(dtos)
    }

    async fn get_odrl_offer_by_id(
        &self,
        odrl_offer_id: &Urn,
    ) -> Outcome<Option<OdrlPolicyDto>> {
        // cache
        if let Ok(Some(dto)) = self.cache.get_odrl_offer_cache().get_single(odrl_offer_id).await {
            return Ok(Some(dto));
        }

        // db
        let odrl_policy = self
            .repo
            .get_odrl_offer_repo()
            .get_odrl_offer_by_id(odrl_offer_id)
            .await?;

        let dto: Option<OdrlPolicyDto> = odrl_policy.map(Into::into);

        // hydration
        if let Some(dto) = &dto {
            let _ = self.cache.get_odrl_offer_cache().set_single(odrl_offer_id, dto).await;
        }
        Ok(dto)
    }

    async fn create_odrl_offer(
        &self,
        new_odrl_offer_model: &NewOdrlPolicyDto,
    ) -> Outcome<OdrlPolicyDto> {
        // db
        let new_model = new_odrl_offer_model.clone().into();
        let odrl_policy = self
            .repo
            .get_odrl_offer_repo()
            .create_odrl_offer(&new_model)
            .await?;

        let dto: OdrlPolicyDto = odrl_policy.into();
        let policy_id = Urn::from_str(dto.inner.id.as_str())?;

        // hydration
        let cache = self.cache.get_odrl_offer_cache();
        let _ = cache.set_single(&policy_id, &dto).await;
        let _ = cache.add_to_collection(&policy_id, 0.0).await;

        // lookup
        if let Ok(target_urn) = Urn::from_str(&dto.inner.entity) {
            let _ = cache.add_to_relation("target", &target_urn, &policy_id, 0.0).await;
        }

        Ok(dto)
    }

    async fn delete_odrl_offer_by_id(&self, odrl_offer_id: &Urn) -> Outcome<()> {
        let current = self.get_odrl_offer_by_id(odrl_offer_id).await?;

        // db
        self.repo
            .get_odrl_offer_repo()
            .delete_odrl_offer_by_id(odrl_offer_id)
            .await?;

        // cache invalidation
        let cache = self.cache.get_odrl_offer_cache();
        let _ = cache.delete_single(odrl_offer_id).await;
        let _ = cache.remove_from_collection(odrl_offer_id).await;

        // lookup invalidation
        if let Some(dto) = current {
            if let Ok(target_urn) = Urn::from_str(&dto.inner.entity) {
                let _ = cache.remove_from_relation("target", &target_urn, odrl_offer_id).await;
            }
        }

        Ok(())
    }

    async fn delete_odrl_offers_by_entity(&self, entity_id: &Urn) -> Outcome<()> {
        // db
        let current_policies = self.get_all_odrl_offers_by_entity(entity_id).await?;

        // db
        self.repo
            .get_odrl_offer_repo()
            .delete_odrl_offers_by_entity(entity_id)
            .await?;

        // invalidation
        let cache = self.cache.get_odrl_offer_cache();
        for policy in current_policies {
            if let Ok(id) = Urn::from_str(policy.inner.id.as_str()) {
                let _ = cache.delete_single(&id).await;
                let _ = cache.remove_from_collection(&id).await;
            }
        }

        // lookup invalidation
        let _ = cache.remove_from_relation("target", entity_id, &Urn::from_str("nil:nil")?).await; // dummy trigger

        Ok(())
    }
}
