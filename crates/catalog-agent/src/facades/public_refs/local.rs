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

//! Catalog resources in a profile, their ODRL offers and hosted schemas, as anyone dereferences
//! them.

use std::sync::Arc;

use common::public_refs::{PublicDocument, PublicQuery, PublicResolverTrait};
use urn::Urn;
use ymir::config::traits::{ApiConfigTrait, CommonConfigTrait};
use ymir::errors::Outcome;
use ymir::json_schema::JSON_SCHEMA_MEDIA_TYPE;

use crate::profiles::dcat3::Dcat3;
use crate::profiles::shared::dcat::DcatProfileSpec;
use crate::setup::context::AppContext;
use crate::SERVICE_NAME;

/// Resolves the URNs the catalog holds: resources not private nor drafts, their offers, schemas.
pub struct CatalogPublicResolver {
    ctx: Arc<AppContext>,
}

impl CatalogPublicResolver {
    const DEFAULT_PROFILE: &'static str = Dcat3::KEY;
    const SCHEMA_NID: &'static str = "schema";

    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self { ctx }
    }

    async fn schema(&self, urn: &Urn, version: Option<i32>) -> Outcome<Option<PublicDocument>> {
        Ok(self
            .ctx
            .schemas()
            .public(urn.as_str(), version)
            .await?
            .map(|(body, url)| {
                PublicDocument::json_ld(body)
                    .with_media_type(JSON_SCHEMA_MEDIA_TYPE)
                    .with_location(url)
            }))
    }

    /// Offers have no profile of their own: they are ODRL in the DSP context.
    async fn offer(&self, urn: &Urn) -> Outcome<Option<PublicDocument>> {
        Ok(self
            .ctx
            .offers()
            .public(urn)
            .await?
            .map(PublicDocument::json_ld))
    }

    async fn resource(&self, urn: &Urn, profile: &str) -> Outcome<Option<PublicDocument>> {
        let Some(body) = self.ctx.projections().public(profile, urn).await? else {
            return Ok(None);
        };
        let mut document = PublicDocument::json_ld(body).with_location(format!(
            "{}/{SERVICE_NAME}/public/{profile}/resources/{urn}",
            self.ctx.config.common().get_api_version()
        ));
        if let Some(adapter) = self.ctx.profiles.get(profile) {
            document = document.with_profile(adapter.profile_iri());
        }
        Ok(Some(document))
    }
}

#[async_trait::async_trait]
impl PublicResolverTrait for CatalogPublicResolver {
    async fn resolve(&self, urn: &Urn, query: &PublicQuery) -> Outcome<Option<PublicDocument>> {
        if urn.nid() == Self::SCHEMA_NID {
            return self.schema(urn, query.version).await;
        }
        let profile = query.profile.as_deref().unwrap_or(Self::DEFAULT_PROFILE);
        if let Some(document) = self.resource(urn, profile).await? {
            return Ok(Some(document));
        }
        self.offer(urn).await
    }
}
