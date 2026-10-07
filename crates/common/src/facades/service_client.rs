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

//! Service-to-service calls through the shared ymir client.

use serde::Serialize;
use serde::de::DeserializeOwned;
use ymir::errors::Outcome;
use ymir::services::client::ClientExt;
use ymir::types::http::HttpBody;
use ymir::utils::http_client;

/// HTTP client for calls between agents. Calls go without token: agents trust their internal
/// network, and carrying the user across agents is up to each remote facade.
#[derive(Default)]
pub struct ServiceHttpClient;

impl ServiceHttpClient {
    pub fn new() -> Self {
        Self
    }

    /// GET decoding the JSON response.
    pub async fn get_json<R: DeserializeOwned + Send>(&self, url: &str) -> Outcome<R> {
        http_client().get_json(url, None).await
    }

    /// POST decoding the JSON response.
    pub async fn post_json<T, R>(&self, url: &str, body: &T) -> Outcome<R>
    where
        T: Serialize + Sync,
        R: DeserializeOwned + Send,
    {
        http_client().post_json(url, None, body).await
    }

    /// POST whose response body is ignored, for endpoints that may answer with an empty 2xx.
    pub async fn post<T: Serialize + Sync>(&self, url: &str, body: &T) -> Outcome<()> {
        http_client().post_ok(url, None, HttpBody::json(body)?).await
    }
}
