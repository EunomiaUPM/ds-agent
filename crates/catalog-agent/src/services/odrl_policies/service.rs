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

//! Offer service over the repositories; an offer must sit on a resource of the kind it names.

use std::sync::Arc;

use common::errors::NotFoundExt;
use common::oauth::{OwnedTrait, Owner, OwnerScope};
use common::paginated_spec::{Cursor, Page, Paginated, Sort};
use common::query::QueryFilter;
use urn::{Urn, UrnBuilder};
use ymir::errors::{ErrorReason, Errors, Outcome};
use ymir::types::oauth::UserInfo;

use crate::entities::commands::NewOdrlOfferCommand;
use crate::entities::filters::OdrlPolicyFilter;
use crate::entities::odrl_offer::OdrlOffer;
use crate::services::odrl_policies::views::OdrlPolicyView;
use crate::services::odrl_policies::OdrlPolicyServiceTrait;
use crate::setup::context::AppContext;

pub struct OdrlPolicyService {
    ctx: Arc<AppContext>,
}

impl OdrlPolicyService {
    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self { ctx }
    }

    fn mint() -> Urn {
        UrnBuilder::new("odrl-policy", &uuid::Uuid::new_v4().to_string())
            .build()
            .expect("a UUID is a valid URN NSS")
    }

    async fn emit_deleted(&self, offer: &OdrlOffer) {
        events::emit_action!(
            self.ctx.events,
            &offer.owner(),
            crate::EVENT_PREFIX,
            "offer",
            "delete",
            &events::EntityDeletedDto::new(&offer.id)
        );
    }
}

#[async_trait::async_trait]
impl OdrlPolicyServiceTrait for OdrlPolicyService {
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn list(
        &self,
        user: &UserInfo,
        filter: &OdrlPolicyFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<OdrlOffer>> {
        filter.validate()?;
        let page = page.clamped();
        let scope = OwnerScope::seeing(user);
        let repo = self.ctx.repos.odrl_offer_repo();
        let items = repo.get_page(&scope, filter, &page, sort).await?;
        let total = repo.count(&scope, filter).await?;
        Ok(Paginated::from_page(items, &page, Some(total), |o| {
            Cursor::encode_composite(&o.created_at, o.id.as_ref())
        }))
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn batch(&self, user: &UserInfo, ids: &[Urn]) -> Outcome<Vec<OdrlOffer>> {
        self.ctx
            .repos
            .odrl_offer_repo()
            .get_batch(&OwnerScope::seeing(user), ids)
            .await
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn by_entities(&self, user: &UserInfo, entities: &[Urn]) -> Outcome<Vec<OdrlOffer>> {
        self.ctx
            .repos
            .odrl_offer_repo()
            .get_by_entities(&OwnerScope::seeing(user), entities)
            .await
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get(&self, user: &UserInfo, id: &Urn) -> Outcome<OdrlOffer> {
        self.ctx
            .repos
            .odrl_offer_repo()
            .get_by_id(&OwnerScope::seeing(user), id)
            .await?
            .or_not_found(id, "odrl offer")
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn create(&self, user: &UserInfo, command: &NewOdrlOfferCommand) -> Outcome<OdrlOffer> {
        let target = self
            .ctx
            .repos
            .resource_repo()
            .get_by_id(&OwnerScope::seeing(user), &command.entity_id)
            .await?;
        let expected = command.entity_type.kind();
        if target.as_ref().map(|t| t.kind()) != Some(expected) {
            return Err(Errors::validation_with(vec![ErrorReason {
                path: "entityId".into(),
                code: 1001,
                message: format!(
                    "{} is not a {} you can see",
                    command.entity_id,
                    expected.term()
                ),
            }]));
        }
        let offer = OdrlOffer {
            id: command.id.clone().unwrap_or_else(Self::mint),
            owner: Owner::for_new(user, command.owner.clone(), command.visibility.clone()),
            odrl_offer: serde_json::to_value(command.odrl_offer.inner())?,
            entity: command.entity_id.clone(),
            entity_type: command.entity_type.to_string(),
            created_at: chrono::Utc::now(),
            source_template_id: command.source_template_id.clone(),
            source_template_version: command.source_template_version.clone(),
            instantiation_parameters: command.instantiation_parameters.clone(),
            description: command.description.clone(),
            composition: command.composition.clone(),
        };
        let stored = self
            .ctx
            .repos
            .odrl_offer_repo()
            .insert(&offer, &command.references)
            .await?;
        self.ctx.cache.invalidate().await;
        events::emit_action!(
            self.ctx.events,
            &stored.owner(),
            crate::EVENT_PREFIX,
            "offer",
            "create",
            &OdrlPolicyView::assemble(stored.clone())
        );
        Ok(stored)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn delete(&self, user: &UserInfo, id: &Urn) -> Outcome<()> {
        let deleted = self
            .ctx
            .repos
            .odrl_offer_repo()
            .delete(&OwnerScope::acting(user), id)
            .await?;
        self.ctx.cache.invalidate().await;
        self.emit_deleted(&deleted).await;
        Ok(())
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn delete_by_entity(&self, user: &UserInfo, entity: &Urn) -> Outcome<()> {
        let deleted = self
            .ctx
            .repos
            .odrl_offer_repo()
            .delete_by_entity(&OwnerScope::acting(user), entity)
            .await?;
        self.ctx.cache.invalidate().await;
        for offer in &deleted {
            self.emit_deleted(offer).await;
        }
        Ok(())
    }
}
