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

//! Data service service over the repository and the cache.

use crate::cache::factory_trait::CatalogAgentCacheTrait;
use crate::data::entities::dataservice::NewDataServiceModel;
use crate::data::factory_trait::CatalogAgentRepoTrait;
use crate::entities::data_services::{DataServiceDto, EditDataServiceDto, NewDataServiceDto};
use crate::entities::filters::DataServiceFilter;
use crate::services::data_services::DataServiceServiceTrait;
use common::oauth::{OwnedTrait, Owner, OwnerScope, RoleTrait, UserInfo};
use common::errors::NotFoundExt;
use common::paginated_spec::{Cursor, Page, Paginated, Sort};
use common::query::QueryFilter;
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::Outcome;

/// Data service service that writes through to the cache and emits `catalog:` events.
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

    /// Publishes create, edit and delete events on `event_bus`.
    pub fn with_event_bus(mut self, event_bus: Option<events::EventBus>) -> Self {
        self.event_bus = event_bus;
        self
    }
}

#[async_trait::async_trait]
impl DataServiceServiceTrait for DataServiceService {
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_all_data_services(
        &self,
        user: &UserInfo,
        filters: &DataServiceFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<DataServiceDto>> {
        filters.validate()?;
        let page = page.clamped();

        let (data_services, total) = self
            .repo
            .get_dataservice_repo()
            .get_all_data_services(&OwnerScope::seeing(user), filters, &page, sort)
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

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_batch_data_services(
        &self,
        user: &UserInfo,
        ids: &[Urn],
    ) -> Outcome<Vec<DataServiceDto>> {
        let data_services = self
            .repo
            .get_dataservice_repo()
            .get_batch_data_services(&OwnerScope::seeing(user), ids)
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

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_data_services_by_catalog_id(
        &self,
        user: &UserInfo,
        catalog_id: &Urn,
    ) -> Outcome<Vec<DataServiceDto>> {
        let data_services = self
            .repo
            .get_dataservice_repo()
            .get_data_services_by_catalog_id(&OwnerScope::seeing(user), catalog_id)
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

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_main_data_service(&self, user: &UserInfo) -> Outcome<Option<DataServiceDto>> {
        let data_service = self
            .repo
            .get_dataservice_repo()
            .get_main_data_service()
            .await?;
        let dto: Option<DataServiceDto> = data_service.map(Into::into);

        if let Some(dto) = &dto {
            if let Ok(id) = Urn::from_str(dto.inner.id.as_str()) {
                let _ = self
                    .cache
                    .get_dataservice_cache()
                    .set_main(crate::MAIN_CACHE_KEY, &id, dto)
                    .await;
            }
        }
        Ok(dto)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_data_service_by_id(
        &self,
        user: &UserInfo,
        data_service_id: &Urn,
    ) -> Outcome<DataServiceDto> {
        let data_service = self
            .repo
            .get_dataservice_repo()
            .get_data_service_by_id(&OwnerScope::seeing(user), data_service_id)
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

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn put_data_service_by_id(
        &self,
        user: &UserInfo,
        data_service_id: &Urn,
        edit_data_service_model: &EditDataServiceDto,
    ) -> Outcome<DataServiceDto> {
        let edit_model = edit_data_service_model.clone().into();
        let data_service = self
            .repo
            .get_dataservice_repo()
            .put_data_service_by_id(
                &OwnerScope::acting(user),
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
            &dto.inner.owner(),
            crate::EVENT_PREFIX,
            "dataservice",
            "edit",
            &dto
        );
        Ok(dto)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn create_data_service(
        &self,
        user: &UserInfo,
        new_data_service_model: &NewDataServiceDto,
    ) -> Outcome<DataServiceDto> {
        let mut new_data_service_model = new_data_service_model.clone();
        let owner =
            Owner::for_new(user, new_data_service_model.owner.take(), new_data_service_model.visibility.clone());
        let new_model: NewDataServiceModel = new_data_service_model.into_model(owner);
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
            &dto.inner.owner(),
            crate::EVENT_PREFIX,
            "dataservice",
            "create",
            &dto
        );
        Ok(dto)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn create_main_data_service(
        &self,
        user: &UserInfo,
        new_data_service_model: &NewDataServiceDto,
    ) -> Outcome<DataServiceDto> {
        // The main data service is the connector's, like the main catalog it belongs to.
        user.require_root()?;
        let new_model: NewDataServiceModel =
            new_data_service_model.clone().into_model(Owner::connector());
        let data_service = self
            .repo
            .get_dataservice_repo()
            .create_main_data_service(&new_model)
            .await?;
        let dto: DataServiceDto = data_service.into();

        if let Ok(id) = Urn::from_str(dto.inner.id.as_str()) {
            let _ = self
                .cache
                .get_dataservice_cache()
                .set_main(crate::MAIN_CACHE_KEY, &id, &dto)
                .await;
        }

        events::emit_action!(
            self.event_bus,
            &dto.inner.owner(),
            crate::EVENT_PREFIX,
            "dataservice",
            "create",
            &dto
        );
        Ok(dto)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn delete_data_service_by_id(
        &self,
        user: &UserInfo,
        data_service_id: &Urn,
    ) -> Outcome<()> {
        let deleted = self
            .repo
            .get_dataservice_repo()
            .delete_data_service_by_id(&OwnerScope::acting(user), data_service_id)
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
            &deleted.owner(),
            crate::EVENT_PREFIX,
            "dataservice",
            "delete",
            &events::EntityDeletedDto::new(data_service_id)
        );
        Ok(())
    }
}
