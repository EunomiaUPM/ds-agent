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

//! What the management API carries outside the RDF body: version (`ETag`, `If-Match`),
//! visibility and owner, and the paging of lists.

use axum::http::{header, HeaderMap, HeaderName, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use common::paginated_spec::Paginated;
use serde_json::json;
use ymir::errors::{BadFormat, Errors, Outcome};
use ymir::types::participants::Visibility;

use crate::entities::admin_document::AdminDocument;
use crate::entities::resource::CatalogResource;

/// The headers of the management API and the responses built with them.
pub struct AdminHeaders;

impl AdminHeaders {
    pub const VISIBILITY: &'static str = "eunomia-visibility";
    pub const OWNER: &'static str = "eunomia-owner";
    pub const OWNER_ROLE: &'static str = "eunomia-owner-role";
    pub const TOTAL: &'static str = "x-total-count";
    pub const NEXT_CURSOR: &'static str = "x-next-cursor";
    const JSON_LD: &'static str = "application/ld+json";

    /// The visibility a write asks for (`Private`, `Anonymous`, `Public`), if any.
    pub fn visibility(headers: &HeaderMap) -> Outcome<Option<Visibility>> {
        let Some(value) = headers.get(Self::VISIBILITY) else {
            return Ok(None);
        };
        let text = value.to_str().unwrap_or_default().trim();
        serde_json::from_value(json!(text)).map(Some).map_err(|_| {
            Errors::format(
                BadFormat::Received,
                format!("{} must be Private, Anonymous or Public, not {text}", Self::VISIBILITY),
                None,
            )
        })
    }

    /// The version `If-Match` names (`W/"3"` or `"3"`); `*` or no header asks for none.
    pub fn expected_version(headers: &HeaderMap) -> Outcome<Option<i64>> {
        let Some(value) = headers.get(header::IF_MATCH) else {
            return Ok(None);
        };
        let text = value.to_str().unwrap_or_default().trim();
        if text == "*" {
            return Ok(None);
        }
        let bare = text.trim_start_matches("W/").trim_matches('"');
        bare.parse::<i64>().map(Some).map_err(|_| {
            Errors::format(BadFormat::Received, format!("If-Match {text} names no version"), None)
        })
    }

    /// One resource as JSON-LD, with its version, visibility and owner.
    pub fn resource(status: StatusCode, resource: &CatalogResource) -> Response {
        let mut response = (status, Json(AdminDocument::render(resource))).into_response();
        let headers = response.headers_mut();
        Self::json_ld(headers);
        let owner = resource.owner();
        Self::set(headers, header::ETAG, &resource.etag());
        Self::set(headers, HeaderName::from_static(Self::VISIBILITY), Self::visibility_name(&owner.visibility));
        Self::set(headers, HeaderName::from_static(Self::OWNER), &owner.user_id);
        Self::set(headers, HeaderName::from_static(Self::OWNER_ROLE), &owner.role.to_string());
        response
    }

    /// Several resources as one JSON-LD `@graph`.
    pub fn graph(resources: &[CatalogResource]) -> Response {
        let mut response = Json(AdminDocument::render_graph(resources)).into_response();
        Self::json_ld(response.headers_mut());
        response
    }

    /// A page as a `@graph`, its total and next cursor in headers.
    pub fn page(page: &Paginated<CatalogResource>) -> Response {
        let mut response = Self::graph(&page.items);
        let headers = response.headers_mut();
        if let Some(total) = page.total {
            Self::set(headers, HeaderName::from_static(Self::TOTAL), &total.to_string());
        }
        if let Some(cursor) = &page.next_cursor {
            Self::set(headers, HeaderName::from_static(Self::NEXT_CURSOR), cursor);
            Self::set(headers, header::LINK, &format!("<?cursor={cursor}>; rel=\"next\""));
        }
        response
    }

    fn visibility_name(visibility: &Visibility) -> &'static str {
        match visibility {
            Visibility::Private => "Private",
            Visibility::Anonymous => "Anonymous",
            Visibility::Public => "Public",
        }
    }

    fn json_ld(headers: &mut HeaderMap) {
        headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(Self::JSON_LD));
    }

    fn set(headers: &mut HeaderMap, name: HeaderName, value: &str) {
        if let Ok(value) = HeaderValue::from_str(value) {
            headers.insert(name, value);
        }
    }
}
