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

use async_trait::async_trait;
use ymir::config::types::HostType;
use ymir::errors::Outcome;

use super::MatesFacadeTrait;
use crate::auth::ServiceHttpClient;
use crate::config::types::min_known_config::MinKnownConfig;
use crate::config::types::traits::MinKnownConfigTrait;
use crate::paginated_spec::Paginated;
use ymir::data::entities::shared::participant::Model as Mates;

pub struct MatesFacadeService {
    config: Arc<MinKnownConfig>,
    client: Arc<ServiceHttpClient>,
}

impl MatesFacadeService {
    pub fn new(config: Arc<MinKnownConfig>, client: Arc<ServiceHttpClient>) -> Self {
        Self { config, client }
    }

    fn base_url(&self) -> String {
        format!("{}/api/v1", self.config.get_host(HostType::Http))
    }
}

#[async_trait]
impl MatesFacadeTrait for MatesFacadeService {
    async fn get_mate_by_id(&self, tenant_id: String, mate_id: String) -> Outcome<Mates> {
        let url = format!("{}/mates/{}", self.base_url(), mate_id);
        self.client.get_json(&url, Some(&tenant_id)).await
    }

    async fn get_me_mate(&self, tenant_id: String) -> Outcome<Mates> {
        let url = format!("{}/mates/myself", self.base_url());
        self.client.get_json(&url, Some(&tenant_id)).await
    }

    async fn get_all_mates(&self, tenant_id: String) -> Outcome<Vec<Mates>> {
        let url = format!("{}/mates/all", self.base_url());
        let page: Paginated<Mates> = self.client.get_json(&url, Some(&tenant_id)).await?;
        Ok(page.items)
    }
}
