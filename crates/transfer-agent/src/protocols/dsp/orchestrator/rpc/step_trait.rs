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
use crate::protocols::dsp::orchestrator::rpc::types::RpcTransferProcessMessageTrait;
use crate::protocols::dsp::persistence::TransferPersistenceTrait;
use crate::protocols::dsp::protocol_types::{
    TransferProcessAckDto, TransferProcessMessageTrait, TransferProcessMessageWrapper,
};
use crate::protocols::dsp::validator::traits::validation_rpc_steps::ValidationRpcSteps;
use common::dsp_common::context_field::ContextField;
use common::facades::ssi_auth_facade::MatesFacadeTrait;
use common::http_client::HttpClient;
use std::str::FromStr;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::Outcome;

#[async_trait::async_trait]
pub(super) trait TransferRpcStep: Send + Sync + 'static {
    type Input: RpcTransferProcessMessageTrait + Clone + Send + Sync + 'static;
    type DspMessage: TransferProcessMessageTrait + Clone + serde::Serialize + Send + Sync + 'static;

    fn url_suffix() -> &'static str;

    async fn validate(
        _validator: &Arc<dyn ValidationRpcSteps>,
        _input: &Self::Input,
    ) -> Outcome<()> {
        Ok(())
    }

    async fn prepare_context(
        ctx: &mut DspTransferContext,
        input: &Self::Input,
        persistence: &Arc<dyn TransferPersistenceTrait>,
    ) -> Outcome<()>;

    async fn pre_hook(
        _dp: &Arc<dyn DataPlaneFacadeTrait>,
        _ctx: &mut DspTransferContext,
    ) -> Outcome<()> {
        Ok(())
    }

    fn build_message(ctx: &DspTransferContext, input: &Self::Input) -> Outcome<Self::DspMessage>;

    async fn send_and_persist(
        http_client: &HttpClient,
        persistence: &Arc<dyn TransferPersistenceTrait>,
        ctx: &mut DspTransferContext,
        payload: Arc<Self::DspMessage>,
    ) -> Outcome<TransferProcessMessageWrapper<TransferProcessAckDto>>;

    async fn post_hook(
        _dp: &Arc<dyn DataPlaneFacadeTrait>,
        _ctx: &mut DspTransferContext,
    ) -> Outcome<()> {
        Ok(())
    }

    async fn apply_auth_token(
        mates_facade: &Arc<dyn MatesFacadeTrait>,
        http_client: &HttpClient,
        peer: &str,
    ) {
        if let Ok(mate) = mates_facade.get_mate_by_id(peer.to_string()).await {
            if let Some(token) = mate.token {
                http_client.set_auth_token(token).await;
            }
        }
    }
}

pub(super) async fn populate_continuation_context(
    ctx: &mut DspTransferContext,
    consumer_pid: &Urn,
    persistence: &Arc<dyn TransferPersistenceTrait>,
) -> Outcome<()> {
    let process = persistence
        .fetch_process(consumer_pid.to_string().as_str())
        .await?;

    let provider_pid =
        Urn::from_str(process.identifiers.get("providerPid").unwrap().as_str())?;
    let consumer_pid_urn =
        Urn::from_str(process.identifiers.get("consumerPid").unwrap().as_str())?;

    // The outgoing URL embeds the *peer's* identifier.
    let identifier_key = match process.inner.role.as_str() {
        "Provider" => "consumerPid",
        _ => "providerPid",
    };
    let peer_url_id = process.identifiers.get(identifier_key).unwrap().clone();
    let is_restart = process.inner.state_attribute.as_deref().unwrap_or("") != "OnRequest";

    ctx.associated_peer_id = process.inner.associated_agent_peer.clone();
    ctx.provider_address = process.inner.callback_address.clone();
    ctx.provider_pid = Some(provider_pid);
    ctx.consumer_pid = Some(consumer_pid_urn);
    ctx.peer_url_id = Some(peer_url_id);
    ctx.is_restart = is_restart;
    ctx.process = Some(process);

    Ok(())
}

pub(super) async fn continuation_send_and_persist<T>(
    http_client: &HttpClient,
    persistence: &Arc<dyn TransferPersistenceTrait>,
    ctx: &mut DspTransferContext,
    payload: Arc<T>,
    url_suffix: &str,
) -> Outcome<TransferProcessMessageWrapper<TransferProcessAckDto>>
where
    T: TransferProcessMessageTrait + Clone + serde::Serialize + Send + Sync + 'static,
{
    let callback = ctx.provider_address.clone().unwrap_or_default();
    let peer_url_id = ctx.peer_url_id.as_deref().unwrap_or_default();
    let peer_url = format!("{}/transfers/{}/{}", callback, peer_url_id, url_suffix);

    let message = TransferProcessMessageWrapper {
        context: ContextField::default(),
        _type: payload.get_message(),
        dto: payload.as_ref().clone(),
    };

    let response: TransferProcessMessageWrapper<TransferProcessAckDto> =
        http_client.post_json(&peer_url, &message).await?;

    let payload_dyn: Arc<dyn TransferProcessMessageTrait> = payload;
    let updated = persistence
        .update_process(ctx, payload_dyn, serde_json::to_value(&message).unwrap())
        .await?;

    ctx.process = Some(updated);
    Ok(response)
}
