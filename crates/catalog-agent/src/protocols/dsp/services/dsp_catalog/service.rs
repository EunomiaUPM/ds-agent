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

//! The provider side over the closures, the DSP adapter and the projection cache.

use std::sync::Arc;

use common::oauth::OwnerScope;
use serde_json::Value;
use urn::Urn;
use ymir::errors::{Errors, Outcome};
use ymir::types::oauth::UserInfo;

use crate::cache::ProjectionKey;
use crate::entities::resource_kind::ResourceKind;
use crate::profiles::dsp::DspProfileAdapter;
use crate::profiles::ProfileAdapterTrait;
use crate::protocols::dsp::entities::protocol_fields::CatalogProtocolFields;
use crate::protocols::dsp::services::dsp_catalog::DspCatalogServiceTrait;
use crate::setup::context::AppContext;

pub struct DspCatalogService {
    ctx: Arc<AppContext>,
}

impl DspCatalogService {
    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self { ctx }
    }

    /// The DSP adapter the composition registered; the plane cannot answer without it.
    fn adapter(&self) -> Outcome<Arc<dyn ProfileAdapterTrait>> {
        self.ctx
            .profiles
            .get(DspProfileAdapter::KEY)
            .ok_or_else(|| Errors::crazy("the DSP profile is not registered", None))
    }

    /// Who reads, for the cache: the peer's identity under the role of its grant.
    fn reader(peer: &UserInfo) -> String {
        format!("{}|{}", peer.id(), peer.role())
    }

    fn not_found(id: &Urn) -> Errors {
        Errors::missing_resource(id.to_string(), "dataset not found", None)
    }
}

#[async_trait::async_trait]
impl DspCatalogServiceTrait for DspCatalogService {
    #[tracing::instrument(level = "info", skip_all, err, fields(user = %peer.id()))]
    async fn catalog(&self, peer: &UserInfo, _request: &CatalogProtocolFields) -> Outcome<Value> {
        let adapter = self.adapter()?;
        let generation = self.ctx.cache.generation().await;
        let key = ProjectionKey::of(adapter.key(), "main", &Self::reader(peer), generation);
        if let Some(cached) = self.ctx.cache.get_projection(&key).await {
            return Ok(cached);
        }
        let (main, set) = self.ctx.closures().main_catalog(&OwnerScope::seeing(peer)).await?;
        let catalog = adapter.project(&set, &main)?;
        self.ctx.cache.put_projection(&key, &catalog).await;
        Ok(catalog)
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(user = %peer.id()))]
    async fn dataset(&self, peer: &UserInfo, id: &Urn) -> Outcome<Value> {
        let adapter = self.adapter()?;
        let generation = self.ctx.cache.generation().await;
        let key = ProjectionKey::of(adapter.key(), id.as_ref(), &Self::reader(peer), generation);
        if let Some(cached) = self.ctx.cache.get_projection(&key).await {
            return Ok(cached);
        }
        let (root, set) = self
            .ctx
            .closures()
            .resource(&OwnerScope::seeing(peer), id)
            .await?
            .filter(|(root, _)| root.kind() == ResourceKind::Dataset)
            .ok_or_else(|| Self::not_found(id))?;
        let dataset = adapter.project(&set, &root)?;
        self.ctx.cache.put_projection(&key, &dataset).await;
        Ok(dataset)
    }
}
