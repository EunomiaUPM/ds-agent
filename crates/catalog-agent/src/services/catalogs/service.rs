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

//! Catalog service over the repository and the cache.

use crate::cache::factory_trait::CatalogAgentCacheTrait;
use crate::data::entities::catalog::EditCatalogModel;
use crate::data::factory_trait::CatalogAgentRepoTrait;
use crate::entities::catalogs::{CatalogDto, EditCatalogDto, NewCatalogDto};
use crate::entities::filters::CatalogFilter;
use crate::services::catalogs::CatalogServiceTrait;
use common::oauth::{OwnedTrait, Owner, OwnerScope, RoleTrait, UserInfo};
use common::errors::NotFoundExt;
use common::paginated_spec::{Cursor, Page, Paginated, Sort};
use common::query::QueryFilter;
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::Outcome;

/// Catalog service that writes through to the cache and emits `catalog:` events.
pub struct CatalogService {
    repo: Arc<dyn CatalogAgentRepoTrait>,
    cache: Arc<dyn CatalogAgentCacheTrait>,
    event_bus: Option<events::EventBus>,
}

impl CatalogService {
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
impl CatalogServiceTrait for CatalogService {
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_all_catalogs(
        &self,
        user: &UserInfo,
        filters: &CatalogFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<CatalogDto>> {
        filters.validate()?;
        let page = page.clamped();

        let (catalogs, total) = self
            .repo
            .get_catalog_repo()
            .get_all_catalogs(&OwnerScope::seeing(user), filters, &page, sort)
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

        Ok(Paginated::from_page(dtos, &page, total, |d| {
            Cursor::encode_composite(&d.inner.dct_issued, &d.inner.id)
        }))
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_batch_catalogs(
        &self,
        user: &UserInfo,
        ids: &[Urn],
    ) -> Outcome<Vec<CatalogDto>> {
        let catalogs = self
            .repo
            .get_catalog_repo()
            .get_batch_catalogs(&OwnerScope::seeing(user), ids)
            .await?;

        let mut dtos: Vec<CatalogDto> = Vec::new();
        let cache = self.cache.get_catalog_cache();
        for c in catalogs {
            let dto: CatalogDto = c.into();
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
    async fn get_catalog_by_id(
        &self,
        user: &UserInfo,
        catalog_id: &Urn,
    ) -> Outcome<CatalogDto> {
        let catalog = self
            .repo
            .get_catalog_repo()
            .get_catalog_by_id(&OwnerScope::seeing(user), catalog_id)
            .await?
            .or_not_found(catalog_id, "catalog")?;

        let dto: CatalogDto = catalog.into();

        let cache = self.cache.get_catalog_cache();
        let _ = cache.set_single(catalog_id, &dto).await;
        let _ = cache
            .add_to_collection(catalog_id, dto.inner.dct_issued.timestamp() as f64)
            .await;

        Ok(dto)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_main_catalog(&self, user: &UserInfo) -> Outcome<Option<CatalogDto>> {
        let catalog = self
            .repo
            .get_catalog_repo()
            .get_main_catalog()
            .await?;
        let dto: Option<CatalogDto> = catalog.map(|c| c.into());

        if let Some(ref dto) = dto {
            let main_id = Urn::from_str(&dto.inner.id)?;
            let _ = self
                .cache
                .get_catalog_cache()
                .set_main(crate::MAIN_CACHE_KEY, &main_id, dto)
                .await;
        }

        Ok(dto)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn put_catalog_by_id(
        &self,
        user: &UserInfo,
        catalog_id: &Urn,
        edit_catalog_model: &EditCatalogDto,
    ) -> Outcome<CatalogDto> {
        let edit_model: EditCatalogModel = edit_catalog_model.clone().into();
        let catalog = self
            .repo
            .get_catalog_repo()
            .put_catalog_by_id(
                &OwnerScope::acting(user),
                catalog_id,
                &edit_model,
            )
            .await?;

        let dto: CatalogDto = catalog.into();
        let catalog_urn = Urn::from_str(dto.inner.id.as_str())?;

        let cache = self.cache.get_catalog_cache();
        let _ = cache.set_single(&catalog_urn, &dto).await;
        let _ = cache
            .add_to_collection(&catalog_urn, dto.inner.dct_issued.timestamp() as f64)
            .await;

        events::emit_action!(
            self.event_bus,
            &dto.inner.owner(),
            crate::EVENT_PREFIX,
            "catalog",
            "edit",
            &dto
        );
        Ok(dto)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn create_catalog(
        &self,
        user: &UserInfo,
        new_catalog_model: &NewCatalogDto,
    ) -> Outcome<CatalogDto> {
        let mut new_catalog_model = new_catalog_model.clone();
        let owner =
            Owner::for_new(user, new_catalog_model.owner.take(), new_catalog_model.visibility.clone());
        let new_model = new_catalog_model.into_model(owner);
        let catalog = self
            .repo
            .get_catalog_repo()
            .create_catalog(&new_model)
            .await?;

        let dto: CatalogDto = catalog.into();
        let catalog_urn = Urn::from_str(dto.inner.id.as_str())?;

        let cache = self.cache.get_catalog_cache();
        let _ = cache.set_single(&catalog_urn, &dto).await;
        let _ = cache
            .add_to_collection(&catalog_urn, dto.inner.dct_issued.timestamp() as f64)
            .await;

        events::emit_action!(
            self.event_bus,
            &dto.inner.owner(),
            crate::EVENT_PREFIX,
            "catalog",
            "create",
            &dto
        );
        Ok(dto)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn create_main_catalog(
        &self,
        user: &UserInfo,
        new_catalog_model: &NewCatalogDto,
    ) -> Outcome<CatalogDto> {
        // The main catalog is the connector's: the root's and public, served over DSP to every
        // peer with the sub-catalogs each one sees.
        user.require_root()?;
        let new_model = new_catalog_model.clone().into_model(Owner::connector());
        let catalog = self
            .repo
            .get_catalog_repo()
            .create_main_catalog(&new_model)
            .await?;
        let catalog_urn = Urn::from_str(&catalog.id)?;
        let dto: CatalogDto = catalog.into();

        let _ = self
            .cache
            .get_catalog_cache()
            .set_main(crate::MAIN_CACHE_KEY, &catalog_urn, &dto)
            .await;

        events::emit_action!(
            self.event_bus,
            &dto.inner.owner(),
            crate::EVENT_PREFIX,
            "catalog",
            "create",
            &dto
        );
        Ok(dto)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn delete_catalog_by_id(&self, user: &UserInfo, catalog_id: &Urn) -> Outcome<()> {
        let deleted = self
            .repo
            .get_catalog_repo()
            .delete_catalog_by_id(&OwnerScope::acting(user), catalog_id)
            .await?;

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

        events::emit_action!(
            self.event_bus,
            &deleted.owner(),
            crate::EVENT_PREFIX,
            "catalog",
            "delete",
            &events::EntityDeletedDto::new(catalog_id)
        );
        Ok(())
    }
}
