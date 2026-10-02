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

//! A running agent of the dev stack, reached over HTTP as its admin.

use axum::http::HeaderMap;
use serde_json::{json, Value};
use ymir::services::client::ClientExt;
use ymir::types::http::HttpBody;
use ymir::utils::{bearer_headers, http_client};

pub struct Agent {
    pub url: String,
    token: Option<String>,
}

impl Agent {
    /// Agent at `$var` or `default`, logged in as the dev admin when it has an OAuth server.
    pub async fn login(var: &str, default: &str) -> Self {
        let url = std::env::var(var).unwrap_or_else(|_| default.to_string());
        let email =
            std::env::var("ADMIN_EMAIL").unwrap_or_else(|_| "admin@admin.local".to_string());
        let password = std::env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "admin".to_string());
        let login = json!({
            "grant_type": "password",
            "client_id": "eunomia-admin-gui",
            "username": email,
            "password": password
        });
        let token = http_client()
            .post_json::<_, Value>(&format!("{url}/oauth/token"), None, &login)
            .await
            .ok()
            .and_then(|r| r["access_token"].as_str().map(str::to_string));
        Self { url, token }
    }

    pub async fn get(&self, path: &str) -> Value {
        http_client()
            .get_json(&format!("{}{path}", self.url), self.headers())
            .await
            .unwrap_or_else(|e| panic!("GET {}{path}: {e:?}", self.url))
    }

    /// POST that must answer 2xx; the body of the answer is ignored.
    pub async fn post(&self, path: &str, body: Value) {
        http_client()
            .post_ok(
                &format!("{}{path}", self.url),
                self.headers(),
                HttpBody::Json(body),
            )
            .await
            .unwrap_or_else(|e| panic!("POST {}{path}: {e:?}", self.url));
    }

    pub async fn post_json(&self, path: &str, body: Value) -> Value {
        http_client()
            .post_json(&format!("{}{path}", self.url), self.headers(), &body)
            .await
            .unwrap_or_else(|e| panic!("POST {}{path}: {e:?}", self.url))
    }

    /// DID the agent serves at `/.well-known/did.json`.
    pub async fn did(&self) -> String {
        self.get("/.well-known/did.json").await["id"]
            .as_str()
            .expect("DID document with an id")
            .to_string()
    }

    fn headers(&self) -> Option<HeaderMap> {
        self.token
            .as_deref()
            .map(|token| bearer_headers(token).expect("valid bearer token"))
    }
}
