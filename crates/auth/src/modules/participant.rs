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
use common::batch_requests::BatchRequestsAsString;
use common::config::types::traits::CommonConfigTrait;
use common::paginated_spec::{Cursor, Page, Paginated, Sort};
use json_value_merge::Merge;
use ymir::config::traits::HostsConfigTrait;
use ymir::config::types::HostType;
use ymir::data::entities::shared::participant::{Model, Plan};
use ymir::data::entities::shared::participant_relation;
use ymir::errors::Outcome;
use ymir::services::client::ClientExt;
use ymir::services::HasWallet;
use ymir::types::listing::{ParticipantListFilter, ParticipantSort};
use ymir::types::oauth::{RolePath, RoleTrait, UserInfo, SYSTEM_USER_ID};
use ymir::types::participants::{ParticipantType, Visibility};
use ymir::utils::http_client;

/// The participant registry: peers and authorities this connector knows, and itself. Every read
/// done for a user applies the visibility rules of the participant repository.
#[async_trait]
pub trait ParticipantModule: HasWallet + HasRepo + HasConfig + Send + Sync + 'static {
    /// Page of participants visible to `user`.
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_all(
        &self,
        user: &UserInfo,
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
            tenant: user.clone(),
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

    /// The participant `id` if `user` sees it; missing-resource error otherwise.
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_by_id(&self, user: &UserInfo, id: &str) -> Outcome<Model> {
        self.repo().participant().get_visible(user, id).await
    }

    /// Who added the participant `id`, as far as `user` may know: anonymous relations of others
    /// come without author. Missing-resource error if `user` does not see the participant.
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_relations(
        &self,
        user: &UserInfo,
        id: &str,
    ) -> Outcome<Vec<participant_relation::Model>> {
        self.get_by_id(user, id).await?;
        self.repo()
            .participant_relation()
            .get_visible_by_participant(user, id)
            .await
    }

    /// This connector itself, derived from the wallet; never stored.
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %_user.id()))]
    async fn get_myself(&self, _user: &UserInfo) -> Outcome<Model> {
        let lock = self.wallet().get_identity();
        let identity = lock.read().await;
        let now = Utc::now();
        Ok(Model {
            participant_id: identity.did().id().to_string(),
            participant_nick: "Myself".to_string(),
            participant_type: ParticipantType::Agent,
            base_url: self.config().common().get_host(HostType::Http),
            saved_at: now,
            last_interaction: now,
            extra_fields: serde_json::json!({}),
        })
    }

    /// The participants among the requested ids that `user` sees.
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_participant_batch(
        &self,
        user: &UserInfo,
        payload: BatchRequestsAsString,
    ) -> Outcome<Vec<Model>> {
        self.repo()
            .participant()
            .get_visible_batch(user, &payload.ids)
            .await
    }

    /// Merges `extra_fields` into the participant's own. Root only: the participant is shared by
    /// the whole organization.
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn update_extra_fields_by_id(
        &self,
        user: &UserInfo,
        id: &str,
        extra_fields: serde_json::Value,
    ) -> Outcome<Model> {
        user.require_root()?;
        let mut mate = self.get_by_id(user, id).await?;
        mate.extra_fields.merge(&extra_fields);
        self.repo().participant().update(mate).await
    }

    /// Adds the participant if new (an existing one is left as it is) and `user`'s relation with
    /// it, with `visibility`.
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn create_participant(
        &self,
        user: &UserInfo,
        payload: Plan,
        visibility: Visibility,
    ) -> Outcome<Model> {
        let model = self.repo().participant().create_if_absent(payload).await?;
        let relation = participant_relation::Model {
            user_id: user.id().to_string(),
            participant_id: model.participant_id.clone(),
            username: user.username().map(ToString::to_string),
            role: user.role().clone(),
            visibility,
        };
        self.repo()
            .participant_relation()
            .force_update(relation)
            .await?;
        Ok(model)
    }

    #[tracing::instrument(level = "info", skip_all, err)]
    async fn sync_directory(&self) -> Outcome<u64> {
        let myself = self.get_myself(&UserInfo::system()).await?;
        let authorities: Vec<Model> = self
            .repo()
            .participant()
            .get_all(None, None)
            .await?
            .into_iter()
            .filter(|p| p.participant_type == ParticipantType::Authority)
            .collect();

        let mut synced = 0;
        for authority in authorities {
            let url = format!(
                "{}/.well-known/federated-catalog",
                authority.base_url.trim_end_matches('/')
            );
            let listed: Vec<Plan> = match http_client().get_json(&url, None).await {
                Ok(listed) => listed,
                Err(e) => {
                    e.log();
                    continue;
                }
            };
            for plan in listed {
                if plan.participant_id == myself.participant_id {
                    continue;
                }
                if self.sync_participant(plan).await? {
                    synced += 1;
                }
            }
        }
        Ok(synced)
    }

    async fn sync_participant(&self, plan: Plan) -> Outcome<bool> {
        let participant_id = plan.participant_id.clone();
        let existing = self
            .repo()
            .participant()
            .get_batch(std::slice::from_ref(&participant_id))
            .await?;
        let changed = match existing.first() {
            None => {
                self.repo().participant().create_if_absent(plan).await?;
                true
            }
            Some(known) => {
                let changed = known.participant_nick != plan.participant_nick
                    || known.base_url != plan.base_url;
                if changed {
                    self.repo().participant().force_update(plan).await?;
                }
                changed
            }
        };
        let relation = participant_relation::Model {
            user_id: SYSTEM_USER_ID.to_string(),
            username: None,
            participant_id,
            role: RolePath::root(),
            visibility: Visibility::Public,
        };
        self.repo()
            .participant_relation()
            .force_update(relation)
            .await?;
        Ok(changed)
    }
}
