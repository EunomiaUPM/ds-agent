/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

use crate::protocols::dsp::context::DspTransferContext;
use crate::protocols::dsp::facades::dataplane_facade::DataPlaneFacadeTrait;
use crate::protocols::dsp::facades::FacadeTrait;
use crate::protocols::dsp::orchestrator::protocol::step_trait::ProtocolStep;
use crate::protocols::dsp::persistence::TransferPersistenceTrait;
use crate::protocols::dsp::protocol_types::{
    TransferProcessAckDto, TransferProcessMessageTrait, TransferProcessMessageWrapper,
    TransferRequestMessageDto,
};
use crate::protocols::dsp::validator::traits::validation_dsp_steps::ValidationDspSteps;
use connector::InteractionConfig;
use std::sync::Arc;
use ymir::errors::{Errors, Outcome};

pub(super) struct ProtocolRequestStep;

#[async_trait::async_trait]
impl ProtocolStep for ProtocolRequestStep {
    type Dto = TransferRequestMessageDto;

    async fn validate(
        _ctx: &DspTransferContext,
        validator: &Arc<dyn ValidationDspSteps>,
        input: &TransferProcessMessageWrapper<TransferRequestMessageDto>,
    ) -> Outcome<()> {
        validator
            .on_transfer_request(&Arc::new(input.clone()))
            .await
    }

    async fn prepare_context(
        ctx: &mut DspTransferContext,
        input: &TransferProcessMessageWrapper<TransferRequestMessageDto>,
        persistence: &Arc<dyn TransferPersistenceTrait>,
        facades: &Arc<dyn FacadeTrait>,
    ) -> Outcome<Option<TransferProcessMessageWrapper<TransferProcessAckDto>>> {
        let agreement_id = input
            .dto
            .get_agreement_id()
            .ok_or(Errors::crazy("no agreement id", None))?;
        let connector_instance = facades
            .get_data_service_facade()
            .await
            .resolve_connector_by_agreement_id(&agreement_id, Option::from(&input.dto.format))
            .await?;

        if matches!(connector_instance.interaction, InteractionConfig::Push(_))
            && input.dto.data_address.is_none()
        {
            return Err(Errors::crazy(
                "PUSH transfer requires a DataAddress from the consumer",
                None,
            ));
        }

        ctx.connector_instance = Some(connector_instance);
        ctx.input_data_address = input.dto.data_address.clone();

        // Idempotency: return the existing ack if the consumerPid is already known.
        let consumer_pid = input
            .dto
            .get_consumer_pid()
            .ok_or(Errors::missing_resource("", "no consumer id", None))?;
        let existing = persistence
            .get_transfer_process_service()
            .await?
            .get_transfer_process_by_key_id("consumerPid", &consumer_pid)
            .await;
        if let Ok(process) = existing {
            return Ok(Some(TransferProcessMessageWrapper::try_from(process)?));
        }

        Ok(None)
    }

    async fn persist(
        ctx: &mut DspTransferContext,
        persistence: &Arc<dyn TransferPersistenceTrait>,
        input: &TransferProcessMessageWrapper<TransferRequestMessageDto>,
    ) -> Outcome<()> {
        let process = persistence
            .create_process(
                ctx,
                Arc::new(input.dto.clone()),
                serde_json::to_value(input).unwrap(),
            )
            .await?;
        ctx.process = Some(process);
        Ok(())
    }

    async fn post_hook(
        ctx: &mut DspTransferContext,
        dp: &Arc<dyn DataPlaneFacadeTrait>,
        _input: &TransferProcessMessageWrapper<TransferRequestMessageDto>,
    ) -> Outcome<()> {
        dp.on_transfer_request_post(ctx).await?;
        Ok(())
    }
}
