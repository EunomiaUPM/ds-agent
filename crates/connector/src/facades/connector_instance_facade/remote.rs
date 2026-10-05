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

//! HTTP adapter for agents in another process.

use axum::http::StatusCode;
use common::config::types::min_known_config::MinKnownConfig;
use common::config::types::traits::MinKnownConfigTrait;
use urn::Urn;
use ymir::config::types::HostType;
use ymir::errors::{Errors, Outcome, PetitionFailure};
use ymir::services::client::ClientExt;
use ymir::utils::http_client;

use crate::entities::connector_instance::ConnectorInstanceDto;
use crate::facades::connector_instance_facade::ConnectorInstanceFacadeTrait;

/// Instances read from the catalog agent's connector API.
///
/// The calls carry no user token yet: the catalog agent answers for whoever its identity
/// provider gives. Propagating the caller is pending (as for the auth facades).
pub struct ConnectorInstanceRemoteFacade {
    instances_url: String,
}

impl ConnectorInstanceRemoteFacade {
    /// `catalog` locates the agent hosting the connector.
    pub fn new(catalog: &MinKnownConfig) -> Self {
        Self {
            instances_url: format!(
                "{}{}/connector/instances",
                catalog.get_host(HostType::Http),
                catalog.get_api_version()
            ),
        }
    }

    /// A 404 is an absent instance, as the local service reports it.
    async fn fetch(&self, url: &str) -> Outcome<Option<ConnectorInstanceDto>> {
        match http_client().get_json(url, None).await {
            Ok(instance) => Ok(Some(instance)),
            Err(Errors::PetitionError {
                failure: PetitionFailure::HttpStatus(StatusCode::NOT_FOUND),
                ..
            }) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

#[async_trait::async_trait]
impl ConnectorInstanceFacadeTrait for ConnectorInstanceRemoteFacade {
    #[tracing::instrument(
        level = "info",
        skip_all,
        err,
        fields(peer.service = "connector")
    )]
    async fn get_instance_by_id(&self, id: &Urn) -> Outcome<Option<ConnectorInstanceDto>> {
        self.fetch(&format!("{}/{id}", self.instances_url)).await
    }

    #[tracing::instrument(
        level = "info",
        skip_all,
        err,
        fields(peer.service = "connector")
    )]
    async fn get_instance_by_distribution(
        &self,
        distribution_id: &Urn,
    ) -> Outcome<Option<ConnectorInstanceDto>> {
        let url = format!("{}/distribution/{distribution_id}", self.instances_url);
        self.fetch(&url).await
    }
}
