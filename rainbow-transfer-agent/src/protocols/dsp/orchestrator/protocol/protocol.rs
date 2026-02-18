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

use crate::protocols::dsp::orchestrator::protocol::ProtocolOrchestratorTrait;
use crate::protocols::dsp::persistence::TransferPersistenceTrait;
use crate::protocols::dsp::protocol_types::{
    TransferCompletionMessageDto, TransferProcessAckDto, TransferProcessMessageTrait,
    TransferProcessMessageWrapper, TransferRequestMessageDto, TransferStartMessageDto,
    TransferSuspensionMessageDto, TransferTerminationMessageDto,
};
use std::str::FromStr;

use crate::protocols::dsp::facades::FacadeTrait;
use crate::protocols::dsp::validator::traits::validation_dsp_steps::ValidationDspSteps;
use anyhow::anyhow;
use rainbow_common::dcat_formats::DctFormats;
use std::sync::Arc;
use urn::Urn;

pub struct ProtocolOrchestratorService {
    facades: Arc<dyn FacadeTrait>,
    validator: Arc<dyn ValidationDspSteps>,
    pub persistence_service: Arc<dyn TransferPersistenceTrait>,
}

impl ProtocolOrchestratorService {
    pub fn new(
        validator: Arc<dyn ValidationDspSteps>,
        persistence_service: Arc<dyn TransferPersistenceTrait>,
        facades: Arc<dyn FacadeTrait>,
    ) -> ProtocolOrchestratorService {
        ProtocolOrchestratorService { validator, persistence_service, facades }
    }
}

#[async_trait::async_trait]
impl ProtocolOrchestratorTrait for ProtocolOrchestratorService {
    async fn on_get_transfer_process(
        &self,
        id: &String,
    ) -> anyhow::Result<TransferProcessMessageWrapper<TransferProcessAckDto>> {
        let transfer_process = self.persistence_service.fetch_process(id.as_str()).await?;
        let transfer_process_dto = TransferProcessMessageWrapper::try_from(transfer_process)?;
        Ok(transfer_process_dto)
    }

    async fn on_transfer_request(
        &self,
        input: &TransferProcessMessageWrapper<TransferRequestMessageDto>,
        associated_agent_peer: &str,
    ) -> anyhow::Result<(TransferProcessMessageWrapper<TransferProcessAckDto>, bool)> {
        // transform and validate
        let input = Arc::new(input.clone());
        self.validator.on_transfer_request(&input).await?;
        dbg!("1.");

        // resolve connector instance: agreement → dataset → distribution → connector
        let agreement_id = input.dto.get_agreement_id().ok_or(anyhow!("no agreement id"))?;
        dbg!("2.");
        let dct_formats = input.dto.format.clone();
        dbg!("3.");
        let connector_instance = self
            .facades
            .get_data_service_facade()
            .await
            .resolve_connector_by_agreement_id(&agreement_id, Option::from(&dct_formats))
            .await?;
        dbg!("4.");

        // validation: PUSH connector requires a DataAddress from the consumer
        if matches!(connector_instance.interaction, rainbow_connector::InteractionConfig::Push(_))
            && input.dto.data_address.is_none()
        {
            return Err(anyhow!(
                "PUSH transfer requires a DataAddress from the consumer"
            ));
        }
        // check idempotency
        let consumer_pid = input.dto.get_consumer_pid().ok_or(anyhow!("no consumer id"))?;
        let process_result = self
            .persistence_service
            .get_transfer_process_service()
            .await?
            .get_transfer_process_by_key_id("consumerPid", &consumer_pid)
            .await;
        match process_result {
            Ok(transfer_process) => {
                let transfer_process_dto =
                    TransferProcessMessageWrapper::try_from(transfer_process)?;
                return Ok((transfer_process_dto, true));
            }
            _ => {}
        }
        dbg!("5.");

        // persist and send
        let transfer_process = self
            .persistence_service
            .create_process(
                "DSP",
                "INBOUND",
                associated_agent_peer,
                None,
                None,
                Arc::new(input.dto.clone()),
                serde_json::to_value(&input).unwrap(),
            )
            .await?;
        dbg!("6.");

        // data plane hook: register the dataplane process for the provider
        let id = Urn::from_str(transfer_process.inner.id.as_str())?;
        self.facades
            .get_data_plane_facade()
            .await
            .on_transfer_request_post(
                &id,
                &connector_instance,
                &input.dto.data_address,
            )
            .await?;

        dbg!("7.");

        // notify

        let transfer_process_dto = TransferProcessMessageWrapper::try_from(transfer_process)?;
        Ok((transfer_process_dto, false))
    }

    async fn on_transfer_start(
        &self,
        id: &String,
        input: &TransferProcessMessageWrapper<TransferStartMessageDto>,
    ) -> anyhow::Result<TransferProcessMessageWrapper<TransferProcessAckDto>> {
        self.validator.on_transfer_start(id, input).await?;
        let dpid = Urn::from_str(id.as_str())?;
        let transfer_process = self
            .persistence_service
            .get_transfer_process_service()
            .await?
            .get_transfer_process_by_key_value(&dpid)
            .await?;
        let transfer_process_id = Urn::from_str(transfer_process.inner.id.as_str())?;
        // persist and send
        let transfer_process = self
            .persistence_service
            .update_process(id, Arc::new(input.dto.clone()), serde_json::to_value(input).unwrap())
            .await?;

        // data plane hook: start local dataplane, applying incoming DataAddress as egress
        // Returns the consumer's own ingress URL to include in the ACK.
        let consumer_ingress = self.facades
            .get_data_plane_facade()
            .await
            .on_transfer_start_post(&transfer_process_id, input.dto.data_address.clone())
            .await?;
        // notify

        let mut transfer_process_dto = TransferProcessMessageWrapper::try_from(transfer_process)?;
        transfer_process_dto.dto.data_address = consumer_ingress;
        Ok(transfer_process_dto)
    }

    async fn on_transfer_suspension(
        &self,
        id: &String,
        input: &TransferProcessMessageWrapper<TransferSuspensionMessageDto>,
    ) -> anyhow::Result<TransferProcessMessageWrapper<TransferProcessAckDto>> {
        self.validator.on_transfer_suspension(id, input).await?;
        let dpid = Urn::from_str(id.as_str())?;
        let transfer_process = self
            .persistence_service
            .get_transfer_process_service()
            .await?
            .get_transfer_process_by_key_value(&dpid)
            .await?;
        let transfer_process_id = Urn::from_str(transfer_process.inner.id.as_str())?;
        // persist and send
        let transfer_process = self
            .persistence_service
            .update_process(id, Arc::new(input.dto.clone()), serde_json::to_value(input).unwrap())
            .await?;

        // data plane hook: stop local dataplane
        self.facades
            .get_data_plane_facade()
            .await
            .on_transfer_suspension_post(&transfer_process_id)
            .await?;

        // notify

        let transfer_process_dto = TransferProcessMessageWrapper::try_from(transfer_process)?;
        Ok(transfer_process_dto)
    }

    async fn on_transfer_completion(
        &self,
        id: &String,
        input: &TransferProcessMessageWrapper<TransferCompletionMessageDto>,
    ) -> anyhow::Result<TransferProcessMessageWrapper<TransferProcessAckDto>> {
        self.validator.on_transfer_completion(id, input).await?;
        let dpid = Urn::from_str(id.as_str())?;
        let transfer_process = self
            .persistence_service
            .get_transfer_process_service()
            .await?
            .get_transfer_process_by_key_value(&dpid)
            .await?;
        let transfer_process_id = Urn::from_str(transfer_process.inner.id.as_str())?;
        // persist and send
        let transfer_process = self
            .persistence_service
            .update_process(id, Arc::new(input.dto.clone()), serde_json::to_value(input).unwrap())
            .await?;

        // data plane hook: stop local dataplane
        self.facades
            .get_data_plane_facade()
            .await
            .on_transfer_completion_post(&transfer_process_id)
            .await?;

        // notify

        let transfer_process_dto = TransferProcessMessageWrapper::try_from(transfer_process)?;
        Ok(transfer_process_dto)
    }

    async fn on_transfer_termination(
        &self,
        id: &String,
        input: &TransferProcessMessageWrapper<TransferTerminationMessageDto>,
    ) -> anyhow::Result<TransferProcessMessageWrapper<TransferProcessAckDto>> {
        self.validator.on_transfer_termination(id, input).await?;
        let dpid = Urn::from_str(id.as_str())?;
        let transfer_process = self
            .persistence_service
            .get_transfer_process_service()
            .await?
            .get_transfer_process_by_key_value(&dpid)
            .await?;
        let transfer_process_id = Urn::from_str(transfer_process.inner.id.as_str())?;
        // persist and send
        let transfer_process = self
            .persistence_service
            .update_process(id, Arc::new(input.dto.clone()), serde_json::to_value(input).unwrap())
            .await?;

        // data plane hook: terminate local dataplane
        self.facades
            .get_data_plane_facade()
            .await
            .on_transfer_termination_post(&transfer_process_id)
            .await?;

        // notify

        let transfer_process_dto = TransferProcessMessageWrapper::try_from(transfer_process)?;
        Ok(transfer_process_dto)
    }
}
