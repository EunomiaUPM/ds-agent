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

//! A resolver asking another agent process for its public references over HTTP, for a process
//! (the gateway) that holds no URNs of its own.

use axum::http::{header, HeaderMap, HeaderValue};
use serde_json::Value;
use urn::Urn;
use ymir::errors::{BadFormat, Errors, Outcome};
use ymir::services::client::ClientTrait;
use ymir::utils::http_client;

use crate::public_refs::{PublicDocument, PublicQuery, PublicResolverTrait};
use crate::routes;

/// The public references of the agent at `base`.
pub struct RemotePublicResolver {
    base: String,
}

impl RemotePublicResolver {
    const SCHEMA_JSON: &'static str = "application/schema+json";

    /// `base` is the agent's HTTP host.
    pub fn new(base: impl Into<String>) -> Self {
        Self {
            base: base.into().trim_end_matches('/').to_string(),
        }
    }

    fn url(&self, urn: &Urn, query: &PublicQuery) -> Outcome<String> {
        let mut url = format!("{}{}/{urn}", self.base, routes::public_refs::PREFIX);
        let encoded = serde_urlencoded::to_string(query)
            .map_err(|e| Errors::format(BadFormat::Sent, "bad query", Some(Box::new(e))))?;
        if !encoded.is_empty() {
            url.push('?');
            url.push_str(&encoded);
        }
        Ok(url)
    }

    /// The agent's media type, among those a public document has.
    fn media_type(headers: &HeaderMap) -> &'static str {
        let given = headers
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default();
        if given.starts_with(Self::SCHEMA_JSON) {
            Self::SCHEMA_JSON
        } else if given.starts_with(PublicDocument::JSON) {
            PublicDocument::JSON
        } else {
            PublicDocument::JSON_LD
        }
    }

    /// The target of the `Link` with `rel`.
    fn link(headers: &HeaderMap, rel: &str) -> Option<String> {
        let wanted = format!("rel=\"{rel}\"");
        headers
            .get_all(header::LINK)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .flat_map(|v| v.split(','))
            .find(|link| link.contains(&wanted))
            .and_then(|link| Some(link.split_once('<')?.1.split_once('>')?.0.to_string()))
    }
}

#[async_trait::async_trait]
impl PublicResolverTrait for RemotePublicResolver {
    #[tracing::instrument(level = "debug", skip_all, err, fields(peer = %self.base, urn = %urn))]
    async fn resolve(&self, urn: &Urn, query: &PublicQuery) -> Outcome<Option<PublicDocument>> {
        let url = self.url(urn, query)?;
        let mut accept = HeaderMap::new();
        accept.insert(
            header::ACCEPT,
            HeaderValue::from_static("application/ld+json, */*"),
        );
        let response = http_client().get(&url, Some(accept)).await?;
        let status = response.status().as_u16();
        if status == 404 {
            return Ok(None);
        }
        let headers = response.headers().clone();
        let bytes = response.bytes().await.map_err(|e| {
            Errors::format(
                BadFormat::Received,
                "unreadable public reference",
                Some(Box::new(e)),
            )
        })?;
        if status == 400 {
            let detail = String::from_utf8_lossy(&bytes).to_string();
            return Err(Errors::format(BadFormat::Received, detail, None));
        }
        if !(200..300).contains(&status) {
            return Err(Errors::crazy(format!("{url} answered {status}"), None));
        }
        let body: Value = serde_json::from_slice(&bytes).map_err(|e| {
            Errors::format(
                BadFormat::Received,
                "public reference is not JSON",
                Some(Box::new(e)),
            )
        })?;
        let mut document =
            PublicDocument::json_ld(body).with_media_type(Self::media_type(&headers));
        if let Some(profile) = Self::link(&headers, "profile") {
            document = document.with_profile(profile);
        }
        // Where the owning agent serves it: its own route, else its canonical URL.
        let location = headers
            .get(header::CONTENT_LOCATION)
            .and_then(|v| v.to_str().ok())
            .map(str::to_string)
            .or_else(|| Self::link(&headers, "canonical"));
        if let Some(location) = location {
            document = document.with_location(location);
        }
        Ok(Some(document))
    }
}
