/*
 *
 *  * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *  *
 *  * This program is free software: you can redistribute it and/or modify
 *  * it under the terms of the GNU General Public License as published by
 *  * the Free Software Foundation, either version 3 of the License, or
 *  * (at your option) any later version.
 *  *
 *  * This program is distributed in the hope that it will be useful,
 *  * but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  * GNU General Public License for more details.
 *  *
 *  * You should have received a copy of the GNU General Public License
 *  * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use crate::data::entities::transfer_process::{
    self as transfer_process_model, EditTransferProcessModel, NewTransferProcessModel,
};
use crate::data::entities::transfer_process_identifier::{
    EditTransferIdentifierModel, NewTransferIdentifierModel,
};
use crate::data::factory_trait::TransferAgentRepoTrait;
use crate::entities::transfer_process::{
    EditTransferProcessDto, NewTransferProcessDto, TransferAgentProcessesTrait, TransferProcessDto,
};
use common::utils::get_urn;
use log::error;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::{Errors, Outcome};

pub struct TransferAgentProcessesService {
    pub transfer_repo: Arc<dyn TransferAgentRepoTrait>,
}

impl TransferAgentProcessesService {
    pub fn new(transfer_repo: Arc<dyn TransferAgentRepoTrait>) -> Self {
        Self { transfer_repo }
    }

    async fn enrich_process(
        &self,
        process: transfer_process_model::Model,
    ) -> Outcome<TransferProcessDto> {
        let process_urn = Urn::from_str(&process.id)?;

        let messages = self
            .transfer_repo
            .get_transfer_message_repo()
            .get_messages_by_process_id(&process_urn)
            .await?;

        let identifiers_models = self
            .transfer_repo
            .get_transfer_process_identifiers_repo()
            .get_identifiers_by_process_id(&process_urn)
            .await
            .map_err(|e| {
                error!("{}", e);
                e
            })?;

        let ids_map: HashMap<String, String> = identifiers_models
            .into_iter()
            .filter_map(|id_model| match id_model.id_value {
                Some(val) => Some((id_model.id_key, val)),
                None => None,
            })
            .collect();

        Ok(TransferProcessDto {
            inner: process,
            identifiers: ids_map,
            messages,
        })
    }
}

#[async_trait::async_trait]
impl TransferAgentProcessesTrait for TransferAgentProcessesService {
    async fn get_all_transfer_processes(
        &self,
        limit: Option<u64>,
        page: Option<u64>,
    ) -> Outcome<Vec<TransferProcessDto>> {
        let processes = self
            .transfer_repo
            .get_transfer_process_repo()
            .get_all_transfer_processes(limit, page)
            .await?;

        let mut dtos = Vec::with_capacity(processes.len());
        for p in processes {
            let dto = self.enrich_process(p).await?;
            dtos.push(dto);
        }

        Ok(dtos)
    }

    async fn get_batch_transfer_processes(
        &self,
        ids: &Vec<Urn>,
    ) -> Outcome<Vec<TransferProcessDto>> {
        let processes = self
            .transfer_repo
            .get_transfer_process_repo()
            .get_batch_transfer_processes(ids)
            .await?;

        let mut dtos = Vec::with_capacity(processes.len());
        for p in processes {
            let dto = self.enrich_process(p).await?;
            dtos.push(dto);
        }

        Ok(dtos)
    }

    async fn get_transfer_process_by_id(&self, id: &Urn) -> Outcome<TransferProcessDto> {
        let process = self
            .transfer_repo
            .get_transfer_process_repo()
            .get_transfer_process_by_id(id)
            .await?
            .ok_or_else(|| Errors::crazy("Transfer Process not found", None))?;

        self.enrich_process(process).await
    }

    async fn get_transfer_process_by_key_id(
        &self,
        key_id: &str,
        id: &Urn,
    ) -> Outcome<TransferProcessDto> {
        let process = self
            .transfer_repo
            .get_transfer_process_repo()
            .get_transfer_process_by_key_id(key_id, id)
            .await?
            .ok_or_else(|| Errors::crazy("Transfer Process not found by key identifier", None))?;

        self.enrich_process(process).await
    }

    async fn get_transfer_process_by_key_value(&self, id: &Urn) -> Outcome<TransferProcessDto> {
        let process = self
            .transfer_repo
            .get_transfer_process_repo()
            .get_transfer_process_by_key_value(id)
            .await?
            .ok_or_else(|| Errors::crazy("Transfer Process not found by key identifier", None))?;

        self.enrich_process(process).await
    }

    async fn create_transfer_process(
        &self,
        new_model_dto: &NewTransferProcessDto,
    ) -> Outcome<TransferProcessDto> {
        let new_process_model: NewTransferProcessModel = new_model_dto.clone().into();

        let created_process = self
            .transfer_repo
            .get_transfer_process_repo()
            .create_transfer_process(&new_process_model)
            .await?;

        let process_urn = Urn::from_str(&created_process.id)?;

        if let Some(identifiers) = &new_model_dto.identifiers {
            for (key, urn_value) in identifiers {
                let new_ident_model = NewTransferIdentifierModel {
                    id: None,
                    transfer_agent_process_id: process_urn.clone(),
                    id_key: key.clone(),
                    id_value: Some(urn_value.to_string()),
                };

                self.transfer_repo
                    .get_transfer_process_identifiers_repo()
                    .create_identifier(&new_ident_model)
                    .await?;
            }
        }

        self.enrich_process(created_process).await
    }

    async fn put_transfer_process(
        &self,
        id: &Urn,
        edit_model_dto: &EditTransferProcessDto,
    ) -> Outcome<TransferProcessDto> {
        let edit_model: EditTransferProcessModel = edit_model_dto.clone().into();

        let updated_process = self
            .transfer_repo
            .get_transfer_process_repo()
            .put_transfer_process(id, &edit_model)
            .await?;

        let process_urn = Urn::from_str(&updated_process.id)?;

        if let Some(identifiers) = &edit_model_dto.identifiers {
            for (key, urn_value) in identifiers {
                let new_ident_model = EditTransferIdentifierModel {
                    id_key: Option::from(key.clone()),
                    id_value: Some(urn_value.to_string()),
                };

                let identifier_model = self
                    .transfer_repo
                    .get_transfer_process_identifiers_repo()
                    .get_identifier_by_key(&process_urn, key)
                    .await?;

                if identifier_model.is_none() {
                    self.transfer_repo
                        .get_transfer_process_identifiers_repo()
                        .create_identifier(&NewTransferIdentifierModel {
                            id: Some(get_urn(None)),
                            transfer_agent_process_id: process_urn.clone(),
                            id_key: key.clone(),
                            id_value: Some(urn_value.to_string()),
                        })
                        .await?;
                } else {
                    let id_urn_ident = Urn::from_str(identifier_model.unwrap().id.as_str())?;

                    self.transfer_repo
                        .get_transfer_process_identifiers_repo()
                        .put_identifier(&id_urn_ident, &new_ident_model)
                        .await?;
                }
            }
        }

        self.enrich_process(updated_process).await
    }

    async fn delete_transfer_process(&self, id: &Urn) -> Outcome<()> {
        self.transfer_repo
            .get_transfer_process_repo()
            .delete_transfer_process(id)
            .await?;
        Ok(())
    }
}
