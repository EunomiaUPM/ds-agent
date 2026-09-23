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

use std::sync::Arc;

use common::auth::access::AccessScope;
use common::batch_requests::BatchRequests;
use common::errors::NotFoundExt;
use common::paginated_spec::Cursor;
use common::query::{MAX_BATCH_IDS, Page, Paginated, QueryFilter, Sort};
use urn::Urn;
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::data::entities::offer::NewOfferModel;
use crate::data::repo_traits::offer_repo::OfferRepoTrait;
use crate::entities::filters::OfferFilter;
use crate::entities::offer::NewOfferDto;
use crate::services::offer::OfferServiceTrait;
use crate::services::offer::views::OfferView;

pub struct OfferService {
    offer_repo: Arc<dyn OfferRepoTrait>,
    event_bus: Option<events::EventBus>,
}

impl OfferService {
    pub fn new(offer_repo: Arc<dyn OfferRepoTrait>) -> Self {
        Self {
            offer_repo,
            event_bus: None,
        }
    }

    pub fn with_event_bus(mut self, event_bus: Option<events::EventBus>) -> Self {
        self.event_bus = event_bus;
        self
    }
}

#[async_trait::async_trait]
impl OfferServiceTrait for OfferService {
    #[tracing::instrument(level = "info", skip_all, err)]
    async fn get_all(
        &self,
        scope: &AccessScope,
        filters: &OfferFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<OfferView>> {
        scope.require_read()?;
        filters.validate()?;

        let mut filters = filters.clone();
        filters.tenant_id = scope.resolve_query_tenant(filters.tenant_id.as_deref())?;

        let page = page.clamped();
        let (offers, total) = self
            .offer_repo
            .get_all_offers(&filters, &page, sort)
            .await?;

        let items: Vec<OfferView> = offers.into_iter().map(OfferView::assemble).collect();

        Ok(Paginated::from_page(items, &page, total, |o| {
            Cursor::encode_composite(&o.inner.created_at, &o.inner.id)
        }))
    }

    #[tracing::instrument(level = "info", skip(self, scope), fields(id = %id), err)]
    async fn get_one(&self, scope: &AccessScope, id: &Urn) -> Outcome<OfferView> {
        scope.require_read()?;
        let offer = self
            .offer_repo
            .get_offer_by_id(scope.tenant_filter().map(str::to_string), id)
            .await?
            .or_not_found(id, "offer")?;

        Ok(OfferView::assemble(offer))
    }

    #[tracing::instrument(level = "info", skip(self, scope), fields(message_id = %message_id), err)]
    async fn get_by_negotiation_message(
        &self,
        scope: &AccessScope,
        message_id: &Urn,
    ) -> Outcome<OfferView> {
        scope.require_read()?;
        let offer = self
            .offer_repo
            .get_offer_by_negotiation_message(scope.tenant_filter().map(str::to_string), message_id)
            .await?
            .or_not_found(message_id, "offer")?;

        Ok(OfferView::assemble(offer))
    }

    #[tracing::instrument(level = "info", skip(self, scope), fields(offer_id = %offer_id), err)]
    async fn get_by_offer_id(&self, scope: &AccessScope, offer_id: &Urn) -> Outcome<OfferView> {
        scope.require_read()?;
        let offer = self
            .offer_repo
            .get_offer_by_offer_id(scope.tenant_filter().map(str::to_string), offer_id)
            .await?
            .or_not_found(offer_id, "offer")?;

        Ok(OfferView::assemble(offer))
    }

    #[tracing::instrument(level = "info", skip(self, scope), fields(process_id = %process_id), err)]
    async fn get_by_process(
        &self,
        scope: &AccessScope,
        process_id: &Urn,
    ) -> Outcome<Vec<OfferView>> {
        scope.require_read()?;
        let offers = self
            .offer_repo
            .get_offers_by_negotiation_process(
                scope.tenant_filter().map(str::to_string),
                process_id,
            )
            .await?;

        Ok(offers.into_iter().map(OfferView::assemble).collect())
    }

    #[tracing::instrument(level = "info", skip(self, scope), fields(process_id = %process_id), err)]
    async fn get_last_by_process(
        &self,
        scope: &AccessScope,
        process_id: &Urn,
    ) -> Outcome<OfferView> {
        scope.require_read()?;
        let offer = self
            .offer_repo
            .get_last_offer_by_negotiation_process(
                scope.tenant_filter().map(str::to_string),
                process_id,
            )
            .await?
            .or_not_found(process_id, "offer")?;
        Ok(OfferView::assemble(offer))
    }

    #[tracing::instrument(level = "info", skip_all, err)]
    async fn batch(&self, scope: &AccessScope, req: &BatchRequests) -> Outcome<Vec<OfferView>> {
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

        let offers = self
            .offer_repo
            .get_batch_offers(scope.tenant_filter().map(str::to_string), &req.ids)
            .await?;

        Ok(offers.into_iter().map(OfferView::assemble).collect())
    }

    #[tracing::instrument(level = "info", skip_all, err)]
    async fn create(&self, scope: &AccessScope, cmd: &NewOfferDto) -> Outcome<OfferView> {
        let tenant_id = scope.resolve_create_tenant(cmd.tenant_id.as_deref())?;
        let new_model: NewOfferModel = cmd.clone().into_model(tenant_id);
        let created = self.offer_repo.create_offer(&new_model).await?;

        let view = OfferView::assemble(created);
        events::emit_action!(
            self.event_bus,
            crate::EVENT_PREFIX,
            "offer",
            "create",
            &view
        );
        Ok(view)
    }

    #[tracing::instrument(level = "info", skip(self, scope), fields(id = %id), err)]
    async fn delete(&self, scope: &AccessScope, id: &Urn) -> Outcome<()> {
        scope.require_write()?;
        self.offer_repo
            .delete_offer(scope.tenant_filter().map(str::to_string), id)
            .await?;
        events::emit_action!(
            self.event_bus,
            crate::EVENT_PREFIX,
            "offer",
            "delete",
            &events::EntityDeletedDto::new(id)
        );
        Ok(())
    }
}
