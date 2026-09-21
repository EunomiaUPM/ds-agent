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
use crate::data::entities::catalog::EditCatalogModel;
use crate::data::factory_trait::CatalogAgentRepoTrait;
use crate::entities::catalogs::{CatalogDto, CatalogEntityTrait, EditCatalogDto, NewCatalogDto};
use crate::entities::filters::CatalogFilter;
use common::auth::AccessScope;
use common::errors::NotFoundExt;
use common::paginated_spec::{Cursor, Page, Paginated, Sort};
use common::query::QueryFilter;
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::Outcome;

pub struct CatalogEntities {
    repo: Arc<dyn CatalogAgentRepoTrait>,
    cache: Arc<dyn CatalogAgentCacheTrait>,
    event_bus: Option<events::EventBus>,
}

impl CatalogEntities {
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
impl CatalogEntityTrait for CatalogEntities {
    async fn get_all_catalogs(
        &self,
        scope: &AccessScope,
        filters: &CatalogFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<CatalogDto>> {
        scope.require_read()?;
        filters.validate()?;
        let mut filters = filters.clone();
        filters.tenant_id = scope.resolve_query_tenant(filters.tenant_id.as_deref())?;
        let page = page.clamped();

        let (catalogs, total) = self
            .repo
            .get_catalog_repo()
            .get_all_catalogs(&filters, &page, sort)
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

    async fn get_batch_catalogs(
        &self,
        scope: &AccessScope,
        ids: &[Urn],
    ) -> Outcome<Vec<CatalogDto>> {
        scope.require_read()?;
        let catalogs = self
            .repo
            .get_catalog_repo()
            .get_batch_catalogs(scope.acting_tenant(), ids)
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

    async fn get_catalog_by_id(
        &self,
        scope: &AccessScope,
        catalog_id: &Urn,
    ) -> Outcome<CatalogDto> {
        scope.require_read()?;
        let catalog = self
            .repo
            .get_catalog_repo()
            .get_catalog_by_id(scope.acting_tenant(), catalog_id)
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

    async fn get_main_catalog(&self, scope: &AccessScope) -> Outcome<Option<CatalogDto>> {
        scope.require_read()?;
        let catalog = self
            .repo
            .get_catalog_repo()
            .get_main_catalog(scope.acting_tenant())
            .await?;
        let dto: Option<CatalogDto> = catalog.map(|c| c.into());

        if let Some(ref dto) = dto {
            let main_id = Urn::from_str(&dto.inner.id)?;
            let _ = self.cache.get_catalog_cache().set_main(&main_id, dto).await;
        }

        Ok(dto)
    }

    async fn put_catalog_by_id(
        &self,
        scope: &AccessScope,
        catalog_id: &Urn,
        edit_catalog_model: &EditCatalogDto,
    ) -> Outcome<CatalogDto> {
        scope.require_write()?;
        let edit_model: EditCatalogModel = edit_catalog_model.clone().into();
        let catalog = self
            .repo
            .get_catalog_repo()
            .put_catalog_by_id(scope.acting_tenant(), catalog_id, &edit_model)
            .await?;

        let dto: CatalogDto = catalog.into();
        let catalog_urn = Urn::from_str(dto.inner.id.as_str())?;

        let cache = self.cache.get_catalog_cache();
        let _ = cache.set_single(&catalog_urn, &dto).await;
        let _ = cache
            .add_to_collection(&catalog_urn, dto.inner.dct_issued.timestamp() as f64)
            .await;

        events::emit_action!(self.event_bus, crate::EVENT_PREFIX, "catalog", "edit", &dto);
        Ok(dto)
    }

    async fn create_catalog(
        &self,
        scope: &AccessScope,
        new_catalog_model: &NewCatalogDto,
    ) -> Outcome<CatalogDto> {
        let mut new_catalog_model = new_catalog_model.clone();
        let tenant_id = scope.resolve_create_tenant(new_catalog_model.tenant_id.as_deref())?;
        new_catalog_model.tenant_id = Some(tenant_id.clone());
        let new_model = new_catalog_model.into_model(tenant_id);
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
            crate::EVENT_PREFIX,
            "catalog",
            "create",
            &dto
        );
        Ok(dto)
    }

    async fn create_main_catalog(
        &self,
        scope: &AccessScope,
        new_catalog_model: &NewCatalogDto,
    ) -> Outcome<CatalogDto> {
        let mut new_catalog_model = new_catalog_model.clone();
        let tenant_id = scope.resolve_create_tenant(new_catalog_model.tenant_id.as_deref())?;
        new_catalog_model.tenant_id = Some(tenant_id.clone());
        let new_model = new_catalog_model.into_model(tenant_id);
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
            .set_main(&catalog_urn, &dto)
            .await;

        events::emit_action!(
            self.event_bus,
            crate::EVENT_PREFIX,
            "catalog",
            "create",
            &dto
        );
        Ok(dto)
    }

    async fn delete_catalog_by_id(&self, scope: &AccessScope, catalog_id: &Urn) -> Outcome<()> {
        scope.require_write()?;
        self.repo
            .get_catalog_repo()
            .delete_catalog_by_id(scope.acting_tenant(), catalog_id)
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
            crate::EVENT_PREFIX,
            "catalog",
            "delete",
            &events::EntityDeletedDto::new(catalog_id)
        );
        Ok(())
    }
}
