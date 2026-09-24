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

use common::auth::access::AccessScope;
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

    pub fn with_event_bus(mut self, event_bus: Option<events::EventBus>) -> Self {
        self.event_bus = event_bus;
        self
    }
}

#[async_trait::async_trait]
impl AgreementServiceTrait for AgreementService {
    #[tracing::instrument(level = "info", skip_all, err)]
    async fn get_all(
        &self,
        scope: &AccessScope,
        filters: &AgreementFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<AgreementView>> {
        scope.require_read()?;
        filters.validate()?;

        let mut filters = filters.clone();
        filters.tenant_id = scope.resolve_query_tenant(filters.tenant_id.as_deref())?;

        let page = page.clamped();
        let (agreements, total) = self
            .agreement_repo
            .get_all_agreements(&filters, &page, sort)
            .await?;

        let items: Vec<AgreementView> = agreements
            .into_iter()
            .map(AgreementView::assemble)
            .collect();

        Ok(Paginated::from_page(items, &page, total, |a| {
            Cursor::encode_composite(&a.inner.created_at, &a.inner.id)
        }))
    }

    #[tracing::instrument(level = "info", skip(self, scope), fields(id = %id), err)]
    async fn get_one(&self, scope: &AccessScope, id: &Urn) -> Outcome<AgreementView> {
        scope.require_read()?;
        let agreement = self
            .agreement_repo
            .get_agreement_by_id(scope.tenant_filter().map(str::to_string), id)
            .await?
            .or_not_found(id, "agreement")?;

        Ok(AgreementView::assemble(agreement))
    }

    #[tracing::instrument(level = "info", skip(self, scope), fields(process_id = %process_id), err)]
    async fn get_by_process(
        &self,
        scope: &AccessScope,
        process_id: &Urn,
    ) -> Outcome<AgreementView> {
        scope.require_read()?;
        let agreement = self
            .agreement_repo
            .get_agreement_by_negotiation_process(
                scope.tenant_filter().map(str::to_string),
                process_id,
            )
            .await?
            .or_not_found(process_id, "agreement")?;

        Ok(AgreementView::assemble(agreement))
    }

    #[tracing::instrument(level = "info", skip(self, scope), fields(message_id = %message_id), err)]
    async fn get_by_message(
        &self,
        scope: &AccessScope,
        message_id: &Urn,
    ) -> Outcome<AgreementView> {
        scope.require_read()?;
        let agreement = self
            .agreement_repo
            .get_agreement_by_negotiation_message(
                scope.tenant_filter().map(str::to_string),
                message_id,
            )
            .await?
            .or_not_found(message_id, "agreement")?;

        Ok(AgreementView::assemble(agreement))
    }

    #[tracing::instrument(level = "info", skip(self, scope), fields(assignee = %assignee), err)]
    async fn get_by_assignee(
        &self,
        scope: &AccessScope,
        assignee: &str,
    ) -> Outcome<Vec<AgreementView>> {
        scope.require_read()?;
        let agreements = self
            .agreement_repo
            .get_agreements_by_assignee(scope.tenant_filter().map(str::to_string), assignee)
            .await?;

        Ok(agreements
            .into_iter()
            .map(AgreementView::assemble)
            .collect())
    }

    #[tracing::instrument(level = "info", skip(self, scope), fields(assigner = %assigner), err)]
    async fn get_by_assigner(
        &self,
        scope: &AccessScope,
        assigner: &str,
    ) -> Outcome<Vec<AgreementView>> {
        scope.require_read()?;
        let agreements = self
            .agreement_repo
            .get_agreements_by_assigner(scope.tenant_filter().map(str::to_string), assigner)
            .await?;

        Ok(agreements
            .into_iter()
            .map(AgreementView::assemble)
            .collect())
    }

    #[tracing::instrument(level = "info", skip_all, err)]
    async fn batch(&self, scope: &AccessScope, req: &BatchRequests) -> Outcome<Vec<AgreementView>> {
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

        let agreements = self
            .agreement_repo
            .get_batch_agreements(scope.tenant_filter().map(str::to_string), &req.ids)
            .await?;

        Ok(agreements
            .into_iter()
            .map(AgreementView::assemble)
            .collect())
    }

    #[tracing::instrument(level = "info", skip_all, err)]
    async fn create(&self, scope: &AccessScope, cmd: &NewAgreementDto) -> Outcome<AgreementView> {
        let tenant_id = scope.resolve_create_tenant(cmd.tenant_id.as_deref())?;
        let new_model: NewAgreementModel = cmd.clone().into_model(tenant_id);
        let created = self.agreement_repo.create_agreement(&new_model).await?;

        let view = AgreementView::assemble(created);
        events::emit_action!(
            self.event_bus,
            &view.inner.tenant_id,
            crate::EVENT_PREFIX,
            "agreement",
            "create",
            &view
        );
        Ok(view)
    }

    #[tracing::instrument(level = "info", skip_all, err)]
    async fn edit(
        &self,
        scope: &AccessScope,
        id: &Urn,
        cmd: &EditAgreementDto,
    ) -> Outcome<AgreementView> {
        scope.require_write()?;

        let edit_model: EditAgreementModel = cmd.clone().into();
        let updated = self
            .agreement_repo
            .put_agreement(scope.tenant_filter().map(str::to_string), id, &edit_model)
            .await?;

        let view = AgreementView::assemble(updated);
        events::emit_action!(
            self.event_bus,
            &view.inner.tenant_id,
            crate::EVENT_PREFIX,
            "agreement",
            "edit",
            &view
        );
        Ok(view)
    }

    #[tracing::instrument(level = "info", skip(self, scope), fields(id = %id), err)]
    async fn delete(&self, scope: &AccessScope, id: &Urn) -> Outcome<()> {
        scope.require_write()?;
        let owner = self
            .agreement_repo
            .delete_agreement(scope.tenant_filter().map(str::to_string), id)
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
