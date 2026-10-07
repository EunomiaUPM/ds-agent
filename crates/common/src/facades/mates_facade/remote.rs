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

//! Adapter that reads participants from the auth agent over HTTP.

use std::sync::Arc;

use async_trait::async_trait;
use ymir::config::types::HostType;
use ymir::errors::Outcome;
use ymir::http::routes::fill;
use ymir::services::client::ClientExt;
use ymir::types::oauth::UserInfo;
use ymir::utils::{encode_url_safe_no_pad, http_client};

use crate::config::types::min_known_config::MinKnownConfig;
use crate::config::types::traits::MinKnownConfigTrait;
use crate::facades::mates_facade::MatesFacadeTrait;
use crate::paginated_spec::Paginated;
use crate::routes::auth::mates;
use ymir::data::entities::shared::participant::Model as Mates;

/// Participants read from the auth agent's `/mates` API ([`mates`]).
///
/// The calls carry no user token yet: the auth agent filters for whoever its identity provider
/// gives (the fixed user with `static`; a 401 with `keycloak`). Propagating the caller is pending.
pub struct MatesRemoteFacade {
    config: Arc<MinKnownConfig>,
}

impl MatesRemoteFacade {
    pub fn new(config: Arc<MinKnownConfig>) -> Self {
        Self { config }
    }

    /// The auth agent's `/mates` API, under its API version.
    fn base_url(&self) -> String {
        format!(
            "{}{}{}",
            self.config.get_host(HostType::Http),
            self.config.get_api_version(),
            mates::PREFIX
        )
    }
}

#[async_trait]
impl MatesFacadeTrait for MatesRemoteFacade {
    /// `mate_id` is the plain DID; it travels in base64url, as the auth agent expects it in the
    /// path.
    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "auth"))]
    async fn get_mate_by_id(&self, _user: &UserInfo, mate_id: String) -> Outcome<Mates> {
        let segment = encode_url_safe_no_pad(&mate_id);
        let url = format!("{}{}", self.base_url(), fill(mates::BY_ID, &segment));
        http_client().get_json(&url, None).await
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "auth"))]
    async fn get_me_mate(&self) -> Outcome<Mates> {
        let url = format!("{}{}", self.base_url(), mates::MYSELF);
        http_client().get_json(&url, None).await
    }

    #[tracing::instrument(level = "info", skip_all, err, fields(peer.service = "auth"))]
    async fn get_all_mates(&self, _user: &UserInfo) -> Outcome<Vec<Mates>> {
        let url = format!("{}{}", self.base_url(), mates::ALL);
        let page: Paginated<Mates> = http_client().get_json(&url, None).await?;
        Ok(page.items)
    }
}
