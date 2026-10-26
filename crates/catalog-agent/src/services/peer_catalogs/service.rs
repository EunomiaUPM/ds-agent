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

//! Peer catalog service over the cache and the participant registry.

use std::sync::Arc;

use serde_json::Value;
use tracing::warn;
use ymir::data::entities::shared::participant::Model as Mates;
use ymir::errors::Outcome;
use ymir::types::oauth::UserInfo;

use crate::services::peer_catalogs::PeerCatalogServiceTrait;
use crate::setup::context::AppContext;

pub struct PeerCatalogService {
    ctx: Arc<AppContext>,
}

impl PeerCatalogService {
    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self { ctx }
    }
}

#[async_trait::async_trait]
impl PeerCatalogServiceTrait for PeerCatalogService {
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get_all(&self, user: &UserInfo) -> Outcome<Vec<(Mates, Value)>> {
        let mates = self.ctx.ports.auth.mates.get_all_mates(user).await?;
        let mut found = Vec::new();
        for mate in mates {
            match self.ctx.cache.get_peer_catalog(user.id(), &mate.participant_id).await {
                Ok(Some(catalog)) => found.push((mate, catalog)),
                Ok(None) => {}
                Err(e) => warn!(peer = %mate.participant_id, error = %e, "Peer catalog unreadable in the cache; skipping peer"),
            }
        }
        Ok(found)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn get(&self, user: &UserInfo, peer_id: &str) -> Outcome<Option<Value>> {
        self.ctx.cache.get_peer_catalog(user.id(), peer_id).await
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %user.id()))]
    async fn set(&self, user: &UserInfo, peer_id: &str, catalog: &Value) -> Outcome<()> {
        self.ctx.cache.put_peer_catalog(user.id(), peer_id, catalog).await
    }
}
