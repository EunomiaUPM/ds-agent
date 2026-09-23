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
use crate::data::entities::dataservice::NewDataServiceModel;
use crate::data::factory_trait::CatalogAgentRepoTrait;
use crate::entities::data_services::{DataServiceDto, EditDataServiceDto, NewDataServiceDto};
use crate::entities::filters::DataServiceFilter;
use crate::services::data_services::DataServiceServiceTrait;
use common::auth::AccessScope;
use common::errors::NotFoundExt;
use common::paginated_spec::{Cursor, Page, Paginated, Sort};
use common::query::QueryFilter;
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::Outcome;

pub struct DataServiceService {
    repo: Arc<dyn CatalogAgentRepoTrait>,
    cache: Arc<dyn CatalogAgentCacheTrait>,
    event_bus: Option<events::EventBus>,
}

impl DataServiceService {
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
impl DataServiceServiceTrait for DataServiceService {
    async fn get_all_data_services(
        &self,
        scope: &AccessScope,
        filters: &DataServiceFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<DataServiceDto>> {
        scope.require_read()?;
        filters.validate()?;
        let mut filters = filters.clone();
        filters.tenant_id = scope.resolve_query_tenant(filters.tenant_id.as_deref())?;
        let page = page.clamped();

        let (data_services, total) = self
            .repo
            .get_dataservice_repo()
            .get_all_data_services(&filters, &page, sort)
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

        Ok(Paginated::from_page(dtos, &page, total, |d| {
            Cursor::encode_composite(&d.inner.dct_issued, &d.inner.id)
        }))
    }

    async fn get_batch_data_services(
        &self,
        scope: &AccessScope,
        ids: &[Urn],
    ) -> Outcome<Vec<DataServiceDto>> {
        scope.require_read()?;
        let data_services = self
            .repo
            .get_dataservice_repo()
            .get_batch_data_services(scope.tenant_filter().map(str::to_string), ids)
            .await?;

        let mut dtos: Vec<DataServiceDto> = Vec::new();
        let cache = self.cache.get_dataservice_cache();
        for ds in data_services {
            let dto: DataServiceDto = ds.into();
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

    async fn get_data_services_by_catalog_id(
        &self,
        scope: &AccessScope,
        catalog_id: &Urn,
    ) -> Outcome<Vec<DataServiceDto>> {
        scope.require_read()?;
        let data_services = self
            .repo
            .get_dataservice_repo()
            .get_data_services_by_catalog_id(scope.tenant_filter().map(str::to_string), catalog_id)
            .await?;

        let mut dtos: Vec<DataServiceDto> = Vec::new();
        let cache = self.cache.get_dataservice_cache();
        for ds in data_services {
            let dto: DataServiceDto = ds.into();
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

    async fn get_main_data_service(&self, scope: &AccessScope) -> Outcome<Option<DataServiceDto>> {
        scope.require_read()?;
        let data_service = self
            .repo
            .get_dataservice_repo()
            .get_main_data_service(scope.acting_tenant())
            .await?;
        let dto: Option<DataServiceDto> = data_service.map(Into::into);

        if let Some(dto) = &dto {
            if let Ok(id) = Urn::from_str(dto.inner.id.as_str()) {
                let _ = self.cache.get_dataservice_cache().set_main(&id, dto).await;
            }
        }
        Ok(dto)
    }

    async fn get_data_service_by_id(
        &self,
        scope: &AccessScope,
        data_service_id: &Urn,
    ) -> Outcome<DataServiceDto> {
        scope.require_read()?;
        let data_service = self
            .repo
            .get_dataservice_repo()
            .get_data_service_by_id(scope.tenant_filter().map(str::to_string), data_service_id)
            .await?
            .or_not_found(data_service_id, "data service")?;

        let dto: DataServiceDto = data_service.into();

        let cache = self.cache.get_dataservice_cache();
        let _ = cache.set_single(data_service_id, &dto).await;
        let _ = cache
            .add_to_collection(data_service_id, dto.inner.dct_issued.timestamp() as f64)
            .await;
        Ok(dto)
    }

    async fn put_data_service_by_id(
        &self,
        scope: &AccessScope,
        data_service_id: &Urn,
        edit_data_service_model: &EditDataServiceDto,
    ) -> Outcome<DataServiceDto> {
        scope.require_write()?;
        let edit_model = edit_data_service_model.clone().into();
        let data_service = self
            .repo
            .get_dataservice_repo()
            .put_data_service_by_id(
                scope.tenant_filter().map(str::to_string),
                data_service_id,
                &edit_model,
            )
            .await?;

        let dto: DataServiceDto = data_service.into();
        let ds_urn = Urn::from_str(dto.inner.id.as_str())?;

        let cache = self.cache.get_dataservice_cache();
        let _ = cache.set_single(&ds_urn, &dto).await;
        let _ = cache
            .add_to_collection(&ds_urn, dto.inner.dct_issued.timestamp() as f64)
            .await;

        events::emit_action!(
            self.event_bus,
            crate::EVENT_PREFIX,
            "dataservice",
            "edit",
            &dto
        );
        Ok(dto)
    }

    async fn create_data_service(
        &self,
        scope: &AccessScope,
        new_data_service_model: &NewDataServiceDto,
    ) -> Outcome<DataServiceDto> {
        let mut new_data_service_model = new_data_service_model.clone();
        let tenant_id = scope.resolve_create_tenant(new_data_service_model.tenant_id.as_deref())?;
        new_data_service_model.tenant_id = Some(tenant_id.clone());
        let new_model: NewDataServiceModel = new_data_service_model.into_model(tenant_id);
        let data_service = self
            .repo
            .get_dataservice_repo()
            .create_data_service(&new_model)
            .await?;

        let dto: DataServiceDto = data_service.into();
        let ds_urn = Urn::from_str(dto.inner.id.as_str())?;
        let score = dto.inner.dct_issued.timestamp() as f64;

        let cache = self.cache.get_dataservice_cache();
        let _ = cache.set_single(&ds_urn, &dto).await;
        let _ = cache.add_to_collection(&ds_urn, score).await;

        if let Ok(catalog_id) = Urn::from_str(&dto.inner.catalog_id) {
            let _ = cache
                .add_to_relation("catalogs", &catalog_id, &ds_urn, score)
                .await;
        }

        events::emit_action!(
            self.event_bus,
            crate::EVENT_PREFIX,
            "dataservice",
            "create",
            &dto
        );
        Ok(dto)
    }

    async fn create_main_data_service(
        &self,
        scope: &AccessScope,
        new_data_service_model: &NewDataServiceDto,
    ) -> Outcome<DataServiceDto> {
        let mut new_data_service_model = new_data_service_model.clone();
        let tenant_id = scope.resolve_create_tenant(new_data_service_model.tenant_id.as_deref())?;
        new_data_service_model.tenant_id = Some(tenant_id.clone());
        let new_model: NewDataServiceModel = new_data_service_model.into_model(tenant_id);
        let data_service = self
            .repo
            .get_dataservice_repo()
            .create_main_data_service(&new_model)
            .await?;
        let dto: DataServiceDto = data_service.into();

        if let Ok(id) = Urn::from_str(dto.inner.id.as_str()) {
            let _ = self.cache.get_dataservice_cache().set_main(&id, &dto).await;
        }

        events::emit_action!(
            self.event_bus,
            crate::EVENT_PREFIX,
            "dataservice",
            "create",
            &dto
        );
        Ok(dto)
    }

    async fn delete_data_service_by_id(
        &self,
        scope: &AccessScope,
        data_service_id: &Urn,
    ) -> Outcome<()> {
        scope.require_write()?;
        let deleted = self
            .repo
            .get_dataservice_repo()
            .delete_data_service_by_id(scope.tenant_filter().map(str::to_string), data_service_id)
            .await?;

        let cache = self.cache.get_dataservice_cache();
        let _ = cache.delete_single(data_service_id).await;
        let _ = cache.remove_from_collection(data_service_id).await;
        if let Ok(catalog_id) = Urn::from_str(&deleted.catalog_id) {
            let _ = cache
                .remove_from_relation("catalogs", &catalog_id, data_service_id)
                .await;
        }

        events::emit_action!(
            self.event_bus,
            crate::EVENT_PREFIX,
            "dataservice",
            "delete",
            &events::EntityDeletedDto::new(data_service_id)
        );
        Ok(())
    }
}
