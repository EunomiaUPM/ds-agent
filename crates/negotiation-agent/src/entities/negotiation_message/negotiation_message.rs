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

use crate::data::entities::negotiation_message::{
    self as negotiation_message_model, NewNegotiationMessageModel,
};
use crate::data::factory_trait::NegotiationAgentRepoTrait;
use crate::entities::filters::NegotiationMessageFilter;
use crate::entities::negotiation_message::{
    NegotiationAgentMessagesTrait, NegotiationMessageDto, NewNegotiationMessageDto,
};
use common::paginated_spec::{Cursor, Page, Paginated, Sort};
use common::query::QueryFilter;
use std::str::FromStr;
use std::sync::Arc;
use tracing::error;
use urn::Urn;
use ymir::errors::{Errors, Outcome};

pub struct NegotiationAgentMessagesService {
    pub negotiation_repo: Arc<dyn NegotiationAgentRepoTrait>,
    pub event_bus: Option<events::EventBus>,
}

impl NegotiationAgentMessagesService {
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

    async fn enrich_message(
        &self,
        message: negotiation_message_model::Model,
    ) -> Outcome<NegotiationMessageDto> {
        let message_urn = Urn::from_str(&message.id).map_err(|e| {
            let err = Errors::parse(
                format!(
                    "Invalid URN found in database for message {}. Error: {}",
                    message.id, e
                ),
                None,
            );
            error!("{}", err);
            err
        })?;

        let offer = self
            .negotiation_repo
            .get_offer_repo()
            .get_offer_by_negotiation_message(&message.tenant_id, &message_urn)
            .await
            .map_err(|e| {
                let err = Errors::db(e.to_string(), None);
                error!("Error fetching linked offer: {}", err);
                err
            })?;

        let agreement = self
            .negotiation_repo
            .get_agreement_repo()
            .get_agreement_by_negotiation_message(&message.tenant_id, &message_urn)
            .await
            .map_err(|e| {
                let err = Errors::db(e.to_string(), None);
                error!("Error fetching linked agreement: {}", err);
                err
            })?;

        Ok(NegotiationMessageDto {
            inner: message,
            offer,
            agreement,
        })
    }
}

#[async_trait::async_trait]
impl NegotiationAgentMessagesTrait for NegotiationAgentMessagesService {
    async fn get_all_negotiation_messages(
        &self,
        filters: &NegotiationMessageFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<NegotiationMessageDto>> {
        filters.validate()?;
        let page = page.clamped();

        let (messages, total) = self
            .negotiation_repo
            .get_negotiation_message_repo()
            .get_all_negotiation_messages(filters, &page, sort)
            .await?;

        let mut dtos = Vec::with_capacity(messages.len());
        for msg in messages {
            let dto = self.enrich_message(msg).await?;
            dtos.push(dto);
        }

        Ok(Paginated::from_page(dtos, &page, total, |d| {
            Cursor::encode_composite(&d.inner.created_at, &d.inner.id)
        }))
    }

    async fn get_messages_by_process_id(
        &self,
        process_id: &Urn,
    ) -> Outcome<Vec<NegotiationMessageDto>> {
        let process_opt = self
            .negotiation_repo
            .get_negotiation_process_repo()
            .get_negotiation_process_by_key_value(None, process_id)
            .await?;
        let Some(process) = process_opt else {
            return Ok(vec![]);
        };

        let messages = self
            .negotiation_repo
            .get_negotiation_message_repo()
            .get_messages_by_process_id(&process.tenant_id, process_id)
            .await?;

        let mut dtos = Vec::with_capacity(messages.len());
        for msg in messages {
            let dto = self.enrich_message(msg).await?;
            dtos.push(dto);
        }

        Ok(dtos)
    }

    async fn get_negotiation_message_by_id(
        &self,
        id: &Urn,
    ) -> Outcome<Option<NegotiationMessageDto>> {
        let filter = NegotiationMessageFilter {
            id: Some(id.to_string()),
            ..Default::default()
        };
        let page = Page {
            limit: 1,
            ..Default::default()
        };
        let (messages, _) = self
            .negotiation_repo
            .get_negotiation_message_repo()
            .get_all_negotiation_messages(&filter, &page, &Sort::default())
            .await?;

        if let Some(message) = messages.into_iter().next() {
            Ok(Some(self.enrich_message(message).await?))
        } else {
            Ok(None)
        }
    }

    async fn create_negotiation_message(
        &self,
        new_model_dto: &NewNegotiationMessageDto,
    ) -> Outcome<NegotiationMessageDto> {
        let new_model: NewNegotiationMessageModel = new_model_dto.clone().into();

        let created = self
            .negotiation_repo
            .get_negotiation_message_repo()
            .create_negotiation_message(&new_model)
            .await?;

        let dto = NegotiationMessageDto {
            inner: created,
            offer: None,
            agreement: None,
        };
        events::emit_action!(
            self.event_bus,
            crate::EVENT_PREFIX,
            "message",
            "create",
            &dto
        );
        Ok(dto)
    }

    async fn delete_negotiation_message(&self, id: &Urn) -> Outcome<()> {
        if let Some(msg) = self.get_negotiation_message_by_id(id).await? {
            self.negotiation_repo
                .get_negotiation_message_repo()
                .delete_negotiation_message(&msg.inner.tenant_id, id)
                .await?;
        }
        events::emit_action!(
            self.event_bus,
            crate::EVENT_PREFIX,
            "message",
            "delete",
            &events::EntityDeletedDto::new(id)
        );
        Ok(())
    }
}
