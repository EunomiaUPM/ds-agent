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

//! The write side shared by the inbound and outbound flows, through the agent's services.

use std::sync::Arc;

use common::dsp_common::persistence::{DspPersistenceTrait, INITIAL_STATE};
use ymir::oauth::OwnerScope;
use ymir::types::oauth::UserInfo;
use serde_json::json;
use urn::Urn;
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::data::sea_orm::orm::negotiation_process::Model as ProcessModel;
use crate::data::sea_orm::orm::offer::Model as OfferModel;
use crate::entities::agreement::{EditAgreementDto, NewAgreementDto};
use crate::entities::negotiation_message::NewNegotiationMessageDto;
use crate::entities::negotiation_process::{EditNegotiationProcessDto, NewNegotiationProcessDto};
use crate::entities::offer::NewOfferDto;
use crate::protocols::dsp::entities::process::NegotiationProcess;
use crate::protocols::dsp::entities::state::NegotiationState;
use crate::protocols::dsp::entities::state_change::{
    NegotiationAttachment, NegotiationMessageRecord, NegotiationRecordedMessage,
    NegotiationStateChange,
};
use crate::setup::context::AppContext;

/// Writes negotiation processes and their history through the agent's services.
#[derive(Clone)]
pub struct NegotiationDspPersistence {
    ctx: Arc<AppContext>,
}

impl NegotiationDspPersistence {
    /// Protocol the processes and messages are stored under.
    pub const PROTOCOL: &'static str = "DSP";
    /// State an agreement takes once the negotiation is finalized.
    pub const AGREEMENT_ACTIVE: &'static str = "ACTIVE";

    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self { ctx }
    }

    /// Who the protocol writes as: an in-process flow on a process it already authorized.
    pub fn actor() -> UserInfo {
        UserInfo::system()
    }

    /// Stores a new process as `user` with the message that opened it and what it brought.
    pub async fn open_process(
        &self,
        user: &UserInfo,
        cmd: &NewNegotiationProcessDto,
        record: NegotiationMessageRecord,
        attachment: NegotiationAttachment,
    ) -> Outcome<NegotiationProcess> {
        let correlation = self.ctx.processes().create(user, cmd).await?.correlation;
        let pid = correlation
            .consumer_pid
            .or(correlation.provider_pid)
            .ok_or_else(|| Errors::crazy("opened negotiation has no pid", None))?;
        let process = self.hydrate_process(&pid).await?.ok_or_else(|| {
            Errors::crazy(format!("negotiation {pid} vanished after a write"), None)
        })?;
        let state = process.state;
        let message_id = self.write_message(&process, record, None, state).await?;
        self.attach(&process, &message_id, attachment).await?;
        Ok(process)
    }

    /// Moves the process like `change_process_state` and stores what the message brought.
    pub async fn change_with(
        &self,
        process: &NegotiationProcess,
        change: NegotiationStateChange,
        attachment: NegotiationAttachment,
    ) -> Outcome<NegotiationProcess> {
        let (from, to) = (process.state, change.target);
        let error_details = (change.code.is_some() || change.reason.is_some())
            .then(|| json!({ "code": change.code, "reason": change.reason }));
        let edit = EditNegotiationProcessDto {
            state: Some(to.to_string()),
            state_attribute: None,
            properties: None,
            error_details,
            identifiers: None,
        };
        self.ctx
            .processes()
            .edit(&Self::actor(), &process.id()?, &edit)
            .await?;
        let message_id = self
            .write_message(process, change.message, Some(from), to)
            .await?;
        self.attach(process, &message_id, attachment).await?;
        self.reload(process).await
    }

    /// The offer that opened the process.
    pub async fn opening_offer(&self, process: &NegotiationProcess) -> Outcome<Option<OfferModel>> {
        let offers = self
            .ctx
            .repos
            .offer_repo()
            .get_offers_by_negotiation_process(&OwnerScope::All, &process.id()?)
            .await?;
        Ok(offers.into_iter().next())
    }

    /// The target of the offer that opened the process; the TCK names its tests by it.
    pub async fn opening_target(&self, process: &NegotiationProcess) -> Outcome<Option<String>> {
        Ok(self.opening_offer(process).await?.and_then(|offer| {
            offer
                .offer_content
                .get("target")?
                .as_str()
                .map(str::to_string)
        }))
    }

    /// The offer most recently made in the process, by either side.
    pub async fn last_offer(&self, process: &NegotiationProcess) -> Outcome<Option<OfferModel>> {
        self.ctx
            .repos
            .offer_repo()
            .get_last_offer_by_negotiation_process(&OwnerScope::All, &process.id()?)
            .await
    }

    /// The process again after a write, by the pid that is ours.
    async fn reload(&self, process: &NegotiationProcess) -> Outcome<NegotiationProcess> {
        let pid = process
            .own_pid()
            .ok_or_else(|| Errors::crazy("stored negotiation has no pid of ours", None))?;
        self.hydrate_process(pid)
            .await?
            .ok_or_else(|| Errors::crazy(format!("negotiation {pid} vanished after a write"), None))
    }

    async fn write_message(
        &self,
        process: &NegotiationProcess,
        record: NegotiationMessageRecord,
        from: Option<NegotiationState>,
        to: NegotiationState,
    ) -> Outcome<Urn> {
        let cmd = NewNegotiationMessageDto {
            id: None,
            visibility: None,
            owner: Some(process.owner()),
            negotiation_agent_process_id: process.id()?,
            direction: record.direction.to_string(),
            protocol: Self::PROTOCOL.to_string(),
            message_type: record.message.wire_type().to_string(),
            state_transition_from: from
                .map_or_else(|| INITIAL_STATE.to_string(), |s| s.to_string()),
            state_transition_to: to.to_string(),
            payload: record.payload,
        };
        let view = self.ctx.messages().create(&Self::actor(), &cmd).await?;
        Self::urn(&view.id)
    }

    async fn attach(
        &self,
        process: &NegotiationProcess,
        message_id: &Urn,
        attachment: NegotiationAttachment,
    ) -> Outcome<()> {
        let actor = Self::actor();
        match attachment {
            NegotiationAttachment::Nothing => {}
            NegotiationAttachment::Offer(offer) => {
                let cmd = NewOfferDto {
                    id: None,
                    visibility: None,
                    owner: Some(process.owner()),
                    negotiation_agent_process_id: process.id()?,
                    negotiation_agent_message_id: message_id.clone(),
                    offer_id: offer.offer_id,
                    offer_content: offer.content,
                };
                self.ctx.offers().create(&actor, &cmd).await?;
            }
            NegotiationAttachment::Agreement(agreement) => {
                let cmd = NewAgreementDto {
                    id: Some(agreement.id),
                    visibility: None,
                    owner: Some(process.owner()),
                    negotiation_agent_process_id: process.id()?,
                    negotiation_agent_message_id: message_id.clone(),
                    consumer_participant_id: agreement.consumer_id,
                    provider_participant_id: agreement.provider_id,
                    agreement_content: agreement.content,
                    target: agreement.target,
                };
                self.ctx.agreements().create(&actor, &cmd).await?;
            }
            NegotiationAttachment::ActivateAgreement => {
                let agreements = self.ctx.agreements();
                let stored = agreements.get_by_process(&actor, &process.id()?).await?;
                let edit = EditAgreementDto {
                    state: Some(Self::AGREEMENT_ACTIVE.to_string()),
                };
                agreements
                    .edit(&actor, &Self::urn(&stored.inner.id)?, &edit)
                    .await?;
            }
        }
        Ok(())
    }

    /// The process stored under our own `id`, whoever owns it.
    pub async fn process_by_id(&self, id: &str) -> Outcome<Option<NegotiationProcess>> {
        let urn = id.parse::<Urn>().map_err(|_| {
            Errors::format(BadFormat::Received, format!("not a process id: {id}"), None)
        })?;
        let model = self
            .ctx
            .repos
            .negotiation_process_repo()
            .get_negotiation_process_by_id(&OwnerScope::All, &urn)
            .await?;
        self.with_identifiers(model).await
    }

    /// The process of a stored row, with its identifiers.
    async fn with_identifiers(
        &self,
        model: Option<ProcessModel>,
    ) -> Outcome<Option<NegotiationProcess>> {
        let Some(model) = model else {
            return Ok(None);
        };
        let identifiers = self
            .ctx
            .repos
            .negotiation_identifier_repo()
            .get_identifiers_by_process_id(&Self::urn(&model.id)?)
            .await?;
        NegotiationProcess::hydrate(
            model,
            identifiers.into_iter().map(|i| (i.id_key, i.id_value)),
        )
        .map(Some)
    }

    fn urn(id: &str) -> Outcome<Urn> {
        id.parse()
            .map_err(|_| Errors::crazy(format!("stored id is not a URN: {id}"), None))
    }
}

#[async_trait::async_trait]
impl DspPersistenceTrait for NegotiationDspPersistence {
    type Process = NegotiationProcess;
    type State = NegotiationState;
    type Message = NegotiationMessageRecord;

    async fn hydrate_process(&self, pid: &str) -> Outcome<Option<NegotiationProcess>> {
        let model = self
            .ctx
            .repos
            .negotiation_process_repo()
            .get_negotiation_process_by_key_value(&OwnerScope::All, pid)
            .await?;
        self.with_identifiers(model).await
    }

    async fn change_process_state(
        &self,
        process: &NegotiationProcess,
        change: NegotiationStateChange,
    ) -> Outcome<NegotiationProcess> {
        self.change_with(process, change, NegotiationAttachment::Nothing)
            .await
    }

    async fn record_message(
        &self,
        process: &NegotiationProcess,
        message: NegotiationRecordedMessage,
    ) -> Outcome<()> {
        self.write_message(process, message.message, message.from, message.to)
            .await
            .map(|_| ())
    }
}
