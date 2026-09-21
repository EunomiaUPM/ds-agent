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

use std::sync::Arc;

use common::auth::access::AccessScope;
use common::auth::claims::RbacRole;
use common::batch_requests::BatchRequests;
use common::query::{Page, Sort};
use urn::Urn;
use ymir::errors::{Errors, Outcome};

use crate::cache::cache_traits::entity_cache_trait::EntityCacheTrait;
use crate::data::factory_trait::DataplaneRepoTrait;
use crate::entities::dataplane_transfers::{
    DataplaneTransferDto, DataplaneTransfersEntitiesTrait, EditDataplaneTransferDto,
    NewDataplaneTransferDto,
};
use crate::entities::filters::DataplaneTransferFilter;
use crate::services::dataplane_transfers::{
    DataplaneTransferService, DataplaneTransferServiceTrait,
};

pub struct DataplaneTransfersEntityService {
    service: Arc<DataplaneTransferService>,
}

impl DataplaneTransfersEntityService {
    pub fn new(
        data_plane_repo: Arc<dyn DataplaneRepoTrait>,
        cache: Arc<dyn EntityCacheTrait<DataplaneTransferDto>>,
    ) -> Self {
        Self {
            service: Arc::new(DataplaneTransferService::new(data_plane_repo, cache)),
        }
    }

    pub fn inner_service(&self) -> Arc<DataplaneTransferService> {
        self.service.clone()
    }
}

#[async_trait::async_trait]
impl DataplaneTransfersEntitiesTrait for DataplaneTransfersEntityService {
    async fn get_all_dataplane_transfers(&self) -> Outcome<Vec<DataplaneTransferDto>> {
        let scope = AccessScope::system();
        let paginated = self
            .service
            .get_all(
                &scope,
                &DataplaneTransferFilter::default(),
                &Page::new(u32::MAX, None),
                &Sort::default(),
            )
            .await?;
        Ok(paginated.items)
    }

    async fn get_dataplane_transfer_by_id(
        &self,
        id: &Urn,
    ) -> Outcome<Option<DataplaneTransferDto>> {
        let scope = AccessScope::system();
        match self.service.get_one(&scope, id).await {
            Ok(transfer) => Ok(Some(transfer)),
            Err(Errors::MissingResourceError { .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    async fn get_dataplane_transfer_by_process_id(
        &self,
        process_id: &Urn,
    ) -> Outcome<Option<DataplaneTransferDto>> {
        let scope = AccessScope::system();
        match self.service.get_by_process_id(&scope, process_id).await {
            Ok(transfer) => Ok(Some(transfer)),
            Err(Errors::MissingResourceError { .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    async fn get_batch_dataplane_transfers(
        &self,
        transfer_ids: &Vec<Urn>,
    ) -> Outcome<Vec<DataplaneTransferDto>> {
        let scope = AccessScope::system();
        self.service
            .batch(
                &scope,
                &BatchRequests {
                    ids: transfer_ids.clone(),
                },
            )
            .await
    }

    async fn create_dataplane_transfer(
        &self,
        new_data_plane_process: &NewDataplaneTransferDto,
    ) -> Outcome<DataplaneTransferDto> {
        let scope = AccessScope::from_role(RbacRole::Admin, &new_data_plane_process.tenant_id);
        self.service.create(&scope, new_data_plane_process).await
    }

    async fn put_dataplane_transfer_by_id(
        &self,
        id: &Urn,
        edit_dataplane_transfer: &EditDataplaneTransferDto,
    ) -> Outcome<DataplaneTransferDto> {
        let existing = self
            .get_dataplane_transfer_by_id(id)
            .await?
            .ok_or_else(|| {
                Errors::missing_resource(id.to_string(), "Dataplane transfer not found", None)
            })?;
        let scope = AccessScope::from_role(RbacRole::Admin, &existing.inner.tenant_id);
        self.service.edit(&scope, id, edit_dataplane_transfer).await
    }

    async fn delete_dataplane_transfer(&self, id: &Urn) -> Outcome<()> {
        let existing = self
            .get_dataplane_transfer_by_id(id)
            .await?
            .ok_or_else(|| {
                Errors::missing_resource(id.to_string(), "Dataplane transfer not found", None)
            })?;
        let scope = AccessScope::from_role(RbacRole::Admin, &existing.inner.tenant_id);
        self.service.delete(&scope, id).await
    }
}
