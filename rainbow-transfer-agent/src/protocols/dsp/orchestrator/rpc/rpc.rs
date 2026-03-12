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
use rainbow_common::facades::ssi_auth_facade::MatesFacadeTrait;
use rainbow_common::http_client::HttpClient;
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;

// ─── Service ─────────────────────────────────────────────────────────────────

/// RPC orchestrator for outbound transfer operations.
///
/// Translates internal RPC requests into DSP protocol messages, sends them to
/// the remote peer over HTTP, and persists the resulting state transitions.
/// All five operations (request + four lifecycle steps) are driven by the
/// [`TransferRpcStep`] template; `run_lifecycle` encodes the algorithm once.
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
        RPCOrchestratorService { validator, persistence_service, http_client, facades, mates_facade }
    }
}

// ─── Trait implementation ─────────────────────────────────────────────────────

#[async_trait::async_trait]
impl RPCOrchestratorTrait for RPCOrchestratorService {
    async fn setup_transfer_request(
        &self,
        input: &RpcTransferRequestMessageDto,
    ) -> anyhow::Result<RpcTransferMessageDto<RpcTransferRequestMessageDto>> {
        let (response, process) = self.run_lifecycle::<RequestStep>(input).await?;
        Ok(RpcTransferMessageDto { request: input.clone(), response, transfer_agent_model: process })
    }

    async fn setup_transfer_start(
        &self,
        input: &RpcTransferStartMessageDto,
    ) -> anyhow::Result<RpcTransferMessageDto<RpcTransferStartMessageDto>> {
        let (response, process) = self.run_lifecycle::<StartStep>(input).await?;
        Ok(RpcTransferMessageDto { request: input.clone(), response, transfer_agent_model: process })
    }

    async fn setup_transfer_suspension(
        &self,
        input: &RpcTransferSuspensionMessageDto,
    ) -> anyhow::Result<RpcTransferMessageDto<RpcTransferSuspensionMessageDto>> {
        let (response, process) = self.run_lifecycle::<SuspensionStep>(input).await?;
        Ok(RpcTransferMessageDto { request: input.clone(), response, transfer_agent_model: process })
    }

    async fn setup_transfer_completion(
        &self,
        input: &RpcTransferCompletionMessageDto,
    ) -> anyhow::Result<RpcTransferMessageDto<RpcTransferCompletionMessageDto>> {
        let (response, process) = self.run_lifecycle::<CompletionStep>(input).await?;
        Ok(RpcTransferMessageDto { request: input.clone(), response, transfer_agent_model: process })
    }

    async fn setup_transfer_termination(
        &self,
        input: &RpcTransferTerminationMessageDto,
    ) -> anyhow::Result<RpcTransferMessageDto<RpcTransferTerminationMessageDto>> {
        let (response, process) = self.run_lifecycle::<TerminationStep>(input).await?;
        Ok(RpcTransferMessageDto { request: input.clone(), response, transfer_agent_model: process })
    }
}

// ─── Template engine ──────────────────────────────────────────────────────────

impl RPCOrchestratorService {
    /// Execute any RPC lifecycle step using the [`TransferRpcStep`] template.
    ///
    /// The algorithm is always the same regardless of step type:
    /// validate → prepare context → pre-hook → build message →
    /// auth → send+persist → post-hook.
    async fn run_lifecycle<S: TransferRpcStep>(
        &self,
        input: &S::Input,
    ) -> anyhow::Result<(
        TransferProcessMessageWrapper<TransferProcessAckDto>,
        TransferProcessDto,
    )> {
        S::validate(&self.validator, input).await?;
        let ctx = S::prepare_context(input, &self.persistence_service).await?;

        let dp = self.facades.get_data_plane_facade().await;
        let pre_addr = S::pre_hook(&dp, &ctx).await?;

        let message = S::build_message(input, &ctx, pre_addr)?;

        S::apply_auth_token(&self.mates_facade, &self.http_client, S::auth_peer(&ctx)).await;
        let (response, new_process) = S::send_and_persist(
            &self.http_client,
            &self.persistence_service,
            &ctx,
            Arc::new(message),
            S::url_suffix(),
        )
        .await?;

        let new_id = Urn::from_str(new_process.inner.id.as_str())?;
        S::post_hook(&dp, &new_id).await?;

        Ok((response, new_process))
    }
}
