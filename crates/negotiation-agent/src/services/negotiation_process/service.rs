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

//! Negotiation process management service implementation.

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use common::oauth::{OwnedTrait, Owner, OwnerScope, UserInfo};
use common::batch_requests::BatchRequests;
use common::errors::NotFoundExt;
use common::paginated_spec::Cursor;
use common::query::{MAX_BATCH_IDS, Page, Paginated, QueryFilter, Sort};
use urn::Urn;
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::data::entities::negotiation_process::{
    EditNegotiationProcessModel, Model as NegotiationProcessModel, NewNegotiationProcessModel,
};
use crate::data::entities::negotiation_process_identifier::{
    EditNegotiationIdentifierModel, NewNegotiationIdentifierModel,
};
use crate::data::repo_traits::agreement_repo::AgreementRepoTrait;
use crate::data::repo_traits::negotiation_message_repo::NegotiationMessageRepoTrait;
use crate::data::repo_traits::negotiation_process_identifiers_repo::NegotiationIdentifierRepoTrait;
use crate::data::repo_traits::negotiation_process_repo::NegotiationProcessRepoTrait;
use crate::data::repo_traits::offer_repo::OfferRepoTrait;
use crate::entities::filters::NegotiationProcessFilter;
use crate::entities::negotiation_process::{EditNegotiationProcessDto, NewNegotiationProcessDto};
use crate::services::negotiation_process::NegotiationProcessServiceTrait;
use crate::services::negotiation_process::views::NegotiationProcessView;

/// Process service over the process and identifier repositories, emitting `negotiations:` events.
pub struct NegotiationProcessService {
    process_repo: Arc<dyn NegotiationProcessRepoTrait>,
    identifiers_repo: Arc<dyn NegotiationIdentifierRepoTrait>,
    messages_repo: Arc<dyn NegotiationMessageRepoTrait>,
    offers_repo: Arc<dyn OfferRepoTrait>,
    agreements_repo: Arc<dyn AgreementRepoTrait>,
    event_bus: Option<events::EventBus>,
}

impl NegotiationProcessService {
    pub fn new(
        process_repo: Arc<dyn NegotiationProcessRepoTrait>,
        identifiers_repo: Arc<dyn NegotiationIdentifierRepoTrait>,
        messages_repo: Arc<dyn NegotiationMessageRepoTrait>,
        offers_repo: Arc<dyn OfferRepoTrait>,
        agreements_repo: Arc<dyn AgreementRepoTrait>,
    ) -> Self {
        Self {
            process_repo,
            identifiers_repo,
            messages_repo,
            offers_repo,
            agreements_repo,
            event_bus: None,
        }
    }

    /// Publishes create, edit and delete events on `event_bus`.
    pub fn with_event_bus(mut self, event_bus: Option<events::EventBus>) -> Self {
        self.event_bus = event_bus;
        self
    }

    async fn fetch_details(
        &self,
        process: NegotiationProcessModel,
    ) -> Outcome<NegotiationProcessView> {
        let process_urn = Urn::from_str(&process.id)
            .map_err(|e| Errors::format(BadFormat::Received, e.to_string(), None))?;

        let (identifiers_models, messages, offers, agreement) = tokio::try_join!(
            self.identifiers_repo
                .get_identifiers_by_process_id(&process_urn),
            self.messages_repo
                .get_messages_by_process_id(&OwnerScope::All, &process_urn),
            self.offers_repo
                .get_offers_by_negotiation_process(&OwnerScope::All, &process_urn),
            self.agreements_repo.get_agreement_by_negotiation_process(
                &OwnerScope::All,
                &process_urn
            ),
        )?;

        let identifiers: HashMap<String, String> = identifiers_models
            .into_iter()
            .filter_map(|i| i.id_value.map(|v| (i.id_key, v)))
            .collect();

        Ok(NegotiationProcessView::assemble(
            process,
            identifiers,
            messages,
            offers,
            agreement,
        ))
    }
}

#[async_trait::async_trait]
impl NegotiationProcessServiceTrait for NegotiationProcessService {
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_all(
        &self,
        user: &UserInfo,
        filters: &NegotiationProcessFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<NegotiationProcessView>> {
        filters.validate()?;

        let page = page.clamped();
        let (processes, total) = self
            .process_repo
            .get_all_negotiation_processes(&OwnerScope::seeing(user), filters, &page, sort)
            .await?;

        let urns: Vec<Urn> = processes
            .iter()
            .filter_map(|p| Urn::from_str(&p.id).ok())
            .collect();

        let raw_identifiers = if urns.is_empty() {
            vec![]
        } else {
            self.identifiers_repo
                .get_identifiers_by_batch_process_id(&urns)
                .await?
        };

        let mut grouped_ids: HashMap<String, HashMap<String, String>> = HashMap::new();
        for id in raw_identifiers {
            if let Some(val) = id.id_value {
                grouped_ids
                    .entry(id.negotiation_agent_process_id)
                    .or_default()
                    .insert(id.id_key, val);
            }
        }

        let items: Vec<NegotiationProcessView> = processes
            .into_iter()
            .map(|p| {
                let extra = grouped_ids.remove(&p.id).unwrap_or_default();
                NegotiationProcessView::assemble(p, extra, vec![], vec![], None)
            })
            .collect();

        Ok(Paginated::from_page(items, &page, total, |p| {
            let updated = p.inner.updated_at.as_ref().unwrap_or(&p.inner.created_at);
            Cursor::encode_sorted(&p.inner.created_at, updated, sort)
        }))
    }

    #[tracing::instrument(
        level = "info",
        skip_all,
        err,
        fields(user = %user.id(), id = %id)
    )]
    async fn get_one(&self, user: &UserInfo, id: &Urn) -> Outcome<NegotiationProcessView> {
        let process = self
            .process_repo
            .get_negotiation_process_by_id(&OwnerScope::seeing(user), id)
            .await?
            .or_not_found(id, "negotiation process")?;

        self.fetch_details(process).await
    }

    #[tracing::instrument(
        level = "info",
        skip_all,
        err,
        fields(user = %user.id(), key_id = %key_id, id = %id)
    )]
    async fn get_by_key_id(
        &self,
        user: &UserInfo,
        key_id: &str,
        id: &Urn,
    ) -> Outcome<NegotiationProcessView> {
        let process = self
            .process_repo
            .get_negotiation_process_by_key_id(
                &OwnerScope::seeing(user),
                key_id,
                id,
            )
            .await?
            .or_not_found(id, "negotiation process")?;

        self.fetch_details(process).await
    }

    #[tracing::instrument(
        level = "info",
        skip_all,
        err,
        fields(user = %user.id(), value = %value)
    )]
    async fn get_by_key_value(
        &self,
        user: &UserInfo,
        value: &Urn,
    ) -> Outcome<NegotiationProcessView> {
        let process = self
            .process_repo
            .get_negotiation_process_by_key_value(&OwnerScope::seeing(user), value)
            .await?
            .or_not_found(value, "negotiation process")?;

        self.fetch_details(process).await
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn batch(
        &self,
        user: &UserInfo,
        req: &BatchRequests,
    ) -> Outcome<Vec<NegotiationProcessView>> {
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

        let processes = self
            .process_repo
            .get_batch_negotiation_processes(&OwnerScope::seeing(user), &req.ids)
            .await?;

        let urns: Vec<Urn> = processes
            .iter()
            .filter_map(|p| Urn::from_str(&p.id).ok())
            .collect();

        let raw_identifiers = if urns.is_empty() {
            vec![]
        } else {
            self.identifiers_repo
                .get_identifiers_by_batch_process_id(&urns)
                .await?
        };

        let mut grouped_ids: HashMap<String, HashMap<String, String>> = HashMap::new();
        for id in raw_identifiers {
            if let Some(val) = id.id_value {
                grouped_ids
                    .entry(id.negotiation_agent_process_id)
                    .or_default()
                    .insert(id.id_key, val);
            }
        }

        let views = processes
            .into_iter()
            .map(|p| {
                let extra = grouped_ids.remove(&p.id).unwrap_or_default();
                NegotiationProcessView::assemble(p, extra, vec![], vec![], None)
            })
            .collect();

        Ok(views)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn create(
        &self,
        user: &UserInfo,
        cmd: &NewNegotiationProcessDto,
    ) -> Outcome<NegotiationProcessView> {
        let cmd = cmd.clone();
        let owner = Owner::for_new(user, cmd.owner.clone(), cmd.visibility.clone());
        let new_process_model: NewNegotiationProcessModel = cmd.clone().into_model(owner);
        let created_process = self
            .process_repo
            .create_negotiation_process(&new_process_model)
            .await?;

        let process_urn = Urn::from_str(&created_process.id)
            .map_err(|e| Errors::parse(format!("Generated ID is not a valid URN: {e}"), None))?;

        if let Some(identifiers) = &cmd.identifiers {
            for (key, urn_value) in identifiers {
                let new_ident_model = NewNegotiationIdentifierModel {
                    id: None,
                    owner: created_process.owner(),
                    negotiation_agent_process_id: process_urn.clone(),
                    id_key: key.clone(),
                    id_value: Some(urn_value.to_string()),
                };
                self.identifiers_repo
                    .create_identifier(&new_ident_model)
                    .await?;
            }
        }

        let identifiers = cmd.identifiers.clone().unwrap_or_default();
        let view =
            NegotiationProcessView::assemble(created_process, identifiers, vec![], vec![], None);
        events::emit_action!(
            self.event_bus,
            &view.inner.owner(),
            crate::EVENT_PREFIX,
            "process",
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
        cmd: &EditNegotiationProcessDto,
    ) -> Outcome<NegotiationProcessView> {

        let edit_model: EditNegotiationProcessModel = cmd.clone().into();
        let updated_process = self
            .process_repo
            .put_negotiation_process(&OwnerScope::acting(user), id, &edit_model)
            .await?;

        let process_urn = Urn::from_str(&updated_process.id)
            .map_err(|e| Errors::parse(format!("Updated ID is not a valid URN: {e}"), None))?;

        if let Some(identifiers) = &cmd.identifiers {
            for (key, urn_value) in identifiers {
                let new_ident_model = EditNegotiationIdentifierModel {
                    id_key: Some(key.clone()),
                    id_value: Some(urn_value.to_string()),
                };
                let identifier_model = self
                    .identifiers_repo
                    .get_identifier_by_key(&process_urn, key)
                    .await?;
                if identifier_model.is_none() {
                    self.identifiers_repo
                        .create_identifier(&NewNegotiationIdentifierModel {
                            id: Some(common::utils::get_urn(None)),
                            owner: updated_process.owner(),
                            negotiation_agent_process_id: process_urn.clone(),
                            id_key: key.clone(),
                            id_value: Some(urn_value.to_string()),
                        })
                        .await?;
                } else {
                    let id_urn_ident = Urn::from_str(identifier_model.unwrap().id.as_str())
                        .map_err(|e| {
                            Errors::parse(format!("Identifier URN malformed: {e}"), None)
                        })?;
                    self.identifiers_repo
                        .put_identifier(&id_urn_ident, &new_ident_model)
                        .await?;
                }
            }
        }

        let view = self.fetch_details(updated_process).await?;
        events::emit_action!(
            self.event_bus,
            &view.inner.owner(),
            crate::EVENT_PREFIX,
            "process",
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
            .process_repo
            .delete_negotiation_process(&OwnerScope::acting(user), id)
            .await?;
        events::emit_action!(
            self.event_bus,
            &owner,
            crate::EVENT_PREFIX,
            "process",
            "delete",
            &events::EntityDeletedDto::new(id)
        );
        Ok(())
    }
}
