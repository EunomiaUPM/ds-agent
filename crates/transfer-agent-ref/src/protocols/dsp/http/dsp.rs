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

use axum::extract::{Path, Request, State};
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Router, middleware};
use common::auth::middleware::bearer;
use common::dsp_common::normalizer::dsp_namespace_normalizer;
use common::dsp_common::well_known_types::DSPProtocolVersions;
use common::facades::ssi_auth_facade::SSIAuthFacadeTrait;
use http::StatusCode;
use std::sync::Arc;
use ymir::errors::{BadFormat, Errors};

use crate::protocols::dsp::entities::auth::TransferDSPAuthn;
use crate::protocols::dsp::entities::context_common::TransferContextRaw;
use crate::protocols::dsp::entities::context_dsp::{
    TransferDSPContextParsed, TransferDSPContextRdf, TransferDSPContextTyped,
};
use crate::protocols::dsp::entities::message_types::TransferDSPMessageType;

#[derive(Clone)]
pub struct DspRouter {
    ssi_auth: Arc<dyn SSIAuthFacadeTrait>,
}

impl DspRouter {
    pub fn new(ssi_auth: Arc<dyn SSIAuthFacadeTrait>) -> Self {
        Self { ssi_auth }
    }

    async fn auth_middleware(
        State(state): State<DspRouter>,
        mut request: Request,
        next: Next,
    ) -> Result<impl IntoResponse, StatusCode> {
        let token = bearer(request.headers())
            .map_err(|_| StatusCode::UNAUTHORIZED)?
            .to_owned();
        match state.ssi_auth.verify_token(token).await {
            Ok(mate) => {
                request.extensions_mut().insert(mate);
                Ok(next.run(request).await)
            }
            Err(_) => Err(StatusCode::UNAUTHORIZED),
        }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route("/request", post(Self::handle_transfer_request))
            .route("/{id}", get(Self::handle_get_transfer_process))
            .route("/{id}/start", post(Self::handle_transfer_start))
            .route("/{id}/completion", post(Self::handle_transfer_completion))
            .route("/{id}/termination", post(Self::handle_transfer_termination))
            .route("/{id}/suspension", post(Self::handle_transfer_suspension))
            .layer(middleware::from_fn_with_state(
                self.clone(),
                Self::auth_middleware,
            ))
            .layer(middleware::from_fn(dsp_namespace_normalizer))
            .with_state(self)
    }

    async fn handle_transfer_request(request: Request) -> Result<impl IntoResponse, Errors> {
        // Wire extraction: reads body + headers and the `Mates` the auth
        // middleware resolved into the request extensions.
        let raw = TransferContextRaw::<TransferDSPAuthn>::from_request(request).await?;
        dbg!("Raw: {}", &raw);

        // Parse stage: this endpoint is always a TransferRequestMessage. The JSON
        // body is parsed here; DSP protocol-version detection is not implemented
        // yet, so we default it.
        // ponytail: version detection belongs in a parse stage, defaulted until it exists.
        let json: serde_json::Value = serde_json::from_slice(&raw.body_bytes).map_err(|e| {
            Errors::format(BadFormat::Received, format!("body is not JSON: {e}"), None)
        })?;
        dbg!("JSON: {}", &json);

        let parsed = TransferDSPContextParsed::from_raw(
            raw,
            DSPProtocolVersions::V2025_1,
            TransferDSPMessageType::TransferRequestMessage,
            json,
        )?;
        dbg!("Parsed: {}", &parsed);

        // RDF stage (canonicalize) → typed stage (extract pids / data address).
        let rdf = TransferDSPContextRdf::from_parsed(parsed).await?;
        dbg!("Rdf: {}", &rdf);

        let typed = TransferDSPContextTyped::from_rdf(rdf)?;
        dbg!("Typed: {}", &typed);

        // TODO(loader): `typed` → `TransferDSPContextDomain` via `DspDomainLoader::load`.
        // Blocked on: (1) a `DspDomainLoaderTrait` impl (agreement/role resolution),
        // (2) injecting the loader into `DspRouter` (it only holds `ssi_auth` today).
        tracing::info!(
            message = %typed.message,
            consumer_pid = ?typed.consumer_pid,
            provider_pid = ?typed.provider_pid,
            "DSP transfer request: typed context built",
        );
        Ok(StatusCode::ACCEPTED)
    }

    async fn handle_get_transfer_process(
        State(state): State<DspRouter>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        "ok"
    }

    async fn handle_transfer_start(
        State(state): State<DspRouter>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        "ok"
    }

    async fn handle_transfer_completion(
        State(state): State<DspRouter>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        "ok"
    }

    async fn handle_transfer_termination(
        State(state): State<DspRouter>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        "ok"
    }

    async fn handle_transfer_suspension(
        State(state): State<DspRouter>,
        Path(id): Path<String>,
    ) -> impl IntoResponse {
        "ok"
    }
}
