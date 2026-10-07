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

use crate::entities::protocol::ProtocolId;
use crate::protocols::dsp::entities::idempotency::{
    IdempotencyStoreTrait, InMemoryIdempotencyStore,
};
use crate::protocols::dsp::entities::message_types::TransferDSPMessageType;
use crate::protocols::dsp::services::dsp_handler_pipeline::DSPHandlerPipeline;
use axum::extract::{Path, Request, State};
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Router, middleware};
use ymir::http::OauthHttpMiddleware;
use common::facades::grants_facade::GrantsFacadeTrait;
use common::validation::ValidatorRegistry;
use http::StatusCode;
use std::sync::Arc;
use ymir::errors::AppResult;

use crate::protocols::dsp::entities::context_dsp::{
    TransferDSPContextDomain, TransferDSPContextTyped,
};
use crate::protocols::dsp::services::validator::TransferValidators;

#[derive(Clone)]
pub struct DspRouter {
    grants: Arc<dyn GrantsFacadeTrait>,
    idempotency: Arc<dyn IdempotencyStoreTrait>,
    edge_validators: Arc<ValidatorRegistry<TransferDSPMessageType, TransferDSPContextTyped>>,
    domain_validators: Arc<ValidatorRegistry<TransferDSPMessageType, TransferDSPContextDomain>>,
}

impl DspRouter {
    /// Uses the in-memory idempotency store and default transfer validators.
    pub fn new(grants: Arc<dyn GrantsFacadeTrait>) -> Self {
        Self::with_idempotency_store(grants, Arc::new(InMemoryIdempotencyStore::new()))
    }

    pub fn with_idempotency_store(
        grants: Arc<dyn GrantsFacadeTrait>,
        idempotency: Arc<dyn IdempotencyStoreTrait>,
    ) -> Self {
        Self::with_validators(
            grants,
            idempotency,
            Arc::new(TransferValidators::edge_registry()),
            Arc::new(TransferValidators::dsp_registry()),
        )
    }

    pub fn with_validators(
        grants: Arc<dyn GrantsFacadeTrait>,
        idempotency: Arc<dyn IdempotencyStoreTrait>,
        edge_validators: Arc<ValidatorRegistry<TransferDSPMessageType, TransferDSPContextTyped>>,
        domain_validators: Arc<ValidatorRegistry<TransferDSPMessageType, TransferDSPContextDomain>>,
    ) -> Self {
        Self {
            grants,
            idempotency,
            edge_validators,
            domain_validators,
        }
    }

    async fn auth_middleware(
        State(state): State<DspRouter>,
        mut request: Request,
        next: Next,
    ) -> Result<impl IntoResponse, StatusCode> {
        // DSP 10.1.2.3: an unauthorized client MUST get a 404, not a 401 — the
        // same answer as a missing process (10.1.2.2), so that probing cannot
        // reveal which Transfer Processes exist.
        let token = OauthHttpMiddleware::bearer(request.headers())
            .map_err(|_| StatusCode::NOT_FOUND)?
            .to_owned();
        match state.grants.verify_token(token).await {
            Ok(peer) => {
                request.extensions_mut().insert(peer);
                Ok(next.run(request).await)
            }
            Err(_) => Err(StatusCode::NOT_FOUND),
        }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route("/request", post(Self::handle_transfer_request::<DspRouter>))
            .route("/{id}", get(Self::handle_get_transfer_process))
            .route(
                "/{id}/start",
                post(Self::handle_transfer_start::<DspRouter>),
            )
            .route(
                "/{id}/completion",
                post(Self::handle_transfer_completion::<DspRouter>),
            )
            .route(
                "/{id}/termination",
                post(Self::handle_transfer_termination::<DspRouter>),
            )
            .route(
                "/{id}/suspension",
                post(Self::handle_transfer_suspension::<DspRouter>),
            )
            .layer(middleware::from_fn_with_state(
                self.clone(),
                Self::auth_middleware,
            ))
            .with_state(self)
    }

    async fn build_response<P: DSPHandlerPipeline>(
        request: Request,
        message_route: &TransferDSPMessageType,
        protocol_id: &ProtocolId,
        edge_validators: &ValidatorRegistry<TransferDSPMessageType, TransferDSPContextTyped>,
        domain_validators: &ValidatorRegistry<TransferDSPMessageType, TransferDSPContextDomain>,
    ) -> AppResult<StatusCode> {
        match P::run(
            request,
            message_route,
            protocol_id,
            edge_validators,
            domain_validators,
        )
        .await
        {
            // TODO, project to DSP
            Ok(_res) => Ok(StatusCode::ACCEPTED),
            // TODO, project to DSP
            Err(err) => {
                tracing::warn!(%err, "DSP pipeline rejected the message");
                Ok(StatusCode::BAD_REQUEST)
            }
        }
    }

    async fn handle_transfer_request<P: DSPHandlerPipeline>(
        State(state): State<DspRouter>,
        request: Request,
    ) -> AppResult<StatusCode> {
        Self::build_response::<P>(
            request,
            &TransferDSPMessageType::TransferRequestMessage,
            &ProtocolId::Dsp2025_1,
            &state.edge_validators,
            &state.domain_validators,
        )
        .await
    }

    async fn handle_get_transfer_process(
        State(_state): State<DspRouter>,
        Path(_id): Path<String>,
    ) -> impl IntoResponse {
        "ok"
    }

    async fn handle_transfer_start<P: DSPHandlerPipeline>(
        State(state): State<DspRouter>,
        request: Request,
    ) -> AppResult<StatusCode> {
        Self::build_response::<P>(
            request,
            &TransferDSPMessageType::TransferStartMessage,
            &ProtocolId::Dsp2025_1,
            &state.edge_validators,
            &state.domain_validators,
        )
        .await
    }

    async fn handle_transfer_completion<P: DSPHandlerPipeline>(
        State(state): State<DspRouter>,
        request: Request,
    ) -> AppResult<StatusCode> {
        Self::build_response::<P>(
            request,
            &TransferDSPMessageType::TransferCompletionMessage,
            &ProtocolId::Dsp2025_1,
            &state.edge_validators,
            &state.domain_validators,
        )
        .await
    }

    async fn handle_transfer_termination<P: DSPHandlerPipeline>(
        State(state): State<DspRouter>,
        request: Request,
    ) -> AppResult<StatusCode> {
        Self::build_response::<P>(
            request,
            &TransferDSPMessageType::TransferTerminationMessage,
            &ProtocolId::Dsp2025_1,
            &state.edge_validators,
            &state.domain_validators,
        )
        .await
    }

    async fn handle_transfer_suspension<P: DSPHandlerPipeline>(
        State(state): State<DspRouter>,
        request: Request,
    ) -> AppResult<StatusCode> {
        Self::build_response::<P>(
            request,
            &TransferDSPMessageType::TransferSuspensionMessage,
            &ProtocolId::Dsp2025_1,
            &state.edge_validators,
            &state.domain_validators,
        )
        .await
    }
}
