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
use crate::protocols::dsp::facades::dataplane_facade::DataPlaneFacadeTrait;
use crate::protocols::dsp::facades::FacadeTrait;
use crate::protocols::dsp::orchestrator::protocol::step_trait::{resolve_process, ProtocolContext, ProtocolStep};
use crate::protocols::dsp::persistence::{CreateProcessInput, TransferPersistenceTrait};
use crate::protocols::dsp::protocol_types::{
    DataAddressDto, TransferProcessAckDto, TransferProcessMessageTrait, TransferProcessMessageWrapper,
    TransferRequestMessageDto,
};
use crate::protocols::dsp::validator::traits::validation_dsp_steps::ValidationDspSteps;
use anyhow::anyhow;
use rainbow_connector::InteractionConfig;
use std::sync::Arc;
use urn::Urn;
use ymir::errors::{Errors, Outcome};
// ─── ProtocolRequestStep ──────────────────────────────────────────────────────

/// Handles an inbound `TransferRequestMessage` from a Consumer.
///
/// This is the only inbound step that creates a new process record.  The
/// algorithm is:
/// 1. Validate the message.
/// 2. Resolve the connector for the referenced agreement and enforce PUSH constraints.
/// 3. Check idempotency: return the existing ack if the consumerPid is already known.
/// 4. Persist the new process.
/// 5. Register the provider-side dataplane session.
pub(super) struct ProtocolRequestStep;

#[async_trait::async_trait]
impl ProtocolStep for ProtocolRequestStep {
    type Dto = TransferRequestMessageDto;
    type Context = ProtocolContext;

    async fn validate(
        validator: &Arc<dyn ValidationDspSteps>,
        _id: &str,
        input: &TransferProcessMessageWrapper<TransferRequestMessageDto>,
    ) -> Outcome<()> {
        validator.on_transfer_request(&Arc::new(input.clone())).await
    }

    /// Resolves the connector and checks PUSH constraints.
    ///
    /// Returns `(ctx, Some(ack))` if the consumerPid is already stored
    /// (idempotent re-delivery); `(ctx, None)` otherwise.
    async fn prepare_context(
        _id: &str,
        peer: &str,
        input: &TransferProcessMessageWrapper<TransferRequestMessageDto>,
        persistence: &Arc<dyn TransferPersistenceTrait>,
        facades: &Arc<dyn FacadeTrait>,
    ) -> Outcome<(
        ProtocolContext,
        Option<TransferProcessMessageWrapper<TransferProcessAckDto>>,
    )> {
        // Resolve connector: agreement → dataset → distribution → connector instance.
        let agreement_id = input.dto.get_agreement_id().ok_or(Errors::crazy("no agreement id", None))?;
        let connector_instance = facades
            .get_data_service_facade()
            .await
            .resolve_connector_by_agreement_id(&agreement_id, Option::from(&input.dto.format))
            .await?;

        // PUSH connector requires the Consumer to supply a DataAddress.
        if matches!(connector_instance.interaction, InteractionConfig::Push(_))
            && input.dto.data_address.is_none()
        {
            return Err(Errors::crazy("PUSH transfer requires a DataAddress from the consumer", None));
        }

        let ctx =
            ProtocolContext { process: None, connector_instance, associated_peer: peer.to_string() };

        // Idempotency: return the existing ack if the consumerPid is already known.
        let consumer_pid = input.dto.get_consumer_pid().ok_or(anyhow!("no consumer id"))?;
        let existing = persistence
            .get_transfer_process_service()
            .await?
            .get_transfer_process_by_key_id("consumerPid", &consumer_pid)
            .await;
        if let Ok(process) = existing {
            return Ok((ctx, Some(TransferProcessMessageWrapper::try_from(process)?)));
        }

        Ok((ctx, None))
    }

    /// Creates the new process record for this transfer request.
    async fn persist(
        persistence: &Arc<dyn TransferPersistenceTrait>,
        _id: &str,
        ctx: &ProtocolContext,
        input: &TransferProcessMessageWrapper<TransferRequestMessageDto>,
    ) -> Outcome<TransferProcessDto> {
        persistence
            .create_process(CreateProcessInput {
                id: None,
                protocol: "DSP",
                direction: "INBOUND",
                associated_agent_peer: ctx.associated_peer.clone(),
                provider_pid: None,
                provider_address: None,
                payload_dto: Arc::new(input.dto.clone()),
                payload_value: serde_json::to_value(input).unwrap(),
            })
            .await
    }

    /// Registers the provider-side dataplane session for this transfer.
    async fn post_hook(
        dp: &Arc<dyn DataPlaneFacadeTrait>,
        ctx: &ProtocolContext,
        input: &TransferProcessMessageWrapper<TransferRequestMessageDto>,
        process_id: &Urn,
    ) -> Outcome<Option<DataAddressDto>> {
        let process = resolve_process(&process_id.to_string(), ctx).await?;
        dp.on_transfer_request_post(&process, &ctx.connector_instance, &input.dto.data_address)
            .await?;
        Ok(None)
    }
}
