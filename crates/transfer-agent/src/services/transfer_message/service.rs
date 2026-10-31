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

use ymir::oauth::{Owner, OwnerScope};
use ymir::types::oauth::UserInfo;
use ymir::errors::NotFoundExt;
use ymir::paginated_spec::Cursor;
use ymir::query::{Page, Paginated, QueryFilter, Sort};
use std::sync::Arc;
use urn::Urn;
use ymir::errors::Outcome;

use crate::data::repo::transfer_message::TransferMessageRepoTrait;
use crate::entities::commands::NewTransferMessageCommand;
use crate::entities::filters::TransferMessageFilter;
use crate::services::transfer_message::TransferMessageServiceTrait;
use crate::setup::context::AppContext;
use crate::services::transfer_message::views::TransferMessageView;

pub struct TransferMessageService {
    ctx: Arc<AppContext>,
}

impl TransferMessageService {
    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self { ctx }
    }

    fn message_repo(&self) -> Arc<dyn TransferMessageRepoTrait> {
        self.ctx.repos.transfer_message_repo()
    }

    // Refactors

    /// Validates the date window and
    /// clamps the page size — the normalization shared by both list endpoints.
    fn scoped_query(
        filters: &TransferMessageFilter,
        page: &Page,
    ) -> Outcome<(TransferMessageFilter, Page)> {
        filters.validate()?;
        let page = page.clamped();
        Ok((filters.clone(), page))
    }
}

#[async_trait::async_trait]
impl TransferMessageServiceTrait for TransferMessageService {
    /// Get all TransferMessage entities
    /// Listed within what the caller sees
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_all(
        &self,
        user: &UserInfo,
        filters: &TransferMessageFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<TransferMessageView>> {
        // Ensure access or 403
        let (filters, page) = Self::scoped_query(filters, page)?;
        // Hit db concurrently
        let scope = OwnerScope::seeing(user);
        let repo = self.message_repo();
        let (messages, total) = tokio::try_join!(
            repo.get_all_transfer_messages(&scope, &filters, &page, sort),
            repo.count_transfer_messages(&scope, &filters),
        )?;
        // Assemble into view
        let items = messages
            .into_iter()
            .map(TransferMessageView::assemble)
            .collect();
        Ok(Paginated::from_page(items, &page, Some(total), |m| {
            Cursor::encode_timestamp(&m.occurred_at)
        }))
    }

    /// Get single transfer message entity
    /// Not found unless the caller sees it
    #[tracing::instrument(
        level = "info",
        skip_all,
        err,
        fields(user = %user.id(), process_id = %process_id)
    )]
    async fn get_all_by_process(
        &self,
        user: &UserInfo,
        process_id: &Urn,
        filters: &TransferMessageFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<TransferMessageView>> {
        // Ensure access or 403
        let (filters, page) = Self::scoped_query(filters, page)?;
        // Hit db concurrently
        let scope = OwnerScope::seeing(user);
        let repo = self.message_repo();
        let (messages, total) = tokio::try_join!(
            repo.get_messages_by_process_id(&scope, process_id, &filters, &page, sort),
            repo.count_transfer_messages(&scope, &filters),
        )?;
        // Assemble into view
        let items = messages
            .into_iter()
            .map(TransferMessageView::assemble)
            .collect();
        Ok(Paginated::from_page(items, &page, Some(total), |m| {
            Cursor::encode_timestamp(&m.occurred_at)
        }))
    }

    /// Create a new transfer message entity
    #[tracing::instrument(
        level = "info",
        skip_all,
        err,
        fields(user = %user.id(), id = %id)
    )]
    async fn get_one(&self, user: &UserInfo, id: &Urn) -> Outcome<TransferMessageView> {
        let message = self
            .message_repo()
            .get_transfer_message_by_id(&OwnerScope::seeing(user), id)
            .await?
            .or_not_found(id, "transfer message")?;

        Ok(TransferMessageView::assemble(message))
    }

    /// Edit a transfer message
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn create(
        &self,
        user: &UserInfo,
        cmd: &NewTransferMessageCommand,
    ) -> Outcome<TransferMessageView> {
        let mut cmd = cmd.clone();
        cmd.owner = Some(Owner::for_new(user, cmd.owner.take(), cmd.visibility.clone()));
        // Create in db
        let message = self.message_repo().create_transfer_message(&cmd).await?;
        // Assemble into view
        let owner = message.owner().clone();
        let view = TransferMessageView::assemble(message);
        events::emit_action!(
            self.ctx.events,
            &owner,
            crate::EVENT_PREFIX,
            "message",
            "create",
            &view
        );
        Ok(view)
    }

    /// Delete a transfer message
    #[tracing::instrument(
        level = "info",
        skip_all,
        err,
        fields(user = %user.id(), id = %id)
    )]
    async fn delete(&self, user: &UserInfo, id: &Urn) -> Outcome<()> {
        // Hit db
        let owner = self
            .message_repo()
            .delete_transfer_message(&OwnerScope::acting(user), id)
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
