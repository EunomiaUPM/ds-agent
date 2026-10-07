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

use axum::http::StatusCode;
use catalog_agent::OdrlPolicyDto;
use common::config::types::min_known_config::MinKnownConfig;
use common::config::types::traits::MinKnownConfigTrait;
use urn::Urn;
use ymir::config::types::HostType;
use ymir::errors::{Errors, Outcome, PetitionFailure};
use ymir::services::client::ClientExt;
use ymir::utils::http_client;

use crate::facades::catalog_facade::CatalogFacadeTrait;

/// Offers read from the catalog agent's API.
///
/// The calls carry no user token yet: the catalog agent answers for
/// whoever its identity provider gives. Propagating the caller is pending (as for the auth
/// facades).
pub struct CatalogRemoteFacade {
    offers_url: String,
}

impl CatalogRemoteFacade {
    pub fn new(catalog: &MinKnownConfig) -> Self {
        Self {
            offers_url: format!(
                "{}{}/{}/odrl-policies",
                catalog.get_host(HostType::Http),
                catalog.get_api_version(),
                catalog_agent::SERVICE_NAME
            ),
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
        fields(peer.service = "catalog")
    )]
    async fn get_offer(&self, offer_id: &Urn) -> Outcome<OdrlPolicyDto> {
        let url = format!("{}/{offer_id}", self.offers_url);
        match http_client().get_json(&url, None).await {
            Err(Errors::PetitionError {
                failure: PetitionFailure::HttpStatus(StatusCode::NOT_FOUND),
                ..
            }) => Err(Errors::missing_resource(
                offer_id.to_string(),
                "Offer not found in the catalog",
                None,
            )),
            other => other,
        }
    }
}
