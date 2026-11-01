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

//! The document a resolver returns and how it is written to HTTP.

use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

/// Query of a public reference.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicQuery {
    /// Catalog profile the resource is projected in (`dcat3` when absent).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    /// Version of a versioned resource (the latest when absent).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<i32>,
}

/// A public document with the headers that describe it.
#[derive(Debug, Clone, PartialEq)]
pub struct PublicDocument {
    pub body: Value,
    pub media_type: &'static str,
    /// Sent as `Link: <iri>; rel="profile"`.
    pub profile: Option<String>,
    /// Sent as `Content-Location`: where the owning agent serves the same document.
    pub location: Option<String>,
    /// Sent as `Link: <url>; rel="canonical"`: the stable URL that dereferences the URN.
    pub canonical: Option<String>,
}

impl PublicDocument {
    pub const JSON_LD: &'static str = "application/ld+json";
    pub const JSON: &'static str = "application/json";

    pub fn json_ld(body: Value) -> Self {
        Self {
            body,
            media_type: Self::JSON_LD,
            profile: None,
            location: None,
            canonical: None,
        }
    }

    pub fn with_media_type(mut self, media_type: &'static str) -> Self {
        self.media_type = media_type;
        self
    }

    pub fn with_profile(mut self, iri: impl Into<String>) -> Self {
        self.profile = Some(iri.into());
        self
    }

    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }

    pub fn with_canonical(mut self, url: impl Into<String>) -> Self {
        self.canonical = Some(url.into());
        self
    }

    /// A location given as a path becomes absolute under `base`.
    pub fn anchored(mut self, base: &str) -> Self {
        if let Some(location) = self.location.as_mut().filter(|l| l.starts_with('/')) {
            *location = format!("{base}{location}");
        }
        self
    }

    /// Strong tag over the body.
    pub fn etag(&self) -> String {
        let digest = Sha256::digest(self.body.to_string().as_bytes());
        let hex: String = digest[..12].iter().map(|b| format!("{b:02x}")).collect();
        format!("\"{hex}\"")
    }

    /// `304` when `If-None-Match` matches; otherwise the body, readable from any origin.
    pub fn respond(self, request: &HeaderMap) -> Response {
        let etag = self.etag();
        if request
            .get(header::IF_NONE_MATCH)
            .and_then(|v| v.to_str().ok())
            .is_some_and(|tags| tags.split(',').any(|t| t.trim() == etag))
        {
            return StatusCode::NOT_MODIFIED.into_response();
        }
        let media_type = self.negotiated(request);
        let mut response = Json(self.body).into_response();
        let headers = response.headers_mut();
        headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(media_type));
        headers.insert(header::VARY, HeaderValue::from_static("Accept"));
        headers.insert(
            header::ACCESS_CONTROL_ALLOW_ORIGIN,
            HeaderValue::from_static("*"),
        );
        if let Ok(tag) = HeaderValue::from_str(&etag) {
            headers.insert(header::ETAG, tag);
        }
        for (url, rel) in [(self.profile, "profile"), (self.canonical, "canonical")] {
            if let Some(link) =
                url.and_then(|u| HeaderValue::from_str(&format!("<{u}>; rel=\"{rel}\"")).ok())
            {
                headers.append(header::LINK, link);
            }
        }
        if let Some(location) = self.location.and_then(|l| HeaderValue::from_str(&l).ok()) {
            headers.insert(header::CONTENT_LOCATION, location);
        }
        response
    }

    /// JSON-LD goes as plain JSON to a client that accepts JSON but not JSON-LD.
    fn negotiated(&self, request: &HeaderMap) -> &'static str {
        let accept = request
            .get(header::ACCEPT)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default();
        let plain_only = accept.contains(Self::JSON)
            && !accept.contains(Self::JSON_LD)
            && !accept.contains("*/*");
        if self.media_type == Self::JSON_LD && plain_only {
            Self::JSON
        } else {
            self.media_type
        }
    }
}
