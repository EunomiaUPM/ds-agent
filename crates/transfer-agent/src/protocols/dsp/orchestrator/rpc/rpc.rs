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
use crate::protocols::dsp::context::DspTransferContext;
use crate::protocols::dsp::facades::FacadeTrait;
use crate::protocols::dsp::orchestrator::rpc::step_completion::CompletionStep;
use crate::protocols::dsp::orchestrator::rpc::step_request::RequestStep;
use crate::protocols::dsp::orchestrator::rpc::step_start::StartStep;
use crate::protocols::dsp::orchestrator::rpc::step_suspension::SuspensionStep;
use crate::protocols::dsp::orchestrator::rpc::step_termination::TerminationStep;
use crate::protocols::dsp::orchestrator::rpc::step_trait::TransferRpcStep;
use crate::protocols::dsp::orchestrator::rpc::types::{
    RpcTransferCompletionMessageDto, RpcTransferMessageDto, RpcTransferRequestMessageDto,
    RpcTransferStartMessageDto, RpcTransferSuspensionMessageDto, RpcTransferTerminationMessageDto,
};
use crate::protocols::dsp::orchestrator::rpc::RPCOrchestratorTrait;
use crate::protocols::dsp::persistence::TransferPersistenceTrait;
use crate::protocols::dsp::protocol_types::{TransferProcessAckDto, TransferProcessMessageWrapper};
use crate::protocols::dsp::validator::traits::validation_rpc_steps::ValidationRpcSteps;
use common::facades::ssi_auth_facade::MatesFacadeTrait;
use common::http_client::HttpClient;
use std::sync::Arc;
use ymir::errors::{Errors, Outcome};

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

    async fn run_lifecycle<S: TransferRpcStep>(
        &self,
        ctx: &mut DspTransferContext,
        input: &S::Input,
    ) -> Outcome<(
        TransferProcessMessageWrapper<TransferProcessAckDto>,
        TransferProcessDto,
    )> {
        S::validate(&self.validator, input).await?;
        S::prepare_context(ctx, input, &self.persistence_service).await?;
        let dp = self.facades.get_data_plane_facade().await;
        S::pre_hook(&dp, ctx).await?;
        let message = S::build_message(ctx, input)?;
        S::apply_auth_token(
            &self.mates_facade,
            &self.http_client,
            &ctx.associated_peer_id,
        )
        .await;
        let response = S::send_and_persist(
            &self.http_client,
            &self.persistence_service,
            ctx,
            Arc::new(message),
        )
        .await?;
        S::post_hook(&dp, ctx).await?;
        let process = ctx
            .process
            .clone()
            .ok_or_else(|| Errors::crazy("process missing after send_and_persist", None))?;
        Ok((response, process))
    }
}

#[async_trait::async_trait]
impl RPCOrchestratorTrait for RPCOrchestratorService {
    async fn setup_transfer_request(
        &self,
        ctx: DspTransferContext,
        input: &RpcTransferRequestMessageDto,
    ) -> Outcome<RpcTransferMessageDto<RpcTransferRequestMessageDto>> {
        let mut ctx = ctx;
        let (response, process) = self.run_lifecycle::<RequestStep>(&mut ctx, input).await?;
        Ok(RpcTransferMessageDto {
            request: input.clone(),
            response,
            transfer_agent_model: process,
        })
    }

    async fn setup_transfer_start(
        &self,
        ctx: DspTransferContext,
        input: &RpcTransferStartMessageDto,
    ) -> Outcome<RpcTransferMessageDto<RpcTransferStartMessageDto>> {
        let mut ctx = ctx;
        let (response, process) = self.run_lifecycle::<StartStep>(&mut ctx, input).await?;
        Ok(RpcTransferMessageDto {
            request: input.clone(),
            response,
            transfer_agent_model: process,
        })
    }

    async fn setup_transfer_suspension(
        &self,
        ctx: DspTransferContext,
        input: &RpcTransferSuspensionMessageDto,
    ) -> Outcome<RpcTransferMessageDto<RpcTransferSuspensionMessageDto>> {
        let mut ctx = ctx;
        let (response, process) = self
            .run_lifecycle::<SuspensionStep>(&mut ctx, input)
            .await?;
        Ok(RpcTransferMessageDto {
            request: input.clone(),
            response,
            transfer_agent_model: process,
        })
    }

    async fn setup_transfer_completion(
        &self,
        ctx: DspTransferContext,
        input: &RpcTransferCompletionMessageDto,
    ) -> Outcome<RpcTransferMessageDto<RpcTransferCompletionMessageDto>> {
        let mut ctx = ctx;
        let (response, process) = self
            .run_lifecycle::<CompletionStep>(&mut ctx, input)
            .await?;
        Ok(RpcTransferMessageDto {
            request: input.clone(),
            response,
            transfer_agent_model: process,
        })
    }

    async fn setup_transfer_termination(
        &self,
        ctx: DspTransferContext,
        input: &RpcTransferTerminationMessageDto,
    ) -> Outcome<RpcTransferMessageDto<RpcTransferTerminationMessageDto>> {
        let mut ctx = ctx;
        let (response, process) = self
            .run_lifecycle::<TerminationStep>(&mut ctx, input)
            .await?;
        Ok(RpcTransferMessageDto {
            request: input.clone(),
            response,
            transfer_agent_model: process,
        })
    }
}
