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

use crate::entities::transfer_process::TransferProcessDto;
use crate::protocols::dsp::facades::dataplane_facade::DataPlaneFacadeTrait;
use crate::protocols::dsp::orchestrator::rpc::step_trait::{RpcRequestContext, TransferRpcStep};
use crate::protocols::dsp::orchestrator::rpc::types::RpcTransferRequestMessageDto;
use crate::protocols::dsp::persistence::{CreateProcessInput, TransferPersistenceTrait};
use crate::protocols::dsp::protocol_types::{
    DataAddressDto, TransferProcessAckDto, TransferProcessMessageTrait,
    TransferProcessMessageWrapper, TransferRequestMessageDto,
};
use crate::protocols::dsp::validator::traits::validation_rpc_steps::ValidationRpcSteps;
use common::dsp_common::context_field::ContextField;
use common::http_client::HttpClient;
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::{Errors, Outcome};

// ─── RequestStep ──────────────────────────────────────────────────────────────

/// Initiates a brand-new transfer by sending a `TransferRequestMessage` to the provider.
///
/// Unlike the continuation steps, this step has no pre-existing database record.
/// It pre-allocates a local process ID, calls the dataplane pre-hook to set up
/// the consumer side, sends the request to the peer, and then persists the new
/// process record using the provider PID returned in the acknowledgement.
pub(super) struct RequestStep;

#[async_trait::async_trait]
impl TransferRpcStep for RequestStep {
    type Input = RpcTransferRequestMessageDto;
    type DspMessage = TransferRequestMessageDto;
    type Context = RpcRequestContext;

    fn url_suffix() -> &'static str {
        "request"
    }

    async fn validate(
        validator: &Arc<dyn ValidationRpcSteps>,
        input: &RpcTransferRequestMessageDto,
    ) -> Outcome<()> {
        validator.transfer_request_rpc(input).await
    }

    /// Allocates a fresh process ID and reads routing fields from the input.
    /// No database lookup is performed here
    async fn prepare_context(
        input: &RpcTransferRequestMessageDto,
        _persistence: &Arc<dyn TransferPersistenceTrait>,
    ) -> Outcome<RpcRequestContext> {
        let process_id = Urn::from_str(&format!("urn:transfer-process:{}", uuid::Uuid::new_v4()))?;
        Ok(RpcRequestContext {
            process_id,
            process: None,
            provider_address: input.provider_address.clone(),
            associated_peer: input.associated_agent_peer.clone(),
            input_data_address: input.data_address.clone(),
        })
    }

    /// For PUSH mode: initialises the consumer-side DP and returns the ingest URL
    /// that must be embedded in the outgoing `TransferRequestMessage`.
    ///
    /// For PULL mode: deferred to `post_hook` — no DP URL is needed in the message,
    /// so we wait until the provider has acknowledged the request before allocating
    /// the consumer session.
    async fn pre_hook(
        dp: &Arc<dyn DataPlaneFacadeTrait>,
        ctx: &RpcRequestContext,
    ) -> Outcome<Option<DataAddressDto>> {
        if ctx.input_data_address.is_some() {
            dp.on_transfer_request_pre(&ctx.process_id, &ctx.input_data_address)
                .await
        } else {
            Ok(None)
        }
    }

    /// Converts the RPC input into a `TransferRequestMessageDto`, generating a
    /// fresh `consumerPid`.  If the pre-hook returned an ingest URL (PUSH mode)
    /// it replaces the consumer-supplied data address.
    fn build_message(
        input: &RpcTransferRequestMessageDto,
        _ctx: &RpcRequestContext,
        data_address: Option<DataAddressDto>,
    ) -> Outcome<TransferRequestMessageDto> {
        let mut wrapper: TransferProcessMessageWrapper<TransferRequestMessageDto> =
            input.clone().into();
        // PUSH mode: override data_address with the auto-generated consumer ingest URL.
        if let Some(data_address) = data_address {
            wrapper.dto.data_address = Some(data_address);
        }
        Ok(wrapper.dto)
    }

    fn auth_peer(ctx: &RpcRequestContext) -> &str {
        &ctx.associated_peer
    }

    /// POSTs `TransferRequestMessage` to `{provider_address}/transfers/request`,
    /// then creates the local process record using the `providerPid` from the ack.
    async fn send_and_persist(
        http_client: &HttpClient,
        persistence: &Arc<dyn TransferPersistenceTrait>,
        ctx: &mut RpcRequestContext,
        payload: Arc<TransferRequestMessageDto>,
        url_suffix: &str,
    ) -> Outcome<(
        TransferProcessMessageWrapper<TransferProcessAckDto>,
        TransferProcessDto,
    )> {
        let peer_url = format!("{}/transfers/{}", ctx.provider_address, url_suffix);

        let message = TransferProcessMessageWrapper {
            context: ContextField::default(),
            _type: payload.get_message(),
            dto: payload.as_ref().clone(),
        };

        let response: TransferProcessMessageWrapper<TransferProcessAckDto> =
            http_client.post_json(peer_url.as_str(), &message).await?;

        // Provider PID is only known after the peer acknowledges, so we persist now.
        let process = persistence
            .create_process(CreateProcessInput {
                id: Some(ctx.process_id.clone()),
                protocol: "DSP",
                direction: "OUTBOUND",
                associated_agent_peer: ctx.associated_peer.clone(),
                provider_pid: Some(response.dto.provider_pid.clone()),
                provider_address: Some(ctx.provider_address.clone()),
                payload_dto: payload,
                payload_value: serde_json::to_value(&message).unwrap(),
            })
            .await?;

        ctx.process = Some(process.clone());
        Ok((response, process))
    }

    async fn post_hook(dp: &Arc<dyn DataPlaneFacadeTrait>, ctx: &Self::Context) -> Outcome<()> {
        if ctx.input_data_address.is_none() {
            let process = ctx.process.as_ref().ok_or(Errors::crazy(
                "Process should be defined at this point",
                None,
            ))?;
            dp.on_transfer_request_post(&process, &None, &None).await?;
        }
        Ok(())
    }
}
