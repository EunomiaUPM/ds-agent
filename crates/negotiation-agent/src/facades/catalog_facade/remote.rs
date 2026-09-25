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

use axum::http::StatusCode;
use catalog_agent::OdrlPolicyDto;
use common::auth::ServiceHttpClient;
use common::config::types::min_known_config::MinKnownConfig;
use common::config::types::traits::MinKnownConfigTrait;
use urn::Urn;
use ymir::config::types::HostType;
use ymir::errors::{Errors, Outcome, PetitionFailure};

use crate::facades::catalog_facade::CatalogFacadeTrait;

/// Offers read from the catalog agent's API with the service token.
pub struct CatalogRemoteFacade {
    offers_url: String,
    service_client: Arc<ServiceHttpClient>,
}

impl CatalogRemoteFacade {
    pub fn new(catalog: &MinKnownConfig, service_client: Arc<ServiceHttpClient>) -> Self {
        Self {
            offers_url: format!(
                "{}{}/{}/odrl-policies",
                catalog.get_host(HostType::Http),
                catalog.get_api_version(),
                catalog_agent::SERVICE_NAME
            ),
            service_client,
        }
    }
}

#[async_trait::async_trait]
impl CatalogFacadeTrait for CatalogRemoteFacade {
    /// A 404 becomes the same missing-resource error the local service returns.
    #[tracing::instrument(
        level = "info",
        skip_all,
        err,
        fields(peer.service = "catalog", tenant = %tenant_id)
    )]
    async fn get_offer(&self, tenant_id: &str, offer_id: &Urn) -> Outcome<OdrlPolicyDto> {
        let url = format!("{}/{offer_id}", self.offers_url);
        match self.service_client.get_json(&url, Some(tenant_id)).await {
            Err(Errors::PetitionError {
                failure: PetitionFailure::HttpStatus(StatusCode::NOT_FOUND),
                ..
            }) => Err(Errors::missing_resource(
                offer_id.to_string(),
                "Offer not found in the tenant catalog",
                None,
            )),
            other => other,
        }
    }
}
