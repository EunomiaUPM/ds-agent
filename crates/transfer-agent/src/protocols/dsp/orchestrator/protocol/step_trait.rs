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
use crate::protocols::dsp::facades::FacadeTrait;
use crate::protocols::dsp::persistence::TransferPersistenceTrait;
use crate::protocols::dsp::protocol_types::{
    TransferProcessAckDto, TransferProcessMessageTrait, TransferProcessMessageWrapper,
};
use crate::protocols::dsp::validator::traits::validation_dsp_steps::ValidationDspSteps;
use std::sync::Arc;
use ymir::errors::{Errors, Outcome};

#[async_trait::async_trait]
pub(super) trait ProtocolStep: Send + Sync + 'static {
    type Dto: TransferProcessMessageTrait + Clone + serde::Serialize + Send + Sync + 'static;

    async fn validate(
        ctx: &DspTransferContext,
        validator: &Arc<dyn ValidationDspSteps>,
        input: &TransferProcessMessageWrapper<Self::Dto>,
    ) -> Outcome<()>;

    async fn prepare_context(
        ctx: &mut DspTransferContext,
        input: &TransferProcessMessageWrapper<Self::Dto>,
        persistence: &Arc<dyn TransferPersistenceTrait>,
        facades: &Arc<dyn FacadeTrait>,
    ) -> Outcome<Option<TransferProcessMessageWrapper<TransferProcessAckDto>>>;

    async fn persist(
        ctx: &mut DspTransferContext,
        persistence: &Arc<dyn TransferPersistenceTrait>,
        input: &TransferProcessMessageWrapper<Self::Dto>,
    ) -> Outcome<()>;

    async fn post_hook(
        ctx: &mut DspTransferContext,
        dp: &Arc<dyn DataPlaneFacadeTrait>,
        input: &TransferProcessMessageWrapper<Self::Dto>,
    ) -> Outcome<()>;
}

pub(super) async fn continuation_prepare_context(
    ctx: &mut DspTransferContext,
    persistence: &Arc<dyn TransferPersistenceTrait>,
) -> Outcome<Option<TransferProcessMessageWrapper<TransferProcessAckDto>>> {
    let peer_pid_str = ctx
        .peer_pid
        .as_ref()
        .ok_or_else(|| Errors::crazy("peer_pid missing in protocol continuation step", None))?
        .to_string();
    let process = persistence.fetch_process(&peer_pid_str).await?;
    ctx.process = Some(process);
    Ok(None)
}

pub(super) async fn continuation_persist<T>(
    ctx: &mut DspTransferContext,
    persistence: &Arc<dyn TransferPersistenceTrait>,
    input: &TransferProcessMessageWrapper<T>,
) -> Outcome<()>
where
    T: TransferProcessMessageTrait + Clone + serde::Serialize + Send + Sync + 'static,
{
    let updated = persistence
        .update_process(
            ctx,
            Arc::new(input.dto.clone()),
            serde_json::to_value(input).unwrap(),
        )
        .await?;
    ctx.process = Some(updated);
    Ok(())
}
