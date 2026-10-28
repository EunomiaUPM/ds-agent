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

//! DSP catalog routes (DSP 6.2): every body is read through RDF; failures are `CatalogError`s.

use std::str::FromStr;
use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{middleware, Extension, Json, Router};
use common::dsp_common::PeerAuth;
use common::facades::grants_facade::VerifiedPeer;
use common::flow::FlowError;
use common::rdf::RdfMessage;
use common::routes::catalog as routes;
use serde_json::Value;
use urn::Urn;
use ymir::errors::Errors;

use crate::protocols::dsp::entities::catalog_error::CatalogErrorDto;
use crate::protocols::dsp::entities::message_types::CatalogDspMessageType;
use crate::protocols::dsp::entities::rdf_extractor_dsp::DspCatalog;
use crate::protocols::dsp::setup::context::DspContext;

#[derive(Clone)]
pub struct DspRouter {
    ctx: Arc<DspContext>,
}

impl DspRouter {
    pub fn new(ctx: Arc<DspContext>) -> Self {
        Self { ctx }
    }

    /// Every route behind `PeerAuth`; an unknown peer gets the 401 of the catalog.
    pub fn router(self) -> Router {
        let auth = &self.ctx.app.ports.auth;
        let peer_auth = PeerAuth::new(auth.grants.clone(), auth.mates.clone())
            .rejecting_with(StatusCode::UNAUTHORIZED);
        Router::new()
            .route(routes::dsp::REQUEST, post(Self::handle_catalog_request))
            .route(routes::dsp::DATASET, get(Self::handle_dataset_request))
            .layer(middleware::from_fn_with_state(peer_auth, PeerAuth::run))
            .with_state(self)
    }

    async fn handle_catalog_request(
        State(state): State<Self>,
        Extension(peer): Extension<VerifiedPeer>,
        body: Bytes,
    ) -> Response {
        let json: Value = match serde_json::from_slice(&body) {
            Ok(json) => json,
            Err(e) => return Self::catalog_error(StatusCode::BAD_REQUEST, vec![format!("body is not JSON: {e}")]),
        };
        let message = match RdfMessage::<DspCatalog>::read_as(&CatalogDspMessageType::CatalogRequestMessage, json).await {
            Ok(message) => message,
            Err(e) => return Self::failure(e),
        };
        match state.ctx.catalog.catalog(&peer.participant_id, &message.fields).await {
            Ok(catalog) => (StatusCode::OK, Json(catalog)).into_response(),
            Err(e) => Self::failure(e),
        }
    }

    async fn handle_dataset_request(
        State(state): State<Self>,
        Extension(peer): Extension<VerifiedPeer>,
        Path(id): Path<String>,
    ) -> Response {
        let Ok(id) = Urn::from_str(&id) else {
            return Self::catalog_error(StatusCode::BAD_REQUEST, vec![format!("{id} is not a dataset id")]);
        };
        match state.ctx.catalog.dataset(&peer.participant_id, &id).await {
            Ok(dataset) => (StatusCode::OK, Json(dataset)).into_response(),
            Err(e) => Self::failure(e),
        }
    }

    /// A `CatalogError` (DSP 6.2.3) with the status as its code.
    fn catalog_error(status: StatusCode, reasons: Vec<String>) -> Response {
        (status, Json(CatalogErrorDto::new(status.as_u16().to_string(), reasons))).into_response()
    }

    /// A failure as a `CatalogError`: an unknown dataset is a 404, internals stay hidden.
    fn failure(error: Errors) -> Response {
        let error = FlowError::from(error);
        let status = StatusCode::from_u16(error.status()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        Self::catalog_error(status, error.reasons())
    }
}
