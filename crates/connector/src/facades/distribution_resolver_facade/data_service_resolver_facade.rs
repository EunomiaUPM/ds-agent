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

use crate::facades::distribution_resolver_facade::DistributionFacadeTrait;
use common::auth::ServiceHttpClient;
use common::config::types::traits::CommonConfigTrait;
use serde_json::Value;
use std::sync::Arc;
use ymir::config::traits::{ApiConfigTrait, HostsConfigTrait};
use ymir::config::types::HostType;
use axum::http::StatusCode;
use ymir::errors::{Errors, Outcome, PetitionFailure};

/// Checks distributions against the catalog API, authenticated with the service token.
pub struct DistributionFacadeServiceForConnector {
    distributions_url: String,
    service_client: Arc<ServiceHttpClient>,
}

impl DistributionFacadeServiceForConnector {
    pub fn new(config: &dyn CommonConfigTrait, service_client: Arc<ServiceHttpClient>) -> Self {
        let common = config.common();
        Self {
            distributions_url: format!(
                "{}{}/catalog-agent/distributions",
                common.get_host(HostType::Http),
                common.get_api_version()
            ),
            service_client,
        }
    }
}

#[async_trait::async_trait]
impl DistributionFacadeTrait for DistributionFacadeServiceForConnector {
    async fn resolve_distribution_by_id(
        &self,
        tenant_id: &str,
        distribution_id: &str,
    ) -> Outcome<()> {
        let url = format!("{}/{distribution_id}", self.distributions_url);
        match self
            .service_client
            .get_json::<Value>(&url, Some(tenant_id))
            .await
        {
            Ok(_) => Ok(()),
            Err(Errors::PetitionError {
                failure: PetitionFailure::HttpStatus(StatusCode::NOT_FOUND),
                ..
            }) => Err(Errors::missing_resource(
                distribution_id,
                "Distribution not found in the tenant catalog",
                None,
            )),
            Err(e) => Err(e),
        }
    }
}
