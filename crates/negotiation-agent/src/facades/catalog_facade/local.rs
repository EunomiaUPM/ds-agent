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

use catalog_agent::OdrlPolicyDto;
use catalog_agent::services::odrl_policies::OdrlPolicyServiceTrait;
use common::auth::AccessScope;
use urn::Urn;
use ymir::errors::{Errors, Outcome};

use crate::facades::catalog_facade::CatalogFacadeTrait;

/// Offers read straight from the catalog service, when it shares the process.
pub struct CatalogLocalFacade {
    offers: Arc<dyn OdrlPolicyServiceTrait>,
}

impl CatalogLocalFacade {
    pub fn new(offers: Arc<dyn OdrlPolicyServiceTrait>) -> Self {
        Self { offers }
    }
}

#[async_trait::async_trait]
impl CatalogFacadeTrait for CatalogLocalFacade {
    #[tracing::instrument(
        level = "info",
        skip_all,
        err,
        fields(peer.service = "catalog", tenant = %tenant_id)
    )]
    async fn get_offer(&self, tenant_id: &str, offer_id: &Urn) -> Outcome<OdrlPolicyDto> {
        match self
            .offers
            .get_odrl_offer_by_id(&AccessScope::service(tenant_id), offer_id)
            .await
        {
            Err(Errors::MissingResourceError { .. }) => Err(Errors::missing_resource(
                offer_id.to_string(),
                "Offer not found in the tenant catalog",
                None,
            )),
            other => other,
        }
    }
}
