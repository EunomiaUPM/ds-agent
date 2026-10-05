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

//! Agreement management service implementation.

use std::sync::Arc;

use common::oauth::{OwnedTrait, Owner, OwnerScope, UserInfo};
use common::batch_requests::BatchRequests;
use common::errors::NotFoundExt;
use common::paginated_spec::Cursor;
use common::query::{MAX_BATCH_IDS, Page, Paginated, QueryFilter, Sort};
use urn::Urn;
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::data::entities::agreement::{EditAgreementModel, NewAgreementModel};
use crate::data::repo_traits::agreement_repo::AgreementRepoTrait;
use crate::entities::agreement::{EditAgreementDto, NewAgreementDto};
use crate::entities::filters::AgreementFilter;
use crate::services::agreement::AgreementServiceTrait;
use crate::services::agreement::views::AgreementView;

/// Agreement service over a repository, emitting `negotiations:` events when a bus is set.
pub struct AgreementService {
    agreement_repo: Arc<dyn AgreementRepoTrait>,
    event_bus: Option<events::EventBus>,
}

impl AgreementService {
    pub fn new(agreement_repo: Arc<dyn AgreementRepoTrait>) -> Self {
        Self {
            agreement_repo,
            event_bus: None,
        }
    }

    /// Publishes create, edit and delete events on `event_bus`.
    pub fn with_event_bus(mut self, event_bus: Option<events::EventBus>) -> Self {
        self.event_bus = event_bus;
        self
    }
}

#[async_trait::async_trait]
impl AgreementServiceTrait for AgreementService {
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_all(
        &self,
        user: &UserInfo,
        filters: &AgreementFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<AgreementView>> {
        filters.validate()?;

        let page = page.clamped();
        let (agreements, total) = self
            .agreement_repo
            .get_all_agreements(&OwnerScope::seeing(user), filters, &page, sort)
            .await?;

        let items: Vec<AgreementView> = agreements
            .into_iter()
            .map(AgreementView::assemble)
            .collect();

        Ok(Paginated::from_page(items, &page, total, |a| {
            Cursor::encode_composite(&a.inner.created_at, &a.inner.id)
        }))
    }

    #[tracing::instrument(
        level = "info",
        skip_all,
        err,
        fields(user = %user.id(), id = %id)
    )]
    async fn get_one(&self, user: &UserInfo, id: &Urn) -> Outcome<AgreementView> {
        let agreement = self
            .agreement_repo
            .get_agreement_by_id(&OwnerScope::seeing(user), id)
            .await?
            .or_not_found(id, "agreement")?;

        Ok(AgreementView::assemble(agreement))
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
    ) -> Outcome<AgreementView> {
        let agreement = self
            .agreement_repo
            .get_agreement_by_negotiation_process(
                &OwnerScope::seeing(user),
                process_id,
            )
            .await?
            .or_not_found(process_id, "agreement")?;

        Ok(AgreementView::assemble(agreement))
    }

    #[tracing::instrument(
        level = "info",
        skip_all,
        err,
        fields(user = %user.id(), message_id = %message_id)
    )]
    async fn get_by_message(
        &self,
        user: &UserInfo,
        message_id: &Urn,
    ) -> Outcome<AgreementView> {
        let agreement = self
            .agreement_repo
            .get_agreement_by_negotiation_message(
                &OwnerScope::seeing(user),
                message_id,
            )
            .await?
            .or_not_found(message_id, "agreement")?;

        Ok(AgreementView::assemble(agreement))
    }

    #[tracing::instrument(
        level = "info",
        skip_all,
        err,
        fields(user = %user.id(), assignee = %assignee)
    )]
    async fn get_by_assignee(
        &self,
        user: &UserInfo,
        assignee: &str,
    ) -> Outcome<Vec<AgreementView>> {
        let agreements = self
            .agreement_repo
            .get_agreements_by_assignee(&OwnerScope::seeing(user), assignee)
            .await?;

        Ok(agreements
            .into_iter()
            .map(AgreementView::assemble)
            .collect())
    }

    #[tracing::instrument(
        level = "info",
        skip_all,
        err,
        fields(user = %user.id(), assigner = %assigner)
    )]
    async fn get_by_assigner(
        &self,
        user: &UserInfo,
        assigner: &str,
    ) -> Outcome<Vec<AgreementView>> {
        let agreements = self
            .agreement_repo
            .get_agreements_by_assigner(&OwnerScope::seeing(user), assigner)
            .await?;

        Ok(agreements
            .into_iter()
            .map(AgreementView::assemble)
            .collect())
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn batch(&self, user: &UserInfo, req: &BatchRequests) -> Outcome<Vec<AgreementView>> {
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

        let agreements = self
            .agreement_repo
            .get_batch_agreements(&OwnerScope::seeing(user), &req.ids)
            .await?;

        Ok(agreements
            .into_iter()
            .map(AgreementView::assemble)
            .collect())
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn create(&self, user: &UserInfo, cmd: &NewAgreementDto) -> Outcome<AgreementView> {
        let owner = Owner::for_new(user, cmd.owner.clone(), cmd.visibility.clone());
        let new_model: NewAgreementModel = cmd.clone().into_model(owner);
        let created = self.agreement_repo.create_agreement(&new_model).await?;

        let view = AgreementView::assemble(created);
        events::emit_action!(
            self.event_bus,
            &view.inner.owner(),
            crate::EVENT_PREFIX,
            "agreement",
            "create",
            &view
        );
        Ok(view)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn edit(
        &self,
        user: &UserInfo,
        id: &Urn,
        cmd: &EditAgreementDto,
    ) -> Outcome<AgreementView> {

        let edit_model: EditAgreementModel = cmd.clone().into();
        let updated = self
            .agreement_repo
            .put_agreement(&OwnerScope::acting(user), id, &edit_model)
            .await?;

        let view = AgreementView::assemble(updated);
        events::emit_action!(
            self.event_bus,
            &view.inner.owner(),
            crate::EVENT_PREFIX,
            "agreement",
            "edit",
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
            .agreement_repo
            .delete_agreement(&OwnerScope::acting(user), id)
            .await?;
        events::emit_action!(
            self.event_bus,
            &owner,
            crate::EVENT_PREFIX,
            "agreement",
            "delete",
            &events::EntityDeletedDto::new(id)
        );
        Ok(())
    }
}
