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
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use ymir::data::entities::mates;
use ymir::data::entities::req_vc::Model;
use ymir::errors::AppResult;
use ymir::types::gnap::{ApprovedCallbackBody, CallbackBody};
use ymir::utils::{extract_payload, extract_query_param};

use crate::core::traits::CoreVcRequesterTrait;
use crate::types::entities::ReachAuthority;
use crate::types::wallet_helper::{ProcessUriOid4VCI, ProcessUriOid4VP};

pub struct VcRequesterRouter {
    requester: Arc<dyn CoreVcRequesterTrait>,
}

impl VcRequesterRouter {
    pub fn new(requester: Arc<dyn CoreVcRequesterTrait>) -> Self {
        VcRequesterRouter { requester }
    }

    pub fn router(self) -> Router {
        Router::new()
            .route("/beg", post(Self::beg))
            .route("/all", get(Self::get_all))
            .route("/{id}", get(Self::get_one))
            .route("/callback/{id}", get(Self::get_callback))
            .route("/callback/{id}", post(Self::post_callback))
            .route("/oidc4vci", post(Self::oidc4vci))
            .route("/oidc4vp", post(Self::oidc4vp))
            .with_state(self.requester)
    }

    async fn beg(
        State(requester): State<Arc<dyn CoreVcRequesterTrait>>,
        payload: Result<Json<ReachAuthority>, JsonRejection>,
    ) -> AppResult {
        let payload = extract_payload(payload)?;
        Ok(match requester.beg_vc(payload).await {
            Ok(Some(data)) => data.into_response(),
            Ok(None) => ().into_response(),
            Err(err) => err.into_response(),
        })
    }

    async fn get_all(
        State(requester): State<Arc<dyn CoreVcRequesterTrait>>,
    ) -> AppResult<Json<Vec<Model>>> {
        Ok(Json(requester.get_all().await?))
    }

    async fn get_one(
        State(requester): State<Arc<dyn CoreVcRequesterTrait>>,
        Path(id): Path<String>,
    ) -> AppResult<Json<Model>> {
        Ok(Json(requester.get_by_id(id).await?))
    }
    async fn get_callback(
        State(requester): State<Arc<dyn CoreVcRequesterTrait>>,
        Path(id): Path<String>,
        Query(params): Query<HashMap<String, String>>,
    ) -> AppResult<Json<mates::Model>> {
        let hash = extract_query_param(&params, "hash")?;
        let interact_ref = extract_query_param(&params, "interact_ref")?;
        let payload = ApprovedCallbackBody { interact_ref, hash };
        Ok(Json(requester.continue_req(id, payload).await?))
    }

    async fn post_callback(
        State(requester): State<Arc<dyn CoreVcRequesterTrait>>,
        Path(id): Path<String>,
        payload: Result<Json<CallbackBody>, JsonRejection>,
    ) -> AppResult {
        let payload = extract_payload(payload)?;
        Ok(match payload {
            CallbackBody::Approved(data) => requester
                .continue_req(id, data)
                .await
                .map(Json)
                .into_response(),
            CallbackBody::Rejected(_) => requester.manage_rejection(id).await.into_response(),
        })
    }
    async fn oidc4vci(
        State(requester): State<Arc<dyn CoreVcRequesterTrait>>,
        payload: Result<Json<ProcessUriOid4VCI>, JsonRejection>,
    ) -> AppResult {
        let payload = extract_payload(payload)?;
        Ok(requester.process_oid4vci(&payload).await?.into_response())
    }
    async fn oidc4vp(
        State(requester): State<Arc<dyn CoreVcRequesterTrait>>,
        payload: Result<Json<ProcessUriOid4VP>, JsonRejection>,
    ) -> AppResult {
        let payload = extract_payload(payload)?;
        Ok(requester.process_oid4vp(&payload).await?.into_response())
    }
}
