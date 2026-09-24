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

//! Monolith-only boot seeders.

use common::boot::seeders::BootSeeder;
use common::config::services::CommonConfig;
use common::http_client::{HttpClient, HttpClientError};
use ymir::config::traits::{ApiConfigTrait, HostsConfigTrait};
use ymir::config::types::HostType;
use ymir::data::entities::shared::participant;
use ymir::errors::{Errors, Outcome};

/// Links the agent's own wallet as a participant when the auth plane does not know it yet.
pub struct SelfParticipantOnboarder {
    common: CommonConfig,
}

impl SelfParticipantOnboarder {
    pub fn new(common: CommonConfig) -> Self {
        Self { common }
    }

    /// HTTP client authenticated as the seeded admin (password grant).
    async fn admin_client(&self) -> Outcome<HttpClient> {
        let client = HttpClient::new(1, 30);
        let admin = &self.common.admin_seed;
        let url = format!("{}/oauth/token", self.common.get_host(HostType::Http));
        let token = client
            .post_json::<serde_json::Value, serde_json::Value>(
                &url,
                &serde_json::json!({
                    "grant_type": "password",
                    "username": admin.email,
                    "password": admin.password,
                }),
            )
            .await?;
        let access_token = token
            .get("access_token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Errors::parse("Token response without access_token", None))?;
        client.set_auth_token(access_token.to_string()).await;
        Ok(client)
    }
}

#[async_trait::async_trait]
impl BootSeeder for SelfParticipantOnboarder {
    fn name(&self) -> &'static str {
        "self-participant"
    }

    async fn seed(&self) -> Outcome<()> {
        let client = self.admin_client().await?;
        let base = format!(
            "{}{}",
            self.common.get_host(HostType::Http),
            self.common.get_api_version()
        )
        .replace("host.docker.internal", "127.0.0.1");
        let myself = format!("{base}/mates/myself");
        let participant = match client.get_json::<participant::Model>(&myself).await {
            Ok(participant) => participant,
            Err(HttpClientError::HttpError { status, .. }) if status.as_u16() == 404 => {
                client
                    .post_void::<()>(&format!("{base}/wallet/link"))
                    .await?;
                client.get_json::<participant::Model>(&myself).await?
            }
            Err(e) => return Err(e.into()),
        };
        tracing::info!(
            participant = participant.participant_id,
            "Self participant ready"
        );
        Ok(())
    }
}
