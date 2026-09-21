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

use crate::data::entities::agreement::{EditAgreementModel, NewAgreementModel};
use crate::data::factory_trait::NegotiationAgentRepoTrait;
use crate::entities::agreement::{
    AgreementDto, EditAgreementDto, NegotiationAgentAgreementsTrait, NewAgreementDto,
};
use crate::entities::filters::{AgreementFilter, NegotiationMessageFilter};
use common::paginated_spec::{Cursor, Page, Paginated, Sort};
use common::query::QueryFilter;
use std::sync::Arc;
use tracing::error;
use urn::Urn;
use ymir::errors::{Errors, Outcome};

pub struct NegotiationAgentAgreementsService {
    pub negotiation_repo: Arc<dyn NegotiationAgentRepoTrait>,
    pub event_bus: Option<events::EventBus>,
}

impl NegotiationAgentAgreementsService {
    pub fn new(negotiation_repo: Arc<dyn NegotiationAgentRepoTrait>) -> Self {
        Self {
            negotiation_repo,
            event_bus: None,
        }
    }

    pub fn with_event_bus(mut self, event_bus: Option<events::EventBus>) -> Self {
        self.event_bus = event_bus;
        self
    }
}

#[async_trait::async_trait]
impl NegotiationAgentAgreementsTrait for NegotiationAgentAgreementsService {
    async fn get_all_agreements(
        &self,
        filters: &AgreementFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<AgreementDto>> {
        filters.validate()?;
        let page = page.clamped();

        let (agreements, total) = self
            .negotiation_repo
            .get_agreement_repo()
            .get_all_agreements(filters, &page, sort)
            .await
            .map_err(|e| {
                let err = Errors::db(e.to_string(), None);
                error!("{}", err);
                err
            })?;

        let dtos: Vec<AgreementDto> = agreements
            .into_iter()
            .map(|m| AgreementDto { inner: m })
            .collect();

        Ok(Paginated::from_page(dtos, &page, total, |d| {
            Cursor::encode_composite(&d.inner.created_at, &d.inner.id)
        }))
    }

    async fn get_batch_agreements(&self, ids: &Vec<Urn>) -> Outcome<Vec<AgreementDto>> {
        let mut dtos = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some(dto) = self.get_agreement_by_id(id).await? {
                dtos.push(dto);
            }
        }
        Ok(dtos)
    }

    async fn get_agreement_by_id(&self, id: &Urn) -> Outcome<Option<AgreementDto>> {
        let filter = AgreementFilter {
            id: Some(id.to_string()),
            ..Default::default()
        };
        let page = Page {
            limit: 1,
            ..Default::default()
        };
        let (agreements, _) = self
            .negotiation_repo
            .get_agreement_repo()
            .get_all_agreements(&filter, &page, &Sort::default())
            .await
            .map_err(|e| {
                let err = Errors::db(e.to_string(), None);
                error!("{}", err);
                err
            })?;

        Ok(agreements
            .into_iter()
            .next()
            .map(|m| AgreementDto { inner: m }))
    }

    async fn get_agreement_by_negotiation_process(
        &self,
        id: &Urn,
    ) -> Outcome<Option<AgreementDto>> {
        let process_opt = self
            .negotiation_repo
            .get_negotiation_process_repo()
            .get_negotiation_process_by_key_value(None, id)
            .await?;
        let Some(process) = process_opt else {
            return Ok(None);
        };

        let agreement = self
            .negotiation_repo
            .get_agreement_repo()
            .get_agreement_by_negotiation_process(&process.tenant_id, id)
            .await
            .map_err(|e| {
                let err = Errors::db(e.to_string(), None);
                error!("{}", err);
                err
            })?;

        Ok(agreement.map(|m| AgreementDto { inner: m }))
    }

    async fn get_agreement_by_negotiation_message(
        &self,
        id: &Urn,
    ) -> Outcome<Option<AgreementDto>> {
        let msg_opt = self
            .negotiation_repo
            .get_negotiation_message_repo()
            .get_all_negotiation_messages(
                &NegotiationMessageFilter {
                    id: Some(id.to_string()),
                    ..Default::default()
                },
                &Page {
                    limit: 1,
                    ..Default::default()
                },
                &Sort::default(),
            )
            .await?;
        let Some(msg) = msg_opt.0.into_iter().next() else {
            return Ok(None);
        };

        let agreement = self
            .negotiation_repo
            .get_agreement_repo()
            .get_agreement_by_negotiation_message(&msg.tenant_id, id)
            .await
            .map_err(|e| {
                let err = Errors::db(e.to_string(), None);
                error!("{}", err);
                err
            })?;

        Ok(agreement.map(|m| AgreementDto { inner: m }))
    }

    async fn get_agreements_by_assignee(&self, id: &String) -> Outcome<Vec<AgreementDto>> {
        let filter = AgreementFilter {
            consumer_id: Some(id.clone()),
            ..Default::default()
        };
        let (agreements, _) = self
            .negotiation_repo
            .get_agreement_repo()
            .get_all_agreements(&filter, &Page::default(), &Sort::default())
            .await
            .map_err(|e| {
                let err = Errors::db(e.to_string(), None);
                error!("{}", err);
                err
            })?;

        Ok(agreements
            .into_iter()
            .map(|m| AgreementDto { inner: m })
            .collect())
    }

    async fn get_agreements_by_assigner(&self, id: &String) -> Outcome<Vec<AgreementDto>> {
        let filter = AgreementFilter {
            provider_id: Some(id.clone()),
            ..Default::default()
        };
        let (agreements, _) = self
            .negotiation_repo
            .get_agreement_repo()
            .get_all_agreements(&filter, &Page::default(), &Sort::default())
            .await
            .map_err(|e| {
                let err = Errors::db(e.to_string(), None);
                error!("{}", err);
                err
            })?;

        Ok(agreements
            .into_iter()
            .map(|m| AgreementDto { inner: m })
            .collect())
    }

    async fn create_agreement(&self, new_model_dto: &NewAgreementDto) -> Outcome<AgreementDto> {
        let new_model: NewAgreementModel = new_model_dto.clone().into();

        let created = self
            .negotiation_repo
            .get_agreement_repo()
            .create_agreement(&new_model)
            .await
            .map_err(|e| {
                let err = Errors::db(e.to_string(), None);
                error!("{}", err);
                err
            })?;

        let dto = AgreementDto { inner: created };
        events::emit_action!(
            self.event_bus,
            crate::EVENT_PREFIX,
            "agreement",
            "create",
            &dto
        );
        Ok(dto)
    }

    async fn put_agreement(
        &self,
        id: &Urn,
        edit_model_dto: &EditAgreementDto,
    ) -> Outcome<AgreementDto> {
        let existing = self
            .get_agreement_by_id(id)
            .await?
            .ok_or_else(|| Errors::parse(format!("Agreement {} not found", id), None))?;

        let edit_model: EditAgreementModel = edit_model_dto.clone().into();

        let updated = self
            .negotiation_repo
            .get_agreement_repo()
            .put_agreement(&existing.inner.tenant_id, id, &edit_model)
            .await
            .inspect_err(|e| error!("{}", e))?;

        let dto = AgreementDto { inner: updated };
        events::emit_action!(
            self.event_bus,
            crate::EVENT_PREFIX,
            "agreement",
            "edit",
            &dto
        );
        Ok(dto)
    }

    async fn delete_agreement(&self, id: &Urn) -> Outcome<()> {
        if let Some(agreement) = self.get_agreement_by_id(id).await? {
            self.negotiation_repo
                .get_agreement_repo()
                .delete_agreement(&agreement.inner.tenant_id, id)
                .await
                .inspect_err(|e| error!("{}", e))?;
        }
        events::emit_action!(
            self.event_bus,
            crate::EVENT_PREFIX,
            "agreement",
            "delete",
            &events::EntityDeletedDto::new(id)
        );
        Ok(())
    }
}
