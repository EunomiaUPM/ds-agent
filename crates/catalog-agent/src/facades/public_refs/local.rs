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

//! The hosted schemas a published resource uses, as anyone dereferences them; catalog
//! resources and their offers go through DSP.

use std::sync::Arc;

use common::public_refs::{PublicDocument, PublicQuery, PublicResolverTrait};
use urn::Urn;
use ymir::errors::Outcome;
use ymir::json_schema::JSON_SCHEMA_MEDIA_TYPE;

use crate::setup::context::AppContext;

/// Resolves the catalog's schema URNs.
pub struct CatalogPublicResolver {
    ctx: Arc<AppContext>,
}

impl CatalogPublicResolver {
    const SCHEMA_NID: &'static str = "schema";

    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self { ctx }
    }
}

#[async_trait::async_trait]
impl PublicResolverTrait for CatalogPublicResolver {
    async fn resolve(&self, urn: &Urn, query: &PublicQuery) -> Outcome<Option<PublicDocument>> {
        if urn.nid() != Self::SCHEMA_NID {
            return Ok(None);
        }
        Ok(self
            .ctx
            .schemas()
            .public(urn.as_str(), query.version)
            .await?
            .map(|(body, url)| {
                PublicDocument::json_ld(body)
                    .with_media_type(JSON_SCHEMA_MEDIA_TYPE)
                    .with_location(url)
            }))
    }
}
