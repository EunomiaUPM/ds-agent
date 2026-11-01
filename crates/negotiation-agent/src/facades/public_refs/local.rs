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

//! Agreements as anyone dereferences them: always public, whatever their visibility.

use std::sync::Arc;

use common::public_refs::{PublicDocument, PublicQuery, PublicResolverTrait};
use urn::Urn;
use ymir::errors::Outcome;

use crate::services::agreement::AgreementServiceTrait;

/// Resolves the URNs of the agreements this agent stores.
pub struct NegotiationPublicResolver {
    agreements: Arc<dyn AgreementServiceTrait>,
}

impl NegotiationPublicResolver {
    pub fn new(agreements: Arc<dyn AgreementServiceTrait>) -> Self {
        Self { agreements }
    }
}

#[async_trait::async_trait]
impl PublicResolverTrait for NegotiationPublicResolver {
    async fn resolve(&self, urn: &Urn, _query: &PublicQuery) -> Outcome<Option<PublicDocument>> {
        Ok(self
            .agreements
            .public(urn)
            .await?
            .map(PublicDocument::json_ld))
    }
}
