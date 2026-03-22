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

use crate::protocols::dsp::orchestrator::rpc::types::{
    RpcCatalogRequestMessageDto, RpcDatasetRequestMessageDto,
};
use crate::protocols::dsp::protocol_types::CatalogMessageWrapper;
use crate::protocols::dsp::validator::traits::validate_payload::ValidatePayload;
use crate::protocols::dsp::validator::traits::validation_dsp_steps::ValidationDspSteps;
use crate::protocols::dsp::validator::traits::validation_helpers::ValidationHelpers;
use crate::protocols::dsp::validator::traits::validation_rpc_steps::ValidationRpcSteps;
use std::sync::Arc;
use ymir::errors::Outcome;

pub struct ValidationRpcStepsService {
    payload_validator: Arc<dyn ValidatePayload>,
    helpers: Arc<dyn ValidationHelpers>,
}
impl ValidationRpcStepsService {
    pub fn new(
        payload_validator: Arc<dyn ValidatePayload>,
        helpers: Arc<dyn ValidationHelpers>,
    ) -> Self {
        Self {
            payload_validator,
            helpers,
        }
    }
}

#[async_trait::async_trait]
impl ValidationRpcSteps for ValidationRpcStepsService {
    async fn on_catalog_request(&self, input: &RpcCatalogRequestMessageDto) -> Outcome<()> {
        Ok(())
    }

    async fn on_dataset_request(&self, input: &RpcDatasetRequestMessageDto) -> Outcome<()> {
        Ok(())
    }
}
