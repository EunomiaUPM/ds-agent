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

use common::config::types::min_known_config::MinKnownConfig;
use common::config::types::traits::MinKnownConfigTrait;
use negotiation_agent::AgreementView;
use urn::Urn;
use ymir::config::types::HostType;
use ymir::services::client::ClientExt;
use ymir::utils::http_client;
use ymir::errors::Outcome;

use crate::protocols::dsp::facades::negotiation_facade::NegotiationFacadeTrait;

/// Agreements read from the negotiation agent's API.
pub struct NegotiationRemoteFacade {
    agreements_url: String,
}

impl NegotiationRemoteFacade {
    pub fn new(contracts: &MinKnownConfig) -> Self {
        Self {
            agreements_url: format!(
                "{}{}/{}/agreements",
                contracts.get_host(HostType::Http),
                contracts.get_api_version(),
                negotiation_agent::SERVICE_NAME
            ),
        }
    }
}

#[async_trait::async_trait]
impl NegotiationFacadeTrait for NegotiationRemoteFacade {
        #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "negotiation"))]
    async fn get_agreement(&self, agreement_id: &Urn) -> Outcome<AgreementView> {
        let url = format!("{}/{agreement_id}", self.agreements_url);
        http_client().get_json(&url, None).await
    }
}
