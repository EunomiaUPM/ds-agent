/*
 *
 *  * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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

use crate::protocols::dsp::facades::FacadeTrait;
use crate::protocols::dsp::orchestrator::protocol::persistence::OrchestrationPersistenceForProtocol;
use crate::protocols::dsp::orchestrator::protocol::ProtocolOrchestratorTrait;
use crate::protocols::dsp::protocol_types::{
    CatalogMessageWrapper, CatalogRequestMessageDto, DatasetRequestMessage,
};
use crate::protocols::dsp::types::catalog_definition::Catalog;
use crate::protocols::dsp::types::dataset_definition::Dataset;
use crate::protocols::dsp::validator::traits::validation_dsp_steps::ValidationDspSteps;
use std::sync::Arc;
use ymir::errors::Outcome;

pub struct ProtocolOrchestratorService {
    facades: Arc<dyn FacadeTrait>,
    validator: Arc<dyn ValidationDspSteps>,
    persistence: Arc<OrchestrationPersistenceForProtocol>,
}

impl ProtocolOrchestratorService {
    pub fn new(
        validator: Arc<dyn ValidationDspSteps>,
        facades: Arc<dyn FacadeTrait>,
        persistence: Arc<OrchestrationPersistenceForProtocol>,
    ) -> ProtocolOrchestratorService {
        ProtocolOrchestratorService {
            validator,
            facades,
            persistence,
        }
    }
}

#[async_trait::async_trait]
impl ProtocolOrchestratorTrait for ProtocolOrchestratorService {
    async fn on_catalog_request(
        &self,
        _input: &CatalogMessageWrapper<CatalogRequestMessageDto>,
    ) -> Outcome<Catalog> {
        let catalog = self.persistence.get_catalog().await?;
        Ok(catalog)
    }

    async fn on_dataset_request(
        &self,
        input: &CatalogMessageWrapper<DatasetRequestMessage>,
    ) -> Outcome<Dataset> {
        let dataset = self.persistence.get_dataset(&input.dto.dataset).await?;
        Ok(dataset)
    }
}
