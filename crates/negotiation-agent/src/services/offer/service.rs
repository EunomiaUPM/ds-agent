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

//! Offer service over the repository.

use std::sync::Arc;

use common::oauth::{OwnedTrait, Owner, OwnerScope, UserInfo};
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

/// Offer service over a repository, emitting `negotiations:` events when a bus is set.
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

    /// Publishes create and delete events on `event_bus`.
    pub fn with_event_bus(mut self, event_bus: Option<events::EventBus>) -> Self {
        self.event_bus = event_bus;
        self
    }
}

#[async_trait::async_trait]
impl OfferServiceTrait for OfferService {
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_all(
        &self,
        user: &UserInfo,
        filters: &OfferFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<OfferView>> {
        filters.validate()?;

        let page = page.clamped();
        let (offers, total) = self
            .offer_repo
            .get_all_offers(&OwnerScope::seeing(user), filters, &page, sort)
            .await?;

        let items: Vec<OfferView> = offers.into_iter().map(OfferView::assemble).collect();

        Ok(Paginated::from_page(items, &page, total, |o| {
            Cursor::encode_composite(&o.inner.created_at, &o.inner.id)
        }))
    }

    #[tracing::instrument(
        level = "info",
        skip_all,
        err,
        fields(user = %user.id(), id = %id)
    )]
    async fn get_one(&self, user: &UserInfo, id: &Urn) -> Outcome<OfferView> {
        let offer = self
            .offer_repo
            .get_offer_by_id(&OwnerScope::seeing(user), id)
            .await?
            .or_not_found(id, "offer")?;

        Ok(OfferView::assemble(offer))
    }

    #[tracing::instrument(
        level = "info",
        skip_all,
        err,
        fields(user = %user.id(), message_id = %message_id)
    )]
    async fn get_by_negotiation_message(
        &self,
        user: &UserInfo,
        message_id: &Urn,
    ) -> Outcome<OfferView> {
        let offer = self
            .offer_repo
            .get_offer_by_negotiation_message(&OwnerScope::seeing(user), message_id)
            .await?
            .or_not_found(message_id, "offer")?;

        Ok(OfferView::assemble(offer))
    }

    #[tracing::instrument(
        level = "info",
        skip_all,
        err,
        fields(user = %user.id(), offer_id = %offer_id)
    )]
    async fn get_by_offer_id(&self, user: &UserInfo, offer_id: &Urn) -> Outcome<OfferView> {
        let offer = self
            .offer_repo
            .get_offer_by_offer_id(&OwnerScope::seeing(user), offer_id)
            .await?
            .or_not_found(offer_id, "offer")?;

        Ok(OfferView::assemble(offer))
    }

    #[tracing::instrument(
        level = "info",
        skip_all,
        err,
        fields(user = %user.id(), process_id = %process_id)
    )]
    async fn get_by_process(
        &self,
        user: &UserInfo,
        process_id: &Urn,
    ) -> Outcome<Vec<OfferView>> {
        let offers = self
            .offer_repo
            .get_offers_by_negotiation_process(
                &OwnerScope::seeing(user),
                process_id,
            )
            .await?;

        Ok(offers.into_iter().map(OfferView::assemble).collect())
    }

    #[tracing::instrument(
        level = "info",
        skip_all,
        err,
        fields(user = %user.id(), process_id = %process_id)
    )]
    async fn get_last_by_process(
        &self,
        user: &UserInfo,
        process_id: &Urn,
    ) -> Outcome<OfferView> {
        let offer = self
            .offer_repo
            .get_last_offer_by_negotiation_process(
                &OwnerScope::seeing(user),
                process_id,
            )
            .await?
            .or_not_found(process_id, "offer")?;
        Ok(OfferView::assemble(offer))
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn batch(&self, user: &UserInfo, req: &BatchRequests) -> Outcome<Vec<OfferView>> {
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
            .get_batch_offers(&OwnerScope::seeing(user), &req.ids)
            .await?;

        Ok(offers.into_iter().map(OfferView::assemble).collect())
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn create(&self, user: &UserInfo, cmd: &NewOfferDto) -> Outcome<OfferView> {
        let owner = Owner::for_new(user, cmd.owner.clone(), cmd.visibility.clone());
        let new_model: NewOfferModel = cmd.clone().into_model(owner);
        let created = self.offer_repo.create_offer(&new_model).await?;

        let view = OfferView::assemble(created);
        events::emit_action!(
            self.event_bus,
            &view.inner.owner(),
            crate::EVENT_PREFIX,
            "offer",
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
            .offer_repo
            .delete_offer(&OwnerScope::acting(user), id)
            .await?;
        events::emit_action!(
            self.event_bus,
            &owner,
            crate::EVENT_PREFIX,
            "offer",
            "delete",
            &events::EntityDeletedDto::new(id)
        );
        Ok(())
    }
}
