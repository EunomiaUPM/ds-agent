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

//! Catalogs in and out of the agent: a catalog in any profile and RDF syntax (a preview cut to a
//! number of datasets, or a download), and the import of DCAT documents validated with SHACL.

use std::str::FromStr;
use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, Path, Query, State};
use axum::http::{header, HeaderMap, HeaderName, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use common::rdf::RdfSyntax;
use common::routes::catalog::catalogs as routes;
use serde::Deserialize;
use urn::Urn;
use ymir::errors::{BadFormat, Errors, Outcome};
use ymir::types::oauth::UserInfo;

use crate::entities::commands::{ImportCatalogCommand, ImportMode};
use crate::entities::resource_kind::ResourceKind;
use crate::http::resources::headers::AdminHeaders;
use crate::services::projection::views::{Projection, ProjectionBody};
use crate::services::projection::ProjectionRequest;
use crate::setup::context::AppContext;

/// What the profile route takes.
#[derive(Debug, Default, Deserialize)]
pub struct ProjectionQuery {
    /// Datasets and series kept in a preview.
    pub limit: Option<usize>,
    /// `jsonld`, `turtle`, `ntriples` or `rdfxml`; else the `Accept` header decides.
    pub format: Option<String>,
    /// The whole document as an attachment; `limit` is ignored.
    #[serde(default)]
    pub download: bool,
}

/// What the import route takes.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportQuery {
    pub profile: Option<String>,
    pub mode: Option<ImportMode>,
    #[serde(default)]
    pub dry_run: bool,
    pub target: Option<String>,
    /// Shape groups, comma separated.
    pub checks: Option<String>,
}

#[derive(Clone)]
pub struct CatalogExchangeRouter {
    ctx: Arc<AppContext>,
}

impl CatalogExchangeRouter {
    pub const SHOWN: &'static str = "x-shown-count";
    /// Documents up to this size are imported.
    const MAX_IMPORT_BYTES: usize = 32 * 1024 * 1024;
    const DEFAULT_IMPORT_PROFILE: &'static str = "dcat-ap";

    pub fn new(ctx: Arc<AppContext>) -> Self {
        Self { ctx }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route(routes::PROFILE, get(Self::handle_profile))
            .route(
                routes::IMPORT,
                post(Self::handle_import).layer(DefaultBodyLimit::max(Self::MAX_IMPORT_BYTES)),
            )
            .with_state(self)
    }

    async fn handle_profile(
        State(state): State<Self>,
        user: UserInfo,
        headers: HeaderMap,
        Path((id, profile)): Path<(String, String)>,
        Query(query): Query<ProjectionQuery>,
    ) -> Response {
        let outcome = async {
            let id = Self::urn(&id)?;
            let request = ProjectionRequest {
                profile,
                limit: if query.download { None } else { query.limit },
                syntax: Self::syntax(query.format.as_deref(), &headers)?,
            };
            state
                .ctx
                .projections()
                .project(&user, ResourceKind::Catalog, &id, &request)
                .await
        };
        match outcome.await {
            Ok(projection) => Self::respond(projection, query.download),
            Err(e) => e.into_response(),
        }
    }

    async fn handle_import(
        State(state): State<Self>,
        user: UserInfo,
        headers: HeaderMap,
        Query(query): Query<ImportQuery>,
        body: Bytes,
    ) -> Response {
        let outcome = async {
            let content_type = headers
                .get(header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .unwrap_or(RdfSyntax::JsonLd.media_type());
            let syntax = RdfSyntax::from_media_type(content_type)?;
            let command = ImportCatalogCommand {
                profile: query
                    .profile
                    .unwrap_or_else(|| Self::DEFAULT_IMPORT_PROFILE.to_string()),
                mode: query.mode.unwrap_or_default(),
                dry_run: query.dry_run,
                target: query.target.as_deref().map(Self::urn).transpose()?,
                checks: query
                    .checks
                    .map(|c| {
                        c.split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect()
                    })
                    .unwrap_or_default(),
                visibility: AdminHeaders::visibility(&headers)?,
            };
            state
                .ctx
                .imports()
                .import(&user, syntax, &body, &command)
                .await
        };
        match outcome.await {
            Ok(report) => {
                let status = if report.imported || report.dry_run {
                    StatusCode::OK
                } else {
                    StatusCode::UNPROCESSABLE_ENTITY
                };
                (status, Json(report)).into_response()
            }
            Err(e) => e.into_response(),
        }
    }

    /// The syntax `format` names, else the first RDF one `Accept` asks for, else JSON-LD.
    fn syntax(format: Option<&str>, headers: &HeaderMap) -> Outcome<RdfSyntax> {
        if let Some(format) = format {
            return RdfSyntax::from_name(format).ok_or_else(|| {
                Errors::format(
                    BadFormat::Received,
                    format!("unknown format {format}; use jsonld, turtle, ntriples or rdfxml"),
                    None,
                )
            });
        }
        let accept = headers
            .get(header::ACCEPT)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default();
        Ok(accept
            .split(',')
            .find_map(|media| RdfSyntax::from_media_type(media).ok())
            .unwrap_or(RdfSyntax::JsonLd))
    }

    /// The document with its media type, profile link and how much of it a preview shows.
    fn respond(projection: Projection, download: bool) -> Response {
        let mut response = match &projection.body {
            ProjectionBody::JsonLd(value) => Json(value.clone()).into_response(),
            ProjectionBody::Text(text) => text.clone().into_response(),
        };
        let content_type = match projection.syntax {
            RdfSyntax::JsonLd => projection.syntax.media_type().to_string(),
            other => format!("{}; charset=utf-8", other.media_type()),
        };
        let headers = response.headers_mut();
        Self::set(headers, header::CONTENT_TYPE, &content_type);
        Self::set(
            headers,
            header::LINK,
            &format!("<{}>; rel=\"profile\"", projection.profile_iri),
        );
        Self::set(
            headers,
            HeaderName::from_static(AdminHeaders::TOTAL),
            &projection.truncation.total.to_string(),
        );
        Self::set(
            headers,
            HeaderName::from_static(Self::SHOWN),
            &projection.truncation.shown.to_string(),
        );
        if download {
            Self::set(
                headers,
                header::CONTENT_DISPOSITION,
                &format!("attachment; filename=\"{}\"", projection.file_name()),
            );
        }
        response
    }

    fn set(headers: &mut HeaderMap, name: HeaderName, value: &str) {
        if let Ok(value) = HeaderValue::from_str(value) {
            headers.insert(name, value);
        }
    }

    fn urn(id: &str) -> Outcome<Urn> {
        Urn::from_str(id)
            .map_err(|_| Errors::format(BadFormat::Received, format!("{id} is not a URN"), None))
    }
}
