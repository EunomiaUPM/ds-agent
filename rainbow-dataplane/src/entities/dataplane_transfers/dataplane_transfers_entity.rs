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

use crate::cache::cache_traits::entity_cache_trait::EntityCacheTrait;
use crate::data::entities::dataplane_field::{EditDataPlaneFieldModel, NewDataPlaneFieldModel};
use crate::data::entities::dataplane_transfer_logs::NewTransferLog;
use crate::data::entities::dataplane_transfers::{
    self as dataplane_transfers_model, EditDataplaneTransferModel, NewDataplaneTransfer,
    TransferState,
};
use crate::data::factory_trait::DataplaneRepoTrait;
use crate::data::repo_traits::dataplane_transfers_repo::DataplaneTransfersRepo;
use crate::entities::dataplane_transfers::{
    DataplaneTransferDto, DataplaneTransfersEntitiesTrait, EditDataplaneTransferDto,
    NewDataplaneTransferDto,
};
use rainbow_common::errors::{helpers::BadFormat, CommonErrors, ErrorLog};
use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tracing::error;
use urn::Urn;
use uuid::Uuid;
use ymir::errors::{Errors, Outcome, RepoIntoErrors};

pub struct DataplaneTransfersEntityService {
    pub data_plane_repo: Arc<dyn DataplaneRepoTrait>,
    pub cache: Arc<dyn EntityCacheTrait<DataplaneTransferDto>>,
}

impl DataplaneTransfersEntityService {
    pub fn new(
        data_plane_repo: Arc<dyn DataplaneRepoTrait>,
        cache: Arc<dyn EntityCacheTrait<DataplaneTransferDto>>,
    ) -> Self {
        Self { data_plane_repo, cache }
    }

    async fn enrich_process(
        &self,
        process: dataplane_transfers_model::Model,
    ) -> Outcome<DataplaneTransferDto> {
        let process_urn = Urn::from_str(&process.id)?;

        let fields_models = self
            .data_plane_repo
            .get_dataplane_fields_repo()
            .get_all_dataplane_fields_by_process_id(&process_urn)
            .await?;

        let fields = fields_models
            .into_iter()
            .map(|f| (f.key, f.value.unwrap_or_default()))
            .collect::<HashMap<String, String>>();

        let logs = self
            .data_plane_repo
            .get_dataplane_transfer_logs_repo()
            .get_transfer_logs_by_dataplane_process_id(&process_urn)
            .await?;

        Ok(DataplaneTransferDto { inner: process, fields, logs })
    }
}

#[async_trait::async_trait]
impl DataplaneTransfersEntitiesTrait for DataplaneTransfersEntityService {
    async fn get_all_dataplane_transfers(&self) -> Outcome<Vec<DataplaneTransferDto>> {
        let transfers = self
            .data_plane_repo
            .get_dataplane_transfers_repo()
            .get_all_dataplane_transfers(None, None)
            .await?;

        let mut dtos = Vec::with_capacity(transfers.len());
        for t in transfers {
            let dto = self.enrich_process(t).await?;
            dtos.push(dto);
        }

        Ok(dtos)
    }

    async fn get_dataplane_transfer_by_id(
        &self,
        id: &Urn,
    ) -> Outcome<Option<DataplaneTransferDto>> {
        // 1. Try cache
        if let Some(cached) = self.cache.get_single(id).await? {
            return Ok(Some(cached));
        }

        // 2. Try DB
        let process = self
            .data_plane_repo
            .get_dataplane_transfers_repo()
            .get_dataplane_transfers_by_id(id)
            .await?;

        if let Some(p) = process {
            let enriched = self.enrich_process(p).await?;
            // 3. Update cache
            let _ = self.cache.set_single(id, &enriched).await;
            Ok(Some(enriched))
        } else {
            Ok(None)
        }
    }

    async fn get_dataplane_transfer_by_process_id(
        &self,
        process_id: &Urn,
    ) -> Outcome<Option<DataplaneTransferDto>> {
        let process = self
            .data_plane_repo
            .get_dataplane_transfers_repo()
            .get_by_transfer_process_id(&process_id)
            .await?;

        if let Some(p) = process {
            Ok(Some(self.enrich_process(p).await?))
        } else {
            Ok(None)
        }
    }

    async fn get_batch_dataplane_transfers(
        &self,
        transfer_ids: &Vec<Urn>,
    ) -> Outcome<Vec<DataplaneTransferDto>> {
        let processes = self
            .data_plane_repo
            .get_dataplane_transfers_repo()
            .get_batch_dataplane_transfers(transfer_ids)
            .await?;

        let mut dtos = Vec::with_capacity(processes.len());
        for p in processes {
            let dto = self.enrich_process(p).await?;
            dtos.push(dto);
        }

        Ok(dtos)
    }

    async fn create_dataplane_transfer(
        &self,
        new_data_plane_process: &NewDataplaneTransferDto,
    ) -> Outcome<DataplaneTransferDto> {
        let new_model: NewDataplaneTransfer = new_data_plane_process.clone().into();
        let created_process = self
            .data_plane_repo
            .get_dataplane_transfers_repo()
            .create_dataplane_transfers(&new_model)
            .await?;

        // LOGGING: Creation
        let log = NewTransferLog {
            dataplane_process_id: created_process.id.clone(),
            previous_state: None,
            new_state: new_data_plane_process.state.clone(),
            trigger: "Creation".to_string(),
            reason: None,
        };
        if let Err(e) =
            self.data_plane_repo.get_dataplane_transfer_logs_repo().create_log(log).await
        {
            error!("Failed to create dataplane transfer log: {:?}", e);
        }

        let enriched = self.enrich_process(created_process).await?;

        // Cache update
        let urn_str = enriched.inner.id.clone();
        if let Ok(urn) = Urn::from_str(&urn_str) {
            let _ = self.cache.set_single(&urn, &enriched).await;
        }

        Ok(enriched)
    }

    async fn put_dataplane_transfer_by_id(
        &self,
        id: &Urn,
        edit_dataplane_transfer: &EditDataplaneTransferDto,
    ) -> Outcome<DataplaneTransferDto> {
        // Fetch current state for logging (use cache/DB via self)
        let current_dto = self.get_dataplane_transfer_by_id(id).await?;

        let edit_model = EditDataplaneTransferModel {
            state: edit_dataplane_transfer.state.clone(),
            connector_instance_id: edit_dataplane_transfer.connector_instance_id.clone(),
            ingress_config: edit_dataplane_transfer.ingress_config.clone(),
            egress_config: edit_dataplane_transfer.egress_config.clone(),
            flow_control: edit_dataplane_transfer.flow_control.clone(),
        };

        if let Some(fields) = edit_dataplane_transfer.fields.as_ref() {
            let fields_repo = self.data_plane_repo.get_dataplane_fields_repo();
            fields_repo.delete_all_dataplane_fields_by_process_id(id).await?;

            for (key, value) in fields {
                let new_field =
                    NewDataPlaneFieldModel { key: key.clone(), value: Some(value.clone()) };
                fields_repo.create_dataplane_field(id, &new_field).await?;
            }
        }

        let updated_process = self
            .data_plane_repo
            .get_dataplane_transfers_repo()
            .put_dataplane_transfers(id, &edit_model)
            .await?;

        // LOGGING: Update (if state changed)
        if let Some(new_state) = &edit_dataplane_transfer.state {
            let previous_state = current_dto.as_ref().map(|d| d.inner.state.clone());
            // Only log if state actually changed AND previous state is known (or distinct from new)
            // Handle case where previous is None (should not happen for existing process)
            let changed = match &previous_state {
                Some(prev) => prev != new_state,
                None => true, // If we couldn't fetch previous, assume changed to be safe? Or valid transition from nothing?
            };

            if changed {
                let log = NewTransferLog {
                    dataplane_process_id: updated_process.id.clone(),
                    previous_state,
                    new_state: new_state.clone(),
                    trigger: "Update".to_string(),
                    reason: None,
                };
                if let Err(e) =
                    self.data_plane_repo.get_dataplane_transfer_logs_repo().create_log(log).await
                {
                    error!("Failed to create dataplane transfer log: {:?}", e);
                }
            }
        }

        let enriched = self.enrich_process(updated_process).await?;

        // Update cache
        let _ = self.cache.set_single(id, &enriched).await;

        Ok(enriched)
    }

    async fn delete_dataplane_transfer(&self, id: &Urn) -> Outcome<()> {
        self.data_plane_repo.get_dataplane_transfers_repo().delete_dataplane_transfers(id).await?;

        // Remove from cache
        let _ = self.cache.delete_single(id).await;

        Ok(())
    }
}
