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

use crate::protocols::dsp::context::DspTransferContext;
use crate::protocols::dsp::facades::dataplane_facade::DataPlaneFacadeTrait;
use crate::protocols::dsp::orchestrator::rpc::step_trait::TransferRpcStep;
use crate::protocols::dsp::orchestrator::rpc::types::RpcTransferRequestMessageDto;
use crate::protocols::dsp::persistence::TransferPersistenceTrait;
use crate::protocols::dsp::protocol_types::{
    TransferProcessAckDto, TransferProcessMessageTrait, TransferProcessMessageWrapper,
    TransferRequestMessageDto,
};
use crate::protocols::dsp::validator::traits::validation_rpc_steps::ValidationRpcSteps;
use common::dsp_common::context_field::ContextField;
use common::http_client::HttpClient;
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::Outcome;

pub(super) struct RequestStep;

#[async_trait::async_trait]
impl TransferRpcStep for RequestStep {
    type Input = RpcTransferRequestMessageDto;
    type DspMessage = TransferRequestMessageDto;

    fn url_suffix() -> &'static str {
        "request"
    }

    async fn validate(
        validator: &Arc<dyn ValidationRpcSteps>,
        input: &RpcTransferRequestMessageDto,
    ) -> Outcome<()> {
        validator.transfer_request_rpc(input).await
    }

    async fn prepare_context(
        ctx: &mut DspTransferContext,
        _input: &RpcTransferRequestMessageDto,
        _persistence: &Arc<dyn TransferPersistenceTrait>,
    ) -> Outcome<()> {
        ctx.local_process_id = Some(Urn::from_str(&format!(
            "urn:transfer-process:{}",
            uuid::Uuid::new_v4()
        ))?);
        Ok(())
    }

    async fn pre_hook(
        dp: &Arc<dyn DataPlaneFacadeTrait>,
        ctx: &mut DspTransferContext,
    ) -> Outcome<()> {
        if ctx.input_data_address.is_some() {
            let addr = dp.on_transfer_request_pre(ctx).await?;
            ctx.resolved_data_address = addr;
        }
        Ok(())
    }

    fn build_message(
        ctx: &DspTransferContext,
        input: &RpcTransferRequestMessageDto,
    ) -> Outcome<TransferRequestMessageDto> {
        let mut wrapper: TransferProcessMessageWrapper<TransferRequestMessageDto> =
            input.clone().into();
        if let Some(data_address) = ctx.resolved_data_address.clone() {
            wrapper.dto.data_address = Some(data_address);
        }
        Ok(wrapper.dto)
    }

    async fn send_and_persist(
        http_client: &HttpClient,
        persistence: &Arc<dyn TransferPersistenceTrait>,
        ctx: &mut DspTransferContext,
        payload: Arc<TransferRequestMessageDto>,
    ) -> Outcome<TransferProcessMessageWrapper<TransferProcessAckDto>> {
        let peer_url = format!(
            "{}/transfers/{}",
            ctx.provider_address.as_deref().unwrap_or_default(),
            Self::url_suffix()
        );

        let message = TransferProcessMessageWrapper {
            context: ContextField::default(),
            _type: payload.get_message(),
            dto: payload.as_ref().clone(),
        };

        let response: TransferProcessMessageWrapper<TransferProcessAckDto> =
            http_client.post_json(&peer_url, &message).await?;

        // Provider PID is only known after the peer acknowledges.
        ctx.provider_pid = Some(response.dto.provider_pid.clone());

        let payload_dyn: Arc<dyn TransferProcessMessageTrait> = payload;
        let process = persistence
            .create_process(ctx, payload_dyn, serde_json::to_value(&message).unwrap())
            .await?;

        ctx.process = Some(process);
        Ok(response)
    }

    async fn post_hook(
        dp: &Arc<dyn DataPlaneFacadeTrait>,
        ctx: &mut DspTransferContext,
    ) -> Outcome<()> {
        if ctx.input_data_address.is_none() {
            dp.on_transfer_request_post(ctx).await?;
        }
        Ok(())
    }
}
