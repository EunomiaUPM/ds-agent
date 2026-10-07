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

//! Negotiation message management service implementation.

use std::sync::Arc;

use common::oauth::{OwnedTrait, Owner, OwnerScope, UserInfo};
use common::batch_requests::BatchRequests;
use common::errors::NotFoundExt;
use common::paginated_spec::Cursor;
use common::query::{MAX_BATCH_IDS, Page, Paginated, QueryFilter, Sort};
use urn::Urn;
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::data::entities::negotiation_message::NewNegotiationMessageModel;
use crate::data::repo_traits::agreement_repo::AgreementRepoTrait;
use crate::data::repo_traits::negotiation_message_repo::NegotiationMessageRepoTrait;
use crate::data::repo_traits::offer_repo::OfferRepoTrait;
use crate::entities::filters::NegotiationMessageFilter;
use crate::entities::negotiation_message::NewNegotiationMessageDto;
use crate::services::negotiation_message::NegotiationMessageServiceTrait;
use crate::services::negotiation_message::views::NegotiationMessageView;

/// Message service, emitting `negotiations:` events when a bus is set.
pub struct NegotiationMessageService {
    message_repo: Arc<dyn NegotiationMessageRepoTrait>,
    offer_repo: Arc<dyn OfferRepoTrait>,
    agreement_repo: Arc<dyn AgreementRepoTrait>,
    event_bus: Option<events::EventBus>,
}

impl NegotiationMessageService {
    pub fn new(
        message_repo: Arc<dyn NegotiationMessageRepoTrait>,
        offer_repo: Arc<dyn OfferRepoTrait>,
        agreement_repo: Arc<dyn AgreementRepoTrait>,
    ) -> Self {
        Self {
            message_repo,
            offer_repo,
            agreement_repo,
            event_bus: None,
        }
    }

    /// Publishes create and delete events on `event_bus`.
    pub fn with_event_bus(mut self, event_bus: Option<events::EventBus>) -> Self {
        self.event_bus = event_bus;
        self
    }
}

#[async_trait::async_trait]
impl NegotiationMessageServiceTrait for NegotiationMessageService {
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_all(
        &self,
        user: &UserInfo,
        filters: &NegotiationMessageFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<NegotiationMessageView>> {
        filters.validate()?;

        let page = page.clamped();
        let (messages, total) = self
            .message_repo
            .get_all_negotiation_messages(&OwnerScope::seeing(user), filters, &page, sort)
            .await?;

        let items: Vec<NegotiationMessageView> = messages
            .into_iter()
            .map(|m| NegotiationMessageView::assemble(m, None, None))
            .collect();

        Ok(Paginated::from_page(items, &page, total, |m| {
            Cursor::encode_composite(&m.inner.created_at, &m.inner.id)
        }))
    }

    #[tracing::instrument(
        level = "info",
        skip_all,
        err,
        fields(user = %user.id(), id = %id)
    )]
    async fn get_one(&self, user: &UserInfo, id: &Urn) -> Outcome<NegotiationMessageView> {
        let message = self
            .message_repo
            .get_negotiation_message_by_id(&OwnerScope::seeing(user), id)
            .await?
            .or_not_found(id, "negotiation message")?;

        let offer = self
            .offer_repo
            .get_offer_by_negotiation_message(&OwnerScope::seeing(user), id)
            .await?;

        let agreement = self
            .agreement_repo
            .get_agreement_by_negotiation_message(&OwnerScope::seeing(user), id)
            .await?;

        Ok(NegotiationMessageView::assemble(message, offer, agreement))
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn batch(
        &self,
        user: &UserInfo,
        req: &BatchRequests,
    ) -> Outcome<Vec<NegotiationMessageView>> {
        if req.ids.len() > MAX_BATCH_IDS {
            return Err(Errors::format(
                BadFormat::Received,
                format!("batch request exceeds the maximum of {MAX_BATCH_IDS} ids"),
                None,
            ));
        }
        if req.ids.is_empty() {
            return Ok(vec![]);
        }

        let messages = self
            .message_repo
            .get_batch_negotiation_messages(&OwnerScope::seeing(user), &req.ids)
            .await?;

        let views = messages
            .into_iter()
            .map(|m| NegotiationMessageView::assemble(m, None, None))
            .collect();

        Ok(views)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn create(
        &self,
        user: &UserInfo,
        cmd: &NewNegotiationMessageDto,
    ) -> Outcome<NegotiationMessageView> {
        let owner = Owner::for_new(user, cmd.owner.clone(), cmd.visibility.clone());
        let new_model: NewNegotiationMessageModel = cmd.clone().into_model(owner);
        let created = self
            .message_repo
            .create_negotiation_message(&new_model)
            .await?;

        let view = NegotiationMessageView::assemble(created, None, None);
        events::emit_action!(
            self.event_bus,
            &view.inner.owner(),
            crate::EVENT_PREFIX,
            "message",
            "create",
            &view
        );
        Ok(view)
    }

    #[tracing::instrument(
        level = "info",
        skip_all,
        err,
        fields(user = %user.id(), id = %id)
    )]
    async fn delete(&self, user: &UserInfo, id: &Urn) -> Outcome<()> {
        let owner = self
            .message_repo
            .delete_negotiation_message(&OwnerScope::acting(user), id)
            .await?;
        events::emit_action!(
            self.event_bus,
            &owner,
            crate::EVENT_PREFIX,
            "message",
            "delete",
            &events::EntityDeletedDto::new(id)
        );
        Ok(())
    }
}
