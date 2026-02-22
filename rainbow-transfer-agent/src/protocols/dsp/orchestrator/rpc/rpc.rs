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

use crate::entities::transfer_process::TransferProcessDto;
use crate::protocols::dsp::facades::FacadeTrait;
use crate::protocols::dsp::orchestrator::rpc::types::{
    RpcTransferCompletionMessageDto, RpcTransferMessageDto, RpcTransferProcessMessageTrait,
    RpcTransferRequestMessageDto, RpcTransferStartMessageDto, RpcTransferSuspensionMessageDto,
    RpcTransferTerminationMessageDto,
};
use crate::protocols::dsp::orchestrator::rpc::RPCOrchestratorTrait;
use crate::protocols::dsp::persistence::TransferPersistenceTrait;
use crate::protocols::dsp::protocol_types::{
    TransferCompletionMessageDto, TransferProcessAckDto, TransferProcessMessageTrait,
    TransferProcessMessageWrapper, TransferRequestMessageDto, TransferStartMessageDto,
    TransferSuspensionMessageDto, TransferTerminationMessageDto,
};
use crate::protocols::dsp::transfer_types::TransferState;
use crate::protocols::dsp::validator::traits::validation_rpc_steps::ValidationRpcSteps;
use rainbow_common::dcat_formats::DctFormats;
use rainbow_common::dsp_common::context_field::ContextField;
use rainbow_common::facades::ssi_auth_facade::MatesFacadeTrait;
use rainbow_common::http_client::HttpClient;
use rainbow_connector::InteractionConfig;
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;

#[allow(unused)]
pub struct RPCOrchestratorService {
    validator: Arc<dyn ValidationRpcSteps>,
    persistence_service: Arc<dyn TransferPersistenceTrait>,
    http_client: Arc<HttpClient>,
    facades: Arc<dyn FacadeTrait>,
    mates_facade: Arc<dyn MatesFacadeTrait>,
}

impl RPCOrchestratorService {
    pub fn new(
        validator: Arc<dyn ValidationRpcSteps>,
        persistence_service: Arc<dyn TransferPersistenceTrait>,
        http_client: Arc<HttpClient>,
        facades: Arc<dyn FacadeTrait>,
        mates_facade: Arc<dyn MatesFacadeTrait>,
    ) -> RPCOrchestratorService {
        RPCOrchestratorService {
            validator,
            persistence_service,
            http_client,
            facades,
            mates_facade,
        }
    }
}

#[async_trait::async_trait]
impl RPCOrchestratorTrait for RPCOrchestratorService {
    async fn setup_transfer_request(
        &self,
        input: &RpcTransferRequestMessageDto,
    ) -> anyhow::Result<RpcTransferMessageDto<RpcTransferRequestMessageDto>> {
        // validate
        self.validator.transfer_request_rpc(input).await?;
        // create candidate id
        let transfer_process_id =
            Urn::from_str(&format!("urn:transfer-process:{}", uuid::Uuid::new_v4()))?;
        // push or pull (if DataAddress present is push)

        // dataplane hook: init consumer DP; for PUSH returns the consumer ingest URL
        let consumer_data_address = self
            .facades
            .get_data_plane_facade()
            .await
            .on_transfer_request_pre(&transfer_process_id, &input.data_address)
            .await?;

        // get from input
        let mut request_body: TransferProcessMessageWrapper<TransferRequestMessageDto> =
            input.clone().into();
        // PUSH: override data_address with auto-generated consumer ingest URL
        if let Some(ingest_addr) = consumer_data_address {
            request_body.dto.data_address = Some(ingest_addr);
        }
        let provider_address = input.provider_address.clone();
        // create url
        let peer_url = format!("{}/transfers/request", provider_address);
        // request
        let associated_peer = input.get_associated_agent_peer().unwrap_or("".to_string());
        if let Ok(mate) = self.mates_facade.get_mate_by_id(associated_peer.clone()).await {
            if let Some(token) = mate.token {
                self.http_client.set_auth_token(token).await;
            }
        }
        let response: TransferProcessMessageWrapper<TransferProcessAckDto> =
            self.http_client.post_json(peer_url.as_str(), &request_body).await?;
        // persist
        let transfer_process = self
            .persistence_service
            .create_process_with_id(
                transfer_process_id.clone(),
                "DSP",
                "OUTBOUND",
                &associated_peer,
                Some(response.dto.provider_pid.clone()),
                Some(provider_address.clone()),
                Arc::new(request_body.clone().dto),
                serde_json::to_value(request_body.clone()).unwrap(),
            )
            .await?;

        let response = RpcTransferMessageDto {
            request: input.clone(),
            response,
            transfer_agent_model: transfer_process,
        };
        Ok(response)
    }

    async fn setup_transfer_start(
        &self,
        input: &RpcTransferStartMessageDto,
    ) -> anyhow::Result<RpcTransferMessageDto<RpcTransferStartMessageDto>> {
        // self.validator.transfer_start_rpc(input).await?;
        // get from input
        let input_transfer_id = input.consumer_pid.clone();
        // fetch current process
        let transfer_process =
            self.persistence_service.fetch_process(input_transfer_id.to_string().as_str()).await?;
        let is_restart =
            transfer_process.inner.state_attribute.clone().unwrap_or("".to_string()) != "OnRequest";
        let provider_pid = transfer_process.identifiers.get("providerPid").unwrap();
        let consumer_pid = transfer_process.identifiers.get("consumerPid").unwrap();
        // get uri
        let identifier_key = match transfer_process.inner.role.as_str() {
            "Provider" => "consumerPid",
            "Consumer" => "providerPid",
            _ => "providerPid",
        };
        let peer_url_id = transfer_process.identifiers.get(identifier_key).unwrap();
        // data plane hook: start local DP, returns proxy DataAddress for PULL Provider
        let hook_data_address = self
            .facades
            .get_data_plane_facade()
            .await
            .on_transfer_start_pre(&Urn::from_str(transfer_process.inner.id.as_str())?)
            .await?;
        // create message (hook DataAddress takes priority over manual input)
        let transfer_process_into_trait = TransferStartMessageDto {
            provider_pid: Urn::from_str(provider_pid.as_str())?,
            consumer_pid: Urn::from_str(consumer_pid.as_str())?,
            data_address: match is_restart {
                true => None,
                false => hook_data_address,
            },
        };
        // validate, send and persist
        let (response, transfer_process) = self
            .validate_and_send(
                &transfer_process,
                Arc::new(transfer_process_into_trait.clone()),
                peer_url_id,
                "start",
            )
            .await?;
        // data plane post hook (outbound sender: no incoming DataAddress to apply)
        self.facades
            .get_data_plane_facade()
            .await
            .on_transfer_start_post(&Urn::from_str(transfer_process.inner.id.as_str())?, None)
            .await?;
        // bye!
        let response = RpcTransferMessageDto {
            request: input.clone(),
            response,
            transfer_agent_model: transfer_process,
        };
        Ok(response)
    }

    async fn setup_transfer_suspension(
        &self,
        input: &RpcTransferSuspensionMessageDto,
    ) -> anyhow::Result<RpcTransferMessageDto<RpcTransferSuspensionMessageDto>> {
        self.validator.transfer_suspension_rpc(input).await?;
        // get from input
        let input_transfer_id = input.consumer_pid.clone();
        let input_code = input.code.clone();
        let input_reason = input.reason.clone();
        // fetch current process
        let transfer_process =
            self.persistence_service.fetch_process(input_transfer_id.to_string().as_str()).await?;
        let provider_pid = transfer_process.identifiers.get("providerPid").unwrap();
        let consumer_pid = transfer_process.identifiers.get("consumerPid").unwrap();
        // create message
        let transfer_process_into_trait = TransferSuspensionMessageDto {
            provider_pid: Urn::from_str(provider_pid.as_str())?,
            consumer_pid: Urn::from_str(consumer_pid.as_str())?,
            code: input_code,
            reason: input_reason,
        };
        // get uri
        let identifier_key = match transfer_process.inner.role.as_str() {
            "Provider" => "consumerPid",
            "Consumer" => "providerPid",
            _ => "providerPid",
        };
        let peer_url_id = transfer_process.identifiers.get(identifier_key).unwrap();
        // data plane hook: stop local DP before sending to peer
        self.facades
            .get_data_plane_facade()
            .await
            .on_transfer_suspension_pre(&Urn::from_str(transfer_process.inner.id.as_str())?)
            .await?;
        // validate, send and persist
        let (response, transfer_process) = self
            .validate_and_send(
                &transfer_process,
                Arc::new(transfer_process_into_trait.clone()),
                peer_url_id,
                "suspension",
            )
            .await?;
        // data plane post hook
        self.facades
            .get_data_plane_facade()
            .await
            .on_transfer_suspension_post(&Urn::from_str(transfer_process.inner.id.as_str())?)
            .await?;
        // bye!
        let response = RpcTransferMessageDto {
            request: input.clone(),
            response,
            transfer_agent_model: transfer_process,
        };
        Ok(response)
    }

    async fn setup_transfer_completion(
        &self,
        input: &RpcTransferCompletionMessageDto,
    ) -> anyhow::Result<RpcTransferMessageDto<RpcTransferCompletionMessageDto>> {
        self.validator.transfer_completion_rpc(input).await?;
        // get from input
        let input_transfer_id = input.consumer_pid.clone();
        // fetch current process
        let transfer_process =
            self.persistence_service.fetch_process(input_transfer_id.to_string().as_str()).await?;
        let provider_pid = transfer_process.identifiers.get("providerPid").unwrap();
        let consumer_pid = transfer_process.identifiers.get("consumerPid").unwrap();
        // create message
        let transfer_process_into_trait = TransferCompletionMessageDto {
            provider_pid: Urn::from_str(provider_pid.as_str())?,
            consumer_pid: Urn::from_str(consumer_pid.as_str())?,
        };
        // get uri
        let identifier_key = match transfer_process.inner.role.as_str() {
            "Provider" => "consumerPid",
            "Consumer" => "providerPid",
            _ => "providerPid",
        };
        let peer_url_id = transfer_process.identifiers.get(identifier_key).unwrap();
        // data plane hook: stop local DP before sending to peer
        self.facades
            .get_data_plane_facade()
            .await
            .on_transfer_completion_pre(&Urn::from_str(transfer_process.inner.id.as_str())?)
            .await?;
        // validate, send and persist
        let (response, transfer_process) = self
            .validate_and_send(
                &transfer_process,
                Arc::new(transfer_process_into_trait.clone()),
                peer_url_id,
                "completion",
            )
            .await?;
        // data plane post hook
        self.facades
            .get_data_plane_facade()
            .await
            .on_transfer_completion_post(&Urn::from_str(transfer_process.inner.id.as_str())?)
            .await?;
        // bye!
        let response = RpcTransferMessageDto {
            request: input.clone(),
            response,
            transfer_agent_model: transfer_process,
        };
        Ok(response)
    }

    async fn setup_transfer_termination(
        &self,
        input: &RpcTransferTerminationMessageDto,
    ) -> anyhow::Result<RpcTransferMessageDto<RpcTransferTerminationMessageDto>> {
        self.validator.transfer_termination_rpc(input).await?;
        // get from input
        let input_transfer_id = input.consumer_pid.clone();
        let input_code = input.code.clone();
        let input_reason = input.reason.clone();
        // fetch current process
        let transfer_process =
            self.persistence_service.fetch_process(input_transfer_id.to_string().as_str()).await?;
        let provider_pid = transfer_process.identifiers.get("providerPid").unwrap();
        let consumer_pid = transfer_process.identifiers.get("consumerPid").unwrap();
        // create message
        let transfer_process_into_trait = TransferTerminationMessageDto {
            provider_pid: Urn::from_str(provider_pid.as_str()).unwrap(),
            consumer_pid: Urn::from_str(consumer_pid.as_str()).unwrap(),
            code: input_code,
            reason: input_reason,
        };
        // get uri
        let identifier_key = match transfer_process.inner.role.as_str() {
            "Provider" => "consumerPid",
            "Consumer" => "providerPid",
            _ => "providerPid",
        };
        let peer_url_id = transfer_process.identifiers.get(identifier_key).unwrap();
        // data plane hook: terminate local DP before sending to peer
        self.facades
            .get_data_plane_facade()
            .await
            .on_transfer_termination_pre(&Urn::from_str(transfer_process.inner.id.as_str())?)
            .await?;
        // validate, send and persist
        let (response, transfer_process) = self
            .validate_and_send(
                &transfer_process,
                Arc::new(transfer_process_into_trait.clone()),
                peer_url_id,
                "termination",
            )
            .await?;
        // data plane post hook
        self.facades
            .get_data_plane_facade()
            .await
            .on_transfer_termination_post(&Urn::from_str(transfer_process.inner.id.as_str())?)
            .await?;
        // bye!
        let response = RpcTransferMessageDto {
            request: input.clone(),
            response,
            transfer_agent_model: transfer_process,
        };
        Ok(response)
    }
}

impl RPCOrchestratorService {
    async fn validate_and_send<T>(
        &self,
        transfer_process: &TransferProcessDto,
        payload: Arc<T>,
        peer_url_id: &str,
        url_suffix: &str,
    ) -> anyhow::Result<(
        TransferProcessMessageWrapper<TransferProcessAckDto>,
        TransferProcessDto,
    )>
    where
        T: TransferProcessMessageTrait + Clone + serde::Serialize + 'static,
    {
        // where to send
        let callback_url =
            transfer_process.inner.callback_address.clone().unwrap_or("".to_string());
        let peer_url = format!("{}/transfers/{}/{}", callback_url, peer_url_id, url_suffix);
        // create final message
        let message = TransferProcessMessageWrapper {
            context: ContextField::default(),
            _type: payload.get_message(),
            dto: payload.as_ref().clone(),
        };
        // send message to peer url
        let associated_peer = transfer_process.inner.associated_agent_peer.clone();
        if let Ok(mate) = self.mates_facade.get_mate_by_id(associated_peer).await {
            if let Some(token) = mate.token {
                self.http_client.set_auth_token(token).await;
            }
        }
        let response: TransferProcessMessageWrapper<TransferProcessAckDto> =
            self.http_client.post_json(peer_url.as_str(), &message).await?;
        // persist
        let transfer_process = self
            .persistence_service
            .update_process(
                transfer_process.inner.id.as_str(),
                payload,
                serde_json::to_value(message.clone()).unwrap(),
            )
            .await?;
        // bye!
        Ok((response, transfer_process))
    }
}
