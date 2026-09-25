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
use common::auth::ServiceHttpClient;
use common::config::types::min_known_config::MinKnownConfig;
use common::config::types::traits::MinKnownConfigTrait;
use urn::Urn;
use ymir::config::types::HostType;
use ymir::errors::{Errors, Outcome, PetitionFailure};

use crate::entities::connector_instance::ConnectorInstanceDto;
use crate::facades::connector_instance_facade::ConnectorInstanceFacadeTrait;

/// Instances read from the catalog agent's connector API with the service token.
pub struct ConnectorInstanceRemoteFacade {
    instances_url: String,
    service_client: Arc<ServiceHttpClient>,
}

impl ConnectorInstanceRemoteFacade {
    /// `catalog` locates the agent hosting the connector.
    pub fn new(catalog: &MinKnownConfig, service_client: Arc<ServiceHttpClient>) -> Self {
        Self {
            instances_url: format!(
                "{}{}/connector/instances",
                catalog.get_host(HostType::Http),
                catalog.get_api_version()
            ),
            service_client,
        }
    }

    /// A 404 is an absent instance, as the local service reports it.
    async fn fetch(&self, url: &str, tenant_id: &str) -> Outcome<Option<ConnectorInstanceDto>> {
        match self.service_client.get_json(url, Some(tenant_id)).await {
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
        fields(peer.service = "connector", tenant = %tenant_id)
    )]
    async fn get_instance_by_id(
        &self,
        tenant_id: &str,
        id: &Urn,
    ) -> Outcome<Option<ConnectorInstanceDto>> {
        self.fetch(&format!("{}/{id}", self.instances_url), tenant_id)
            .await
    }

    #[tracing::instrument(
        level = "info",
        skip_all,
        err,
        fields(peer.service = "connector", tenant = %tenant_id)
    )]
    async fn get_instance_by_distribution(
        &self,
        tenant_id: &str,
        distribution_id: &Urn,
    ) -> Outcome<Option<ConnectorInstanceDto>> {
        let url = format!("{}/distribution/{distribution_id}", self.instances_url);
        self.fetch(&url, tenant_id).await
    }
}
