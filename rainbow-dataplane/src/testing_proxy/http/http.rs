/*
 *
 *  * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
 *  *
 *  * This program is free software: you can redistribute it and/or modify
 *  * it under the terms of the GNU General Public License as published by
 *  * the Free Software Foundation, either version 3 of the License, or
 *  * (at your option) any later version.
 *  *
 *  * This program is distributed in the hope that it will be useful,
 *  * but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  * GNU General Public License for more details.
 *  *
 *  * You should have received a copy of the GNU General Public License
 *  * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 */
use crate::entities::dataplane_transfers::DataplaneTransfersEntitiesTrait;
use crate::entities::dataplane_transfers::{InteractionMode, TransferState};
use axum::body::{to_bytes, Body};
use axum::extract::{FromRef, Path, Request, State};
use axum::response::{IntoResponse, Response};
use axum::routing::any;
use axum::Router;
use hyper::Method;
use rainbow_common::adv_protocol::interplane::{DataPlaneProcessDirection, DataPlaneProcessState};
use rainbow_common::utils::get_urn_from_string;
use reqwest::Response as ReqwestResponse;
use reqwest::{Client, StatusCode};
use std::sync::Arc;
use tracing::info;

#[derive(Clone)]
pub struct TestingHTTPProxy {
    client: Client,
    dataplane_service: Arc<dyn DataplaneTransfersEntitiesTrait>,
}

impl FromRef<TestingHTTPProxy> for Client {
    fn from_ref(input: &TestingHTTPProxy) -> Self {
        input.client.clone()
    }
}

impl FromRef<TestingHTTPProxy> for Arc<dyn DataplaneTransfersEntitiesTrait> {
    fn from_ref(input: &TestingHTTPProxy) -> Self {
        input.dataplane_service.clone()
    }
}

impl TestingHTTPProxy {
    pub fn new(dataplane_service: Arc<dyn DataplaneTransfersEntitiesTrait>) -> Self {
        let client = reqwest::Client::new();
        Self { client, dataplane_service }
    }
    pub fn router(self) -> Router {
        Router::new().route("/:data_plane_id", any(Self::forward_request)).with_state(self)
    }

    async fn forward_request(
        State(state): State<TestingHTTPProxy>,
        Path(data_plane_id): Path<String>,
        mut req: Request,
    ) -> impl IntoResponse {
        info!("* /data/{}", data_plane_id);
        // validations
        let data_plane_id = match get_urn_from_string(&data_plane_id) {
            Ok(data_plane_id) => data_plane_id,
            Err(_) => return (StatusCode::BAD_REQUEST, "data_plane_id not urn").into_response(),
        };

        // PDP
        let dataplane = match state
            .dataplane_service
            .get_dataplane_transfer_by_process_id(&data_plane_id)
            .await
        {
            Ok(dataplane) => match dataplane {
                Some(dataplane) => dataplane,
                None => return (StatusCode::BAD_REQUEST, "dataplane id not found").into_response(),
            },
            Err(_) => return (StatusCode::BAD_REQUEST, "dataplane id not found").into_response(),
        };

        match dataplane.inner.interaction_mode {
            InteractionMode::Pull => {}
            _ => return (StatusCode::BAD_REQUEST, "wrong direction").into_response(),
        }

        match dataplane.inner.state {
            TransferState::Started => {}
            TransferState::Init
            | TransferState::Configuring
            | TransferState::Auth
            | TransferState::Ready
            | TransferState::Subscribing => {
                return (StatusCode::FORBIDDEN, "state preparing").into_response()
            }
            TransferState::Stopped | TransferState::Unsubscribing => {
                return (StatusCode::FORBIDDEN, "state stopped").into_response()
            }
            TransferState::Terminated | TransferState::Error => {
                return (StatusCode::FORBIDDEN, "state terminated").into_response()
            }
        }

        let downstream_hop =
            match dataplane.inner.egress_config.get("DownstreamHopUrl").and_then(|v| v.as_str()) {
                Some(url) => url,
                None => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Missing DownstreamHopUrl in egress_config",
                    )
                        .into_response()
                }
            };

        let next_hop = downstream_hop.to_string();
        let body = std::mem::take(req.body_mut());
        let body_bytes = match to_bytes(body, 2024) // MAX_BUFFER
            .await
        {
            Ok(body_bytes) => body_bytes,
            Err(_) => return (StatusCode::BAD_REQUEST, "body too big").into_response(),
        };
        let method = match Method::try_from(req.method()) {
            Ok(method) => method,
            Err(_) => return (StatusCode::BAD_REQUEST, "method not allowed").into_response(),
        };
        let res = state.client.request(method, next_hop).body(body_bytes).send().await;

        // Notify && transfer event
        // Notification
        // Create TransferEvent

        // forward request upstream
        match res {
            Ok(res) => Self::forward_response_helper(res),
            Err(_) => return (StatusCode::BAD_REQUEST, "peer connection problem").into_response(),
        }
    }

    pub fn forward_response_helper(reqwest_response: ReqwestResponse) -> Response {
        let status = reqwest_response.status();
        let headers = reqwest_response.headers().clone();
        let body_stream = reqwest_response.bytes_stream();
        let body = Body::from_stream(body_stream);
        let mut response = Response::builder().status(status);
        let response_headers = response.headers_mut().unwrap();
        for (key, value) in headers.iter() {
            response_headers.insert(key, value.clone());
        }

        response.body(body).unwrap()
    }
}
