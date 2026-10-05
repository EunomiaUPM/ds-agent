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

//! In-process adapter over the participant module.

use std::sync::Arc;

use async_trait::async_trait;
use common::facades::mates_facade::MatesFacadeTrait;
use common::query::QuerySpec;
use ymir::data::entities::shared::participant::Model as Mates;
use ymir::errors::Outcome;
use ymir::types::oauth::UserInfo;

use crate::entities::filters::ParticipantFilter;
use crate::modules::ParticipantModule;

/// Participants read straight from the participant module, as the given user sees them.
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
        fields(peer.service = "auth", user = %user.user_id())
    )]
    async fn get_mate_by_id(&self, user: &UserInfo, mate_id: String) -> Outcome<Mates> {
        self.participants.get_by_id(user, &mate_id).await
    }

    /// Read as the system user: the connector itself is the same for everyone.
    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "auth"))]
    async fn get_me_mate(&self) -> Outcome<Mates> {
        self.participants.get_myself(&UserInfo::system()).await
    }

    /// First page with the default query, as `GET /mates/all` without parameters returns.
    #[tracing::instrument(
        level = "info",
        skip_all,
        err,
        fields(peer.service = "auth", user = %user.user_id())
    )]
    async fn get_all_mates(&self, user: &UserInfo) -> Outcome<Vec<Mates>> {
        let query = QuerySpec::<ParticipantFilter>::default();
        let page = self
            .participants
            .get_all(user, &query.filter, &query.page, &query.sort)
            .await?;
        Ok(page.items)
    }
}
