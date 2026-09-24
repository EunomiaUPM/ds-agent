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

use common::auth::access::AccessScope;
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

    pub fn with_event_bus(mut self, event_bus: Option<events::EventBus>) -> Self {
        self.event_bus = event_bus;
        self
    }
}

#[async_trait::async_trait]
impl NegotiationMessageServiceTrait for NegotiationMessageService {
    #[tracing::instrument(level = "info", skip_all, err)]
    async fn get_all(
        &self,
        scope: &AccessScope,
        filters: &NegotiationMessageFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<NegotiationMessageView>> {
        scope.require_read()?;
        filters.validate()?;

        let mut filters = filters.clone();
        filters.tenant_id = scope.resolve_query_tenant(filters.tenant_id.as_deref())?;

        let page = page.clamped();
        let (messages, total) = self
            .message_repo
            .get_all_negotiation_messages(&filters, &page, sort)
            .await?;

        let items: Vec<NegotiationMessageView> = messages
            .into_iter()
            .map(|m| NegotiationMessageView::assemble(m, None, None))
            .collect();

        Ok(Paginated::from_page(items, &page, total, |m| {
            Cursor::encode_composite(&m.inner.created_at, &m.inner.id)
        }))
    }

    #[tracing::instrument(level = "info", skip(self, scope), fields(id = %id), err)]
    async fn get_one(&self, scope: &AccessScope, id: &Urn) -> Outcome<NegotiationMessageView> {
        scope.require_read()?;
        let message = self
            .message_repo
            .get_negotiation_message_by_id(scope.tenant_filter().map(str::to_string), id)
            .await?
            .or_not_found(id, "negotiation message")?;

        let offer = self
            .offer_repo
            .get_offer_by_negotiation_message(scope.tenant_filter().map(str::to_string), id)
            .await?;

        let agreement = self
            .agreement_repo
            .get_agreement_by_negotiation_message(scope.tenant_filter().map(str::to_string), id)
            .await?;

        Ok(NegotiationMessageView::assemble(message, offer, agreement))
    }

    #[tracing::instrument(level = "info", skip_all, err)]
    async fn batch(
        &self,
        scope: &AccessScope,
        req: &BatchRequests,
    ) -> Outcome<Vec<NegotiationMessageView>> {
        scope.require_read()?;
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
            .get_batch_negotiation_messages(scope.tenant_filter().map(str::to_string), &req.ids)
            .await?;

        let views = messages
            .into_iter()
            .map(|m| NegotiationMessageView::assemble(m, None, None))
            .collect();

        Ok(views)
    }

    #[tracing::instrument(level = "info", skip_all, err)]
    async fn create(
        &self,
        scope: &AccessScope,
        cmd: &NewNegotiationMessageDto,
    ) -> Outcome<NegotiationMessageView> {
        let tenant_id = scope.resolve_create_tenant(cmd.tenant_id.as_deref())?;
        let new_model: NewNegotiationMessageModel = cmd.clone().into_model(tenant_id);
        let created = self
            .message_repo
            .create_negotiation_message(&new_model)
            .await?;

        let view = NegotiationMessageView::assemble(created, None, None);
        events::emit_action!(
            self.event_bus,
            &view.inner.tenant_id,
            crate::EVENT_PREFIX,
            "message",
            "create",
            &view
        );
        Ok(view)
    }

    #[tracing::instrument(level = "info", skip(self, scope), fields(id = %id), err)]
    async fn delete(&self, scope: &AccessScope, id: &Urn) -> Outcome<()> {
        scope.require_write()?;
        let owner = self
            .message_repo
            .delete_negotiation_message(scope.tenant_filter().map(str::to_string), id)
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
