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

use ymir::oauth::{OwnedTrait, Owner, OwnerScope};
use ymir::types::oauth::UserInfo;
use ymir::batch_requests::BatchRequests;
use ymir::errors::NotFoundExt;
use ymir::paginated_spec::Cursor;
use ymir::query::{MAX_BATCH_IDS, Page, Paginated, QueryFilter, Sort};
use urn::Urn;
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::data::sea_orm::orm::negotiation_message::NewNegotiationMessageModel;
use crate::data::repo::negotiation_message::NegotiationMessageRepoTrait;
use crate::entities::filters::NegotiationMessageFilter;
use crate::entities::negotiation_message::NewNegotiationMessageDto;
use crate::setup::context::AppContext;
use crate::services::negotiation_message::NegotiationMessageServiceTrait;
use crate::services::negotiation_message::views::NegotiationMessageView;

/// Message service over the context's repos, emitting `negotiations:` events.
pub struct NegotiationMessageService {
    ctx: Arc<AppContext>,
}

impl NegotiationMessageService {
    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self { ctx }
    }

    fn message_repo(&self) -> Arc<dyn NegotiationMessageRepoTrait> {
        self.ctx.repos.negotiation_message_repo()
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
            .message_repo()
            .get_all_negotiation_messages(&OwnerScope::seeing(user), filters, &page, sort)
            .await?;

        let items: Vec<NegotiationMessageView> = messages
            .into_iter()
            .map(NegotiationMessageView::assemble)
            .collect();

        Ok(Paginated::from_page(items, &page, total, |m| {
            Cursor::encode_composite(&m.occurred_at, &m.id)
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
            .message_repo()
            .get_negotiation_message_by_id(&OwnerScope::seeing(user), id)
            .await?
            .or_not_found(id, "negotiation message")?;

        Ok(NegotiationMessageView::assemble(message))
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
            .message_repo()
            .get_batch_negotiation_messages(&OwnerScope::seeing(user), &req.ids)
            .await?;

        let views = messages
            .into_iter()
            .map(NegotiationMessageView::assemble)
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
            .message_repo()
            .create_negotiation_message(&new_model)
            .await?;

        let view = NegotiationMessageView::assemble(created);
        events::emit_action!(
            self.ctx.events,
            &view.owner(),
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
            .message_repo()
            .delete_negotiation_message(&OwnerScope::acting(user), id)
            .await?;
        events::emit_action!(
            self.ctx.events,
            &owner,
            crate::EVENT_PREFIX,
            "message",
            "delete",
            &events::EntityDeletedDto::new(id)
        );
        Ok(())
    }
}
