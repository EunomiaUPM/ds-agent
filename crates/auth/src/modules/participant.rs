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

use crate::entities::filters::ParticipantFilter;
use crate::services::{HasConfig, HasRepo};
use async_trait::async_trait;
use chrono::Utc;
use common::auth::AccessScope;
use common::batch_requests::BatchRequestsAsString;
use common::config::types::traits::CommonConfigTrait;
use common::facades::VerifyTokenRequest;
use common::paginated_spec::{Cursor, Page, Paginated, Sort};
use json_value_merge::Merge;
use ymir::config::traits::HostsConfigTrait;
use ymir::config::types::HostType;
use ymir::data::entities::shared::participant::{Model, Plan};
use ymir::errors::Outcome;
use ymir::services::HasWallet;
use ymir::types::listing::{ParticipantListFilter, ParticipantSort};
use ymir::types::participants::ParticipantType;

#[async_trait]
pub trait ParticipantModule: HasWallet + HasRepo + HasConfig + Send + Sync + 'static {
    #[tracing::instrument(level = "info", skip_all, err, fields(tenant = %scope.acting_tenant()))]
    async fn get_all(
        &self,
        scope: &AccessScope,
        filter: &ParticipantFilter,
        page: &Page,
        sort: &Sort,
    ) -> Outcome<Paginated<Model>> {
        let list_page = page.list_page(
            sort,
            ParticipantSort::SavedAt,
            ParticipantSort::LastInteraction,
        )?;
        let list_filter = ParticipantListFilter {
            tenant_id: scope.tenant_filter().map(str::to_string),
            participant_type: filter.r#type.clone().unwrap_or(ParticipantType::All),
            nick_contains: filter.participant_nick.clone(),
            id_contains: filter.participant_id.clone(),
            saved_after: filter.created_after,
            saved_before: filter.created_before,
        };
        let listed = self
            .repo()
            .participant()
            .find_page(&list_filter, &list_page)
            .await?;
        let sort_field = list_page.sort;
        Ok(Paginated::from_page(
            listed.items,
            &page.clamped(),
            Some(listed.total),
            |last| {
                let ts = match sort_field {
                    ParticipantSort::SavedAt => last.saved_at,
                    ParticipantSort::LastInteraction => last.last_interaction,
                };
                Cursor::encode_composite(&ts, &last.participant_id)
            },
        ))
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(tenant = %scope.acting_tenant()))]
    async fn get_by_id(&self, scope: &AccessScope, id: String) -> Outcome<Model> {
        self.repo()
            .participant()
            .get_by_id(scope.acting_tenant(), &id)
            .await
    }

    /// This connector as seen by `scope`'s tenant; derived from the wallet, never stored.
    #[tracing::instrument(level = "info", skip_all, err, fields(tenant = %scope.acting_tenant()))]
    async fn get_me(&self, scope: &AccessScope) -> Outcome<Model> {
        let lock = self.wallet().get_identity();
        let identity = lock.read().await;
        let now = Utc::now();
        Ok(Model {
            tenant_id: scope.acting_tenant().clone(),
            participant_id: identity.did().id().to_string(),
            participant_nick: "Myself".to_string(),
            participant_type: ParticipantType::Agent,
            base_url: self.config().common().get_host(HostType::Http),
            token: None,
            saved_at: now,
            last_interaction: now,
            extra_fields: serde_json::json!({}),
        })
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(tenant = %scope.acting_tenant()))]
    async fn get_participant_batch(
        &self,
        scope: &AccessScope,
        payload: BatchRequestsAsString,
    ) -> Outcome<Vec<Model>> {
        self.repo()
            .participant()
            .get_batch(scope.acting_tenant(), &payload.ids)
            .await
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(tenant = %scope.acting_tenant()))]
    async fn get_by_token(
        &self,
        scope: &AccessScope,
        payload: VerifyTokenRequest,
    ) -> Outcome<Model> {
        let mate = self
            .repo()
            .participant()
            .get_by_token(&payload.token)
            .await?;
        scope.ensure_visible(&mate.tenant_id, &mate.participant_id)?;
        Ok(mate)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(tenant = %scope.acting_tenant()))]
    async fn update_extra_fields_by_id(
        &self,
        scope: &AccessScope,
        id: String,
        extra_fields: serde_json::Value,
    ) -> Outcome<Model> {
        scope.require_write()?;
        let mut mate = self.get_by_id(scope, id).await?;
        let mut merged_extra_fields = mate.extra_fields.clone();
        merged_extra_fields.merge(&extra_fields);
        mate.extra_fields = merged_extra_fields;
        self.repo().participant().update(mate).await
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(tenant = %scope.acting_tenant()))]
    async fn create_participant(&self, scope: &AccessScope, mut payload: Plan) -> Outcome<Model> {
        payload.tenant_id = scope.resolve_create_tenant(Some(&payload.tenant_id))?;
        self.repo().participant().create(payload).await
    }
}
