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

pub mod persistence;
pub mod protocol;
pub mod step_agreement_reception;
pub mod step_agreement_verification;
pub mod step_consumer_request;
pub mod step_initial_offer;
pub mod step_initial_request;
pub mod step_negotiation_event;
pub mod step_provider_offer;
pub mod step_termination;
pub mod step_trait;

use crate::protocols::dsp::protocol_types::{
    NegotiationAckMessageDto, NegotiationAgreementMessageDto, NegotiationEventMessageDto,
    NegotiationOfferInitMessageDto, NegotiationOfferMessageDto, NegotiationProcessMessageWrapper,
    NegotiationRequestInitMessageDto, NegotiationRequestMessageDto,
    NegotiationTerminationMessageDto, NegotiationVerificationMessageDto,
};
use ymir::data::entities::shared::participant::Model as Mates;
use ymir::errors::Outcome;

#[async_trait::async_trait]
pub trait ProtocolOrchestratorTrait: Send + Sync + 'static {
    async fn on_get_negotiation(
        &self,
        id: &str,
        mate: &Mates,
    ) -> Outcome<NegotiationProcessMessageWrapper<NegotiationAckMessageDto>>;

    async fn on_initial_contract_request(
        &self,
        input: &NegotiationProcessMessageWrapper<NegotiationRequestInitMessageDto>,
        mate: &Mates,
    ) -> Outcome<(
        NegotiationProcessMessageWrapper<NegotiationAckMessageDto>,
        bool,
    )>;

    async fn on_consumer_request(
        &self,
        id: &str,
        input: &NegotiationProcessMessageWrapper<NegotiationRequestMessageDto>,
        mate: &Mates,
    ) -> Outcome<NegotiationProcessMessageWrapper<NegotiationAckMessageDto>>;

    async fn on_agreement_verification(
        &self,
        id: &str,
        input: &NegotiationProcessMessageWrapper<NegotiationVerificationMessageDto>,
        mate: &Mates,
    ) -> Outcome<NegotiationProcessMessageWrapper<NegotiationAckMessageDto>>;

    async fn on_initial_provider_offer(
        &self,
        input: &NegotiationProcessMessageWrapper<NegotiationOfferInitMessageDto>,
        mate: &Mates,
    ) -> Outcome<(
        NegotiationProcessMessageWrapper<NegotiationAckMessageDto>,
        bool,
    )>;

    async fn on_provider_offer(
        &self,
        id: &str,
        input: &NegotiationProcessMessageWrapper<NegotiationOfferMessageDto>,
        mate: &Mates,
    ) -> Outcome<NegotiationProcessMessageWrapper<NegotiationAckMessageDto>>;

    async fn on_agreement_reception(
        &self,
        id: &str,
        input: &NegotiationProcessMessageWrapper<NegotiationAgreementMessageDto>,
        mate: &Mates,
    ) -> Outcome<NegotiationProcessMessageWrapper<NegotiationAckMessageDto>>;

    async fn on_negotiation_event(
        &self,
        id: &str,
        input: &NegotiationProcessMessageWrapper<NegotiationEventMessageDto>,
        mate: &Mates,
    ) -> Outcome<NegotiationProcessMessageWrapper<NegotiationAckMessageDto>>;

    async fn on_negotiation_termination(
        &self,
        id: &str,
        input: &NegotiationProcessMessageWrapper<NegotiationTerminationMessageDto>,
        mate: &Mates,
    ) -> Outcome<NegotiationProcessMessageWrapper<NegotiationAckMessageDto>>;
}
