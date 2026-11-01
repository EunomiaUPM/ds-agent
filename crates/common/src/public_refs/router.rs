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

//! The route of the public references, asking each resolver in turn.

use std::str::FromStr;
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use urn::Urn;
use ymir::errors::{BadFormat, Errors, Outcome};

use crate::public_refs::{PublicDocument, PublicQuery, PublicResolverTrait};
use crate::routes;

/// Routes `/{urn}` under `/public`, with no authentication.
#[derive(Clone)]
pub struct PublicRefsRouter {
    /// The process's exterior HTTP host, which canonical URLs and locations hang from.
    base: Arc<str>,
    resolvers: Arc<[Arc<dyn PublicResolverTrait>]>,
}

impl PublicRefsRouter {
    /// Resolvers are asked in this order.
    pub fn new(base: impl Into<String>, resolvers: Vec<Arc<dyn PublicResolverTrait>>) -> Self {
        Self {
            base: base.into().trim_end_matches('/').into(),
            resolvers: resolvers.into(),
        }
    }

    /// The URL anyone dereferences `urn` at through this process.
    pub fn canonical(&self, urn: &Urn) -> String {
        format!("{}{}/{urn}", self.base, routes::public_refs::PREFIX)
    }

    pub fn router(self) -> Router {
        Router::new()
            .route(routes::public_refs::BY_URN, get(Self::handle_urn))
            .with_state(self)
    }

    async fn handle_urn(
        State(state): State<Self>,
        headers: HeaderMap,
        Path(urn): Path<String>,
        Query(query): Query<PublicQuery>,
    ) -> Response {
        match state.resolve(&urn, &query).await {
            Ok(document) => document.respond(&headers),
            Err(e) => e.into_response(),
        }
    }

    /// The first resolver's document; the same 404 whether the URN is unknown or not public.
    pub async fn resolve(&self, urn: &str, query: &PublicQuery) -> Outcome<PublicDocument> {
        let parsed = Urn::from_str(urn).map_err(|e| {
            Errors::format(
                BadFormat::Received,
                format!("{urn} is not a URN"),
                Some(Box::new(e)),
            )
        })?;
        for resolver in self.resolvers.iter() {
            if let Some(document) = resolver.resolve(&parsed, query).await? {
                return Ok(document
                    .anchored(&self.base)
                    .with_canonical(self.canonical(&parsed)));
            }
        }
        Err(Errors::missing_resource(
            urn,
            "public reference not found",
            None,
        ))
    }
}
