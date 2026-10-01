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

use crate::protocols::dsp::protocol_types::{
    NegotiationAgreementMessageDto, NegotiationEventMessageDto, NegotiationOfferInitMessageDto,
    NegotiationOfferMessageDto, NegotiationProcessMessageWrapper, NegotiationRequestInitMessageDto,
    NegotiationRequestMessageDto, NegotiationTerminationMessageDto,
    NegotiationVerificationMessageDto,
};
use common::dsp_common::DspActor;
use ymir::errors::Outcome;

#[async_trait::async_trait]
pub trait ValidationDspSteps: Send + Sync + 'static {
    async fn on_contract_request_init(
        &self,
        input: &NegotiationProcessMessageWrapper<NegotiationRequestInitMessageDto>,
    ) -> Outcome<()>;
    async fn on_contract_request(
        &self,
        actor: &DspActor,
        uri_id: &String,
        input: &NegotiationProcessMessageWrapper<NegotiationRequestMessageDto>,
    ) -> Outcome<()>;
    async fn on_contract_offer_init(
        &self,
        input: &NegotiationProcessMessageWrapper<NegotiationOfferInitMessageDto>,
    ) -> Outcome<()>;
    async fn on_contract_offer(
        &self,
        actor: &DspActor,
        uri_id: &String,
        input: &NegotiationProcessMessageWrapper<NegotiationOfferMessageDto>,
    ) -> Outcome<()>;
    async fn on_contract_agreement(
        &self,
        actor: &DspActor,
        uri_id: &String,
        input: &NegotiationProcessMessageWrapper<NegotiationAgreementMessageDto>,
    ) -> Outcome<()>;
    async fn on_contract_agreement_verification(
        &self,
        actor: &DspActor,
        uri_id: &String,
        input: &NegotiationProcessMessageWrapper<NegotiationVerificationMessageDto>,
    ) -> Outcome<()>;
    async fn on_contract_event(
        &self,
        actor: &DspActor,
        uri_id: &String,
        input: &NegotiationProcessMessageWrapper<NegotiationEventMessageDto>,
    ) -> Outcome<()>;
    async fn on_contract_termination(
        &self,
        actor: &DspActor,
        uri_id: &String,
        input: &NegotiationProcessMessageWrapper<NegotiationTerminationMessageDto>,
    ) -> Outcome<()>;
}
