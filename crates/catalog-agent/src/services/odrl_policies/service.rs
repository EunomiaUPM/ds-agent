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

//! ODRL offer service over the repository and the cache.

use crate::cache::factory_trait::CatalogAgentCacheTrait;
use crate::data::entities::odrl_offer::NewOdrlOfferModel;
use crate::data::factory_trait::CatalogAgentRepoTrait;
use crate::entities::filters::OdrlPolicyFilter;
use crate::entities::odrl_policies::{NewOdrlPolicyDto, OdrlPolicyDto};
use crate::services::odrl_policies::OdrlPolicyServiceTrait;
use common::oauth::{OwnedTrait, Owner, OwnerScope, UserInfo};
use common::errors::NotFoundExt;
use common::paginated_spec::{Cursor, Page, Paginated, Sort};
use common::query::QueryFilter;
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::Outcome;

/// ODRL offer service that writes through to the cache and emits `catalog:` events.
pub struct OdrlPolicyService {
    repo: Arc<dyn CatalogAgentRepoTrait>,
    cache: Arc<dyn CatalogAgentCacheTrait>,
    event_bus: Option<events::EventBus>,
}

impl OdrlPolicyService {
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

    /// Publishes create and delete events on `event_bus`.
    pub fn with_event_bus(mut self, event_bus: Option<events::EventBus>) -> Self {
        self.event_bus = event_bus;
        self
    }
}

#[async_trait::async_trait]
impl OdrlPolicyServiceTrait for OdrlPolicyService {
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_all_odrl_offers(
        &self,
        user: &UserInfo,
        filters: &OdrlPolicyFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<OdrlPolicyDto>> {
        filters.validate()?;
        let page = page.clamped();

        let (odrl_policies, total) = self
            .repo
            .get_odrl_offer_repo()
            .get_all_odrl_offers(&OwnerScope::seeing(user), filters, &page, sort)
            .await?;

        let dtos: Vec<OdrlPolicyDto> = odrl_policies.into_iter().map(Into::into).collect();

        // hydration
        let cache = self.cache.get_odrl_offer_cache();
        for dto in &dtos {
            if let Ok(id) = Urn::from_str(dto.inner.id.as_str()) {
                let score = dto.inner.created_at.timestamp() as f64;
                let _ = cache.set_single(&id, dto).await;
                let _ = cache.add_to_collection(&id, score).await;
            }
        }

        Ok(Paginated::from_page(dtos, &page, total, |d| {
            Cursor::encode_composite(&d.inner.created_at, &d.inner.id)
        }))
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_batch_odrl_offers(
        &self,
        user: &UserInfo,
        ids: &[Urn],
    ) -> Outcome<Vec<OdrlPolicyDto>> {
        let odrl_policies = self
            .repo
            .get_odrl_offer_repo()
            .get_batch_odrl_offers(&OwnerScope::seeing(user), ids)
            .await?;

        let mut dtos: Vec<OdrlPolicyDto> = Vec::new();
        let cache = self.cache.get_odrl_offer_cache();
        for p in odrl_policies {
            let dto: OdrlPolicyDto = p.into();
            if let Ok(id) = Urn::from_str(dto.inner.id.as_str()) {
                let _ = cache.set_single(&id, &dto).await;
            }
            dtos.push(dto);
        }
        Ok(dtos)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_all_odrl_offers_by_entity(
        &self,
        user: &UserInfo,
        entity: &Urn,
    ) -> Outcome<Vec<OdrlPolicyDto>> {
        let odrl_policies = self
            .repo
            .get_odrl_offer_repo()
            .get_all_odrl_offers_by_entity(&OwnerScope::seeing(user), entity)
            .await?;

        let mut dtos: Vec<OdrlPolicyDto> = Vec::new();
        let cache = self.cache.get_odrl_offer_cache();
        for p in odrl_policies {
            let dto: OdrlPolicyDto = p.into();
            if let Ok(id) = Urn::from_str(dto.inner.id.as_str()) {
                let _ = cache.set_single(&id, &dto).await;
                let _ = cache.add_to_relation("target", entity, &id, 0.0).await;
            }
            dtos.push(dto);
        }
        Ok(dtos)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_odrl_offer_by_id(
        &self,
        user: &UserInfo,
        odrl_offer_id: &Urn,
    ) -> Outcome<OdrlPolicyDto> {
        let odrl_policy = self
            .repo
            .get_odrl_offer_repo()
            .get_odrl_offer_by_id(&OwnerScope::seeing(user), odrl_offer_id)
            .await?
            .or_not_found(odrl_offer_id, "odrl offer")?;

        let dto: OdrlPolicyDto = odrl_policy.into();
        let _ = self
            .cache
            .get_odrl_offer_cache()
            .set_single(odrl_offer_id, &dto)
            .await;
        Ok(dto)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn create_odrl_offer(
        &self,
        user: &UserInfo,
        new_odrl_offer_model: &NewOdrlPolicyDto,
    ) -> Outcome<OdrlPolicyDto> {
        let mut new_odrl_offer_model = new_odrl_offer_model.clone();
        let owner =
            Owner::for_new(user, new_odrl_offer_model.owner.take(), new_odrl_offer_model.visibility.clone());
        let new_model: NewOdrlOfferModel = new_odrl_offer_model.into_model(owner);
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
            let _ = cache
                .add_to_relation("target", &target_urn, &policy_id, 0.0)
                .await;
        }

        events::emit_action!(
            self.event_bus,
            &dto.inner.owner(),
            crate::EVENT_PREFIX,
            "offer",
            "create",
            &dto
        );
        Ok(dto)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn delete_odrl_offer_by_id(
        &self,
        user: &UserInfo,
        odrl_offer_id: &Urn,
    ) -> Outcome<()> {
        let deleted = self
            .repo
            .get_odrl_offer_repo()
            .delete_odrl_offer_by_id(&OwnerScope::acting(user), odrl_offer_id)
            .await?;

        let cache = self.cache.get_odrl_offer_cache();
        let _ = cache.delete_single(odrl_offer_id).await;
        let _ = cache.remove_from_collection(odrl_offer_id).await;
        if let Ok(target_urn) = Urn::from_str(&deleted.entity) {
            let _ = cache
                .remove_from_relation("target", &target_urn, odrl_offer_id)
                .await;
        }

        events::emit_action!(
            self.event_bus,
            &deleted.owner(),
            crate::EVENT_PREFIX,
            "offer",
            "delete",
            &events::EntityDeletedDto::new(odrl_offer_id)
        );
        Ok(())
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn delete_odrl_offers_by_entity(
        &self,
        user: &UserInfo,
        entity_id: &Urn,
    ) -> Outcome<()> {
        let deleted = self
            .repo
            .get_odrl_offer_repo()
            .delete_odrl_offers_by_entity(&OwnerScope::acting(user), entity_id)
            .await?;

        let cache = self.cache.get_odrl_offer_cache();
        for policy in &deleted {
            if let Ok(id) = Urn::from_str(policy.id.as_str()) {
                let _ = cache.delete_single(&id).await;
                let _ = cache.remove_from_collection(&id).await;
                let _ = cache.remove_from_relation("target", entity_id, &id).await;
                events::emit_action!(
                    self.event_bus,
                    &policy.owner(),
                    crate::EVENT_PREFIX,
                    "offer",
                    "delete",
                    &events::EntityDeletedDto::new(&id)
                );
            }
        }
        Ok(())
    }
}
