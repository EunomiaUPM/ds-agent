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

use base64::Engine;

use crate::data::repo::transfer_message::TransferMessageRepoTrait;
use crate::entities::commands::NewTransferMessageCommand;
use crate::entities::filters::TransferMessageFilter;
use crate::entities::transfer_message::TransferMessage;
use crate::services::transfer_message::TransferMessageServiceTrait;
use crate::services::transfer_message::views::TransferMessageView;
use common::auth::access::AccessScope;
use common::query::{Page, Paginated, Sort, clamp_page_limit, validate_date_range};
use std::sync::Arc;
use urn::Urn;
use ymir::errors::{Errors, Outcome};

pub(crate) struct TransferMessageService {
    message_repo: Arc<dyn TransferMessageRepoTrait>,
}

impl TransferMessageService {
    pub fn new(message_repo: Arc<dyn TransferMessageRepoTrait>) -> Self {
        Self { message_repo }
    }

    // Refactors

    /// Validates the date window, injects the scope's tenant into the filter, and
    /// clamps the page size — the normalization shared by both list endpoints.
    #[allow(clippy::result_large_err)]
    fn scoped_query(
        scope: &AccessScope,
        filters: &TransferMessageFilter,
        page: &Page,
    ) -> Outcome<(TransferMessageFilter, Page)> {
        validate_date_range(filters.created_after, filters.created_before)?;
        let mut filters = filters.clone();
        if let Some(tenant) = scope.tenant_filter() {
            filters.tenant_id = Some(tenant);
        }
        let page = Page {
            limit: clamp_page_limit(page.limit),
            cursor: page.cursor.clone(),
        };
        Ok((filters, page))
    }

    /// Confirms a restricted scope owns `id`, mapping a missing or foreign record
    /// to a 404 or 403. Admins (unrestricted) skip the lookup entirely.
    async fn ensure_access(&self, scope: &AccessScope, id: &Urn) -> Outcome<()> {
        if scope.tenant_filter().is_some() {
            let message = self
                .message_repo
                .get_transfer_message_by_id(id)
                .await?
                .ok_or_else(|| self.not_found(id))?;
            if !scope.permits(&message.tenant_id().to_string()) {
                return Err(self.not_allowed());
            }
        }
        Ok(())
    }

    // Helpers

    /// 404 used for missing records
    #[allow(clippy::result_large_err)]
    fn not_found(&self, id: &Urn) -> Errors {
        Errors::missing_resource(id.to_string(), "transfer message not found", None)
    }

    /// 403 used for unauthorized access
    #[allow(clippy::result_large_err)]
    fn not_allowed(&self) -> Errors {
        Errors::unauthorized("User not allowed for this action", None)
    }

    /// Encode next cursors
    fn encode_cursor(&self, msg: &TransferMessage) -> String {
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(msg.occurred_at().to_rfc3339())
    }
}

#[async_trait::async_trait]
impl TransferMessageServiceTrait for TransferMessageService {
    /// Get all TransferMessage entities
    /// Validate filtering based in tenant-id
    #[tracing::instrument(level = "info", skip_all, err)]
    async fn get_all(
        &self,
        scope: &AccessScope,
        filters: &TransferMessageFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<TransferMessageView>> {
        // Ensure access or 403
        let (filters, page) = Self::scoped_query(scope, filters, page)?;
        // Hit db concurrently
        let (messages, total) = tokio::try_join!(
            self.message_repo
                .get_all_transfer_messages(&filters, &page, sort),
            self.message_repo.count_transfer_messages(&filters),
        )?;
        // Create cursor
        let next_cursor = if messages.len() == page.limit as usize {
            messages.last().map(|m| self.encode_cursor(m))
        } else {
            None
        };
        // Assemble into view
        let items = messages
            .into_iter()
            .map(TransferMessageView::assemble)
            .collect();
        // Return paginated
        Ok(Paginated {
            items,
            next_cursor,
            total: Some(total),
        })
    }

    /// Get single transfer message entity
    /// If tenant-id is coincident ok, otherwise not_found
    #[tracing::instrument(level = "info", skip(self, scope, filters, page, sort), fields(process_id = %process_id), err)]
    async fn get_all_by_process(
        &self,
        scope: &AccessScope,
        process_id: &Urn,
        filters: &TransferMessageFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<TransferMessageView>> {
        // Ensure access or 403
        let (filters, page) = Self::scoped_query(scope, filters, page)?;
        // Hit db concurrently
        let (messages, total) = tokio::try_join!(
            self.message_repo
                .get_messages_by_process_id(process_id, &filters, &page, sort),
            self.message_repo.count_transfer_messages(&filters),
        )?;
        // Create cursor
        let next_cursor = if messages.len() == page.limit as usize {
            messages.last().map(|m| self.encode_cursor(m))
        } else {
            None
        };
        // Assemble into view
        let items = messages
            .into_iter()
            .map(TransferMessageView::assemble)
            .collect();
        // Return paginated
        Ok(Paginated {
            items,
            next_cursor,
            total: Some(total),
        })
    }

    /// Create a new transfer message entity
    #[tracing::instrument(level = "info", skip(self, scope), fields(id = %id), err)]
    async fn get_one(&self, scope: &AccessScope, id: &Urn) -> Outcome<TransferMessageView> {
        let message = self
            .message_repo
            .get_transfer_message_by_id(id)
            .await?
            // Filter by tenancy
            .filter(|m| scope.permits(&m.tenant_id().to_string()))
            .ok_or_else(|| self.not_found(id))?;
        Ok(TransferMessageView::assemble(message))
    }

    /// Edit a transfer message
    #[tracing::instrument(level = "info", skip_all, err)]
    async fn create(
        &self,
        scope: &AccessScope,
        cmd: &NewTransferMessageCommand,
    ) -> Outcome<TransferMessageView> {
        let mut cmd = cmd.clone();
        // Non-admins are forced into their own tenant; admins default to their acting
        // tenant only when the body leaves it unset.
        if scope.tenant_filter().is_some() || cmd.tenant_id.is_none() {
            cmd.tenant_id = Some(scope.acting_tenant().clone());
        }
        // Create in db
        let message = self.message_repo.create_transfer_message(&cmd).await?;
        // Assemble into view
        Ok(TransferMessageView::assemble(message))
    }

    /// Delete a transfer message
    #[tracing::instrument(level = "info", skip(self, scope), fields(id = %id), err)]
    async fn delete(&self, scope: &AccessScope, id: &Urn) -> Outcome<()> {
        // Validate access
        self.ensure_access(scope, id).await?;
        // Hit db
        self.message_repo.delete_transfer_message(id).await
    }
}
