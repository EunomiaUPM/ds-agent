/*
 *
 *  * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
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

use crate::subscriptions::MicroserviceSubscriptionKey;
use common::config::services::traits::GatewayConfigTrait;
use common::config::services::GatewayConfig;
use common::config::types::traits::{CommonConfigTrait, MinKnownConfigTrait};
use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use tracing::{debug, error};
use ymir::config::traits::HostsConfigTrait;
use ymir::config::types::HostType;
use ymir::errors::{Errors, Outcome};

pub struct GatewaySubscriptions {
    config: GatewayConfig,
    client: Client,
}

impl GatewaySubscriptions {
    pub fn new(config: GatewayConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to build reqwest client");
        Self { config, client }
    }
    pub async fn subscribe_to_microservice(
        &self,
        microservice_key_name: MicroserviceSubscriptionKey,
    ) -> Outcome<()> {
        let is_datahub = self.config.is_catalog_datahub();
        let microservice_url = match microservice_key_name {
            MicroserviceSubscriptionKey::Catalog => match is_datahub {
                true => self.config.contracts().get_host(HostType::Http),
                false => self.config.catalog().get_host(HostType::Http),
            },
        };
        let microservice_url = microservice_url.trim_end_matches("/");
        let microservice_tag = match microservice_key_name {
            MicroserviceSubscriptionKey::Catalog => match is_datahub {
                true => "contract-negotiation",
                false => "catalog",
            },
        };
        let subscription_base = format!("/api/v1/{}/subscriptions", microservice_tag);
        let subscription_url = format!("{}{}", microservice_url, subscription_base);
        debug!(subscription_url);

        let notification_gateway_endpoint = "/incoming-notification";
        let notification_gateway_url = format!(
            "{}{}",
            self.config.common().get_host(HostType::Http),
            notification_gateway_endpoint
        );

        let request = self
            .client
            .post(&subscription_url)
            .json(&json!({
                "callbackAddress": notification_gateway_url
            }))
            .send()
            .await;
        let request = match request {
            Ok(request) => request,
            Err(e) => {
                error!("Error on subscribing. Microservice not available{}", e);
                return Err(Errors::parse(
                    &format!("Error on subscribing. Microservice not available {}", e),
                    None,
                ));
            }
        };
        if !request.status().is_success() {
            return Err(Errors::parse(
                &format!("Error on subscribing. Status {}", request.status()),
                None,
            ));
        }
        Ok(())
    }
}
