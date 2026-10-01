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

//! Service-to-service calls through the shared ymir client, authenticated with a cached
//! client_credentials token.

use std::time::{Duration, Instant};

use axum::http::{HeaderMap, HeaderValue};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use ymir::config::traits::HostsConfigTrait;
use ymir::config::types::HostType;
use ymir::errors::{Errors, Outcome, PetitionFailure};
use ymir::services::client::ClientExt;
use ymir::types::http::{HttpBody, StatusCode};
use ymir::utils::{bearer_headers, http_client};

use crate::auth::TENANT_HEADER;
use crate::config::services::CommonConfig;
use crate::config::types::ServiceClientConfig;

/// Renews the cached token this long before it expires.
const TOKEN_RENEWAL_MARGIN: Duration = Duration::from_secs(30);

#[derive(Deserialize)]
struct ClientCredentialsResponse {
    access_token: String,
    expires_in: u64,
}

pub struct ServiceHttpClient {
    token_url: String,
    client_id: String,
    client_secret: String,
    token: RwLock<Option<(String, Instant)>>,
}

impl ServiceHttpClient {
    /// `own_host` is the caller's HTTP host, used when the config names no token endpoint.
    pub fn new(config: &ServiceClientConfig, own_host: &str) -> Self {
        Self {
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
    pub fn from_common(common: &CommonConfig) -> Self {
        Self::new(&common.service_client, &common.get_host(HostType::Http))
    }

    /// GET acting on `tenant`; `None` leaves the tenant to the service token.
    pub async fn get_json<R: DeserializeOwned + Send>(
        &self,
        url: &str,
        tenant: Option<&str>,
    ) -> Outcome<R> {
        let headers = self.headers(tenant).await?;
        self.forget_on_unauthorized(http_client().get_json(url, Some(headers)).await)
            .await
    }

    pub async fn post_json<T, R>(&self, url: &str, tenant: Option<&str>, body: &T) -> Outcome<R>
    where
        T: Serialize + Sync,
        R: DeserializeOwned + Send,
    {
        let headers = self.headers(tenant).await?;
        self.forget_on_unauthorized(http_client().post_json(url, Some(headers), body).await)
            .await
    }

    /// POST whose response body is ignored, for endpoints that may answer with an empty 2xx.
    pub async fn post<T: Serialize + Sync>(
        &self,
        url: &str,
        tenant: Option<&str>,
        body: &T,
    ) -> Outcome<()> {
        let headers = self.headers(tenant).await?;
        let sent = http_client()
            .post_ok(url, Some(headers), HttpBody::json(body)?)
            .await;
        self.forget_on_unauthorized(sent).await
    }

    async fn headers(&self, tenant: Option<&str>) -> Outcome<HeaderMap> {
        let mut headers = bearer_headers(&self.bearer().await?)?;
        if let Some(tenant) = tenant {
            let value = HeaderValue::from_str(tenant).map_err(|e| {
                Errors::parse("Tenant is not a valid header value", Some(Box::new(e)))
            })?;
            headers.insert(TENANT_HEADER, value);
        }
        Ok(headers)
    }

    /// A 401 means the cached token was revoked or rotated: drop it so the next call renews.
    async fn forget_on_unauthorized<R>(&self, outcome: Outcome<R>) -> Outcome<R> {
        if let Err(Errors::PetitionError {
            failure: PetitionFailure::HttpStatus(StatusCode::UNAUTHORIZED),
            ..
        }) = &outcome
        {
            self.token.write().await.take();
        }
        outcome
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
        let issued: ClientCredentialsResponse = http_client()
            .post_json(&self.token_url, None, &body)
            .await
            .map_err(|e| {
                Errors::unauthorized(format!("service token request failed: {e}"), None)
            })?;
        let lifetime = Duration::from_secs(issued.expires_in).saturating_sub(TOKEN_RENEWAL_MARGIN);
        *self.token.write().await = Some((issued.access_token.clone(), Instant::now() + lifetime));
        Ok(issued.access_token)
    }
}
