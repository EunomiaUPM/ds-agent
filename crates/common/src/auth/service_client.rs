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

//! HTTP client for service-to-service calls, authenticated with a client_credentials token.

use std::time::{Duration, Instant};

use reqwest::RequestBuilder;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use ymir::config::traits::HostsConfigTrait;
use ymir::config::types::HostType;
use ymir::errors::{Errors, Outcome};

use crate::auth::TENANT_HEADER;
use crate::config::services::CommonConfig;
use crate::config::types::ServiceClientConfig;
use crate::http_client::HttpClientError;

/// Renews the cached token this long before it expires.
const TOKEN_RENEWAL_MARGIN: Duration = Duration::from_secs(30);

#[derive(Deserialize)]
struct ClientCredentialsResponse {
    access_token: String,
    expires_in: u64,
}

pub struct ServiceHttpClient {
    http: reqwest::Client,
    token_url: String,
    client_id: String,
    client_secret: String,
    token: RwLock<Option<(String, Instant)>>,
}

impl ServiceHttpClient {
    /// `own_host` is the caller's HTTP host, used when the config names no token endpoint.
    pub fn new(config: &ServiceClientConfig, own_host: &str, timeout_secs: u64) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .expect("Failed to build reqwest client");
        Self {
            http,
            token_url: config
                .token_url
                .clone()
                .unwrap_or_else(|| format!("{own_host}/oauth/token")),
            client_id: config.client_id.clone(),
            client_secret: config.client_secret.clone(),
            token: RwLock::new(None),
        }
    }

    /// Client for an agent, with the credentials and host of its own common config.
    pub fn from_common(common: &CommonConfig, timeout_secs: u64) -> Self {
        Self::new(
            &common.service_client,
            &common.get_host(HostType::Http),
            timeout_secs,
        )
    }

    /// GET acting on `tenant`; `None` leaves the tenant to the service token.
    pub async fn get_json<R: DeserializeOwned>(
        &self,
        url: &str,
        tenant: Option<&str>,
    ) -> Outcome<R> {
        self.send(self.http.get(url), tenant).await
    }

    pub async fn post_json<T, R>(&self, url: &str, tenant: Option<&str>, body: &T) -> Outcome<R>
    where
        T: Serialize + Sync,
        R: DeserializeOwned,
    {
        self.send(self.http.post(url).json(body), tenant).await
    }

    async fn send<R: DeserializeOwned>(
        &self,
        builder: RequestBuilder,
        tenant: Option<&str>,
    ) -> Outcome<R> {
        let mut builder = builder.bearer_auth(self.bearer().await?);
        if let Some(tenant) = tenant {
            builder = builder.header(TENANT_HEADER, tenant);
        }
        let response = builder.send().await.map_err(HttpClientError::from)?;
        let status = response.status();
        if !status.is_success() {
            if status == reqwest::StatusCode::UNAUTHORIZED {
                self.token.write().await.take();
            }
            let message = response.text().await.unwrap_or_default();
            return Err(HttpClientError::HttpError { status, message }.into());
        }
        Ok(response.json::<R>().await.map_err(HttpClientError::from)?)
    }

    /// Cached service token, renewed through the client_credentials grant when close to expiry.
    async fn bearer(&self) -> Outcome<String> {
        if let Some((token, renew_at)) = self.token.read().await.as_ref() {
            if Instant::now() < *renew_at {
                return Ok(token.clone());
            }
        }
        let body = serde_json::json!({
            "grant_type": "client_credentials",
            "client_id": self.client_id,
            "client_secret": self.client_secret,
        });
        let response = self
            .http
            .post(&self.token_url)
            .json(&body)
            .send()
            .await
            .map_err(HttpClientError::from)?;
        let status = response.status();
        if !status.is_success() {
            let message = response.text().await.unwrap_or_default();
            return Err(Errors::unauthorized(
                format!("service token request failed ({status}): {message}"),
                None,
            ));
        }
        let issued: ClientCredentialsResponse =
            response.json().await.map_err(HttpClientError::from)?;
        let lifetime = Duration::from_secs(issued.expires_in).saturating_sub(TOKEN_RENEWAL_MARGIN);
        *self.token.write().await = Some((issued.access_token.clone(), Instant::now() + lifetime));
        Ok(issued.access_token)
    }
}
