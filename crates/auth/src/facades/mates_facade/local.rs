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

use async_trait::async_trait;
use common::auth::AccessScope;
use common::facades::mates_facade::MatesFacadeTrait;
use common::query::QuerySpec;
use ymir::data::entities::shared::participant::Model as Mates;
use ymir::errors::Outcome;

use crate::entities::filters::ParticipantFilter;
use crate::modules::ParticipantModule;

/// Participants read straight from the auth services, scoped as the remote service token would be.
pub struct MatesLocalFacade {
    participants: Arc<dyn ParticipantModule>,
}

impl MatesLocalFacade {
    pub fn new(participants: Arc<dyn ParticipantModule>) -> Self {
        Self { participants }
    }
}

#[async_trait]
impl MatesFacadeTrait for MatesLocalFacade {
    #[tracing::instrument(
        level = "info",
        skip_all,
        err,
        fields(peer.service = "auth", tenant = %tenant_id)
    )]
    async fn get_mate_by_id(&self, tenant_id: String, mate_id: String) -> Outcome<Mates> {
        self.participants
            .get_by_id(&AccessScope::service(&tenant_id), mate_id)
            .await
    }

    #[tracing::instrument(
        level = "info",
        skip_all,
        err,
        fields(peer.service = "auth", tenant = %tenant_id)
    )]
    async fn get_me_mate(&self, tenant_id: String) -> Outcome<Mates> {
        self.participants
            .get_me(&AccessScope::service(&tenant_id))
            .await
    }

    /// First page with the default query, as `GET /mates/all` without parameters returns.
    #[tracing::instrument(
        level = "info",
        skip_all,
        err,
        fields(peer.service = "auth", tenant = %tenant_id)
    )]
    async fn get_all_mates(&self, tenant_id: String) -> Outcome<Vec<Mates>> {
        let query = QuerySpec::<ParticipantFilter>::default();
        let page = self
            .participants
            .get_all(
                &AccessScope::service(&tenant_id),
                &query.filter,
                &query.page,
                &query.sort,
            )
            .await?;
        Ok(page.items)
    }
}
