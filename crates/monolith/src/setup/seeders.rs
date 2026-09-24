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

use axum::http::{HeaderMap, StatusCode};
use common::boot::seeders::BootSeeder;
use common::config::services::CommonConfig;
use ymir::config::traits::{ApiConfigTrait, HostsConfigTrait};
use ymir::config::types::HostType;
use ymir::data::entities::shared::participant;
use ymir::errors::{Errors, Outcome, PetitionFailure};
use ymir::services::client::ClientExt;
use ymir::types::http::HttpBody;
use ymir::utils::{bearer_headers, http_client};

/// Links the agent's own wallet as a participant when the auth plane does not know it yet.
pub struct SelfParticipantOnboarder {
    common: CommonConfig,
}

impl SelfParticipantOnboarder {
    pub fn new(common: CommonConfig) -> Self {
        Self { common }
    }

    /// Request headers authenticated as the seeded admin (password grant).
    async fn admin_headers(&self) -> Outcome<HeaderMap> {
        let admin = &self.common.admin_seed;
        let url = format!("{}/oauth/token", self.common.get_host(HostType::Http));
        let token = http_client()
            .post_json::<serde_json::Value, serde_json::Value>(
                &url,
                None,
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
        bearer_headers(access_token)
    }
}

#[async_trait::async_trait]
impl BootSeeder for SelfParticipantOnboarder {
    fn name(&self) -> &'static str {
        "self-participant"
    }

    async fn seed(&self) -> Outcome<()> {
        let headers = self.admin_headers().await?;
        let base = format!(
            "{}{}",
            self.common.get_host(HostType::Http),
            self.common.get_api_version()
        )
        .replace("host.docker.internal", "127.0.0.1");
        let myself = format!("{base}/mates/myself");
        let client = http_client();
        let participant = match client
            .get_json::<participant::Model>(&myself, Some(headers.clone()))
            .await
        {
            Ok(participant) => participant,
            Err(Errors::PetitionError {
                failure: PetitionFailure::HttpStatus(StatusCode::NOT_FOUND),
                ..
            }) => {
                client
                    .post_ok(
                        &format!("{base}/wallet/link"),
                        Some(headers.clone()),
                        HttpBody::None,
                    )
                    .await?;
                client
                    .get_json::<participant::Model>(&myself, Some(headers))
                    .await?
            }
            Err(e) => return Err(e),
        };
        tracing::info!(
            participant = participant.participant_id,
            "Self participant ready"
        );
        Ok(())
    }
}
