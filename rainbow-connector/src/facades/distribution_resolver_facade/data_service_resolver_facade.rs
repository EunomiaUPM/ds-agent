/*
 *
 *  * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
 *  *
 *  * This program is free software: you can redistribute it and/or modify
 *  * it under the terms of the GNU General Public License as published by
 *  * the Free Software Foundation, either version 3 of the License, or
 *  * (at your option) any later version.
 *  *
 *  * This program is distributed in the hope that it will be useful,
 *  * but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  * GNU General Public License for more details.
 *  *
 *  * You should have received a copy of the GNU General Public License
 *  * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use crate::facades::distribution_resolver_facade::{Distribution, DistributionFacadeTrait};
use rainbow_common::config::types::traits::CommonConfigTrait;
use rainbow_common::http_client::HttpClient;
use std::sync::Arc;
use ymir::config::traits::HostsConfigTrait;
use ymir::config::types::HostType;

pub struct DistributionFacadeServiceForConnector {
    catalog_base_url: String,
    client: Arc<HttpClient>,
}

impl DistributionFacadeServiceForConnector {
    pub fn new(config: &dyn CommonConfigTrait, client: Arc<HttpClient>) -> Self {
        let catalog_base_url = config.common().get_host(HostType::Http);
        Self { catalog_base_url, client }
    }
}

#[async_trait::async_trait]
impl DistributionFacadeTrait for DistributionFacadeServiceForConnector {
    async fn resolve_distribution_by_id(
        &self,
        distribution_id: &String,
    ) -> anyhow::Result<Distribution> {
        let distribution_url = format!(
            "{}/api/v1/catalog-agent/distributions/{}",
            self.catalog_base_url, distribution_id
        );
        let distribution = self.client.get_json::<Distribution>(distribution_url.as_str()).await?;
        Ok(distribution)
    }
}
