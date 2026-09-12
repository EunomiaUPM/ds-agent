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

use std::str::FromStr;
use std::sync::Arc;

use axum::extract::ws::WebSocketUpgrade;
use axum::extract::{Path, Query, Request, State};
use axum::http::{header, HeaderValue, StatusCode, Uri};
use axum::middleware;
use axum::response::{IntoResponse, Response};
use axum::routing::{any, delete, get, post, put};
use axum::{body::Body, Json, Router};
use common::config::services::GatewayConfig;
use common::config::types::traits::CommonConfigTrait;
use events::bus::envelope::{EventEnvelope, Topic};
use events::bus::{EventBus, EventBusTrait};
use events::data::repo::{CreateSubscriptionDto, UpdateSubscriptionDto};
use events::http::dlq_router::ListDlqQuery;
use events::http::events_router::PublishEventRequest;
use rust_embed::Embed;
use serde_json::{json, Value};
use tower_http::cors::{Any, CorsLayer};
use ymir::config::traits::HostsConfigTrait;
use ymir::config::types::HostType;

use crate::auth::BffAuthMiddleware;
use crate::events::feed_router::{BffEventFeedRouter, ListEventsQuery};
use crate::events::sse_handler::{SseQuery, SseStreamHandler};
use crate::events::ws_handler::BffWebSocketHandler;
use crate::gateway::service::{GatewayService, GatewayServiceTrait};
use crate::setup::context::AppContext;

#[derive(Embed)]
#[folder = "src/static/admin/dist"]
pub struct ReactApp;

/// Main HTTP router for the BFF gateway, proxying microservices and streaming events.
#[derive(Clone)]
pub struct GatewayHttpRouter {
    ctx: Arc<AppContext>,
    service: Arc<dyn GatewayServiceTrait>,
}

impl GatewayHttpRouter {
    /// Construct router using legacy GatewayConfig.
    pub fn new(config: GatewayConfig) -> Self {
        let events_ctx = events::setup::context::AppContext::in_memory(None);
        let ctx = Arc::new(AppContext::new(
            config.clone(),
            Some(events_ctx.event_bus),
            None,
        ));
        let service = Arc::new(GatewayService::new(config));
        Self { ctx, service }
    }

    /// Construct router using enterprise application context.
    pub fn with_context(ctx: Arc<AppContext>) -> Self {
        let service = Arc::new(GatewayService::new(ctx.config.clone()));
        Self { ctx, service }
    }

    /// Build the combined router with reverse proxy, events, websockets, and static fallback.
    pub fn router(self) -> Router {
        let cors = CorsLayer::new()
            .allow_methods(Any)
            .allow_origin(Any)
            .allow_headers(Any);

        let auth_middleware = BffAuthMiddleware::new(self.ctx.oauth_validator.clone(), false);

        let ws_handler = Arc::new(BffWebSocketHandler::new(
            self.ctx.event_bus.clone(),
            self.ctx.legacy_notification_tx.clone(),
        ));

        let api_router = Router::new()
            .route(
                "/ws",
                get({
                    let handler = ws_handler.clone();
                    move |ws: WebSocketUpgrade| {
                        let h = handler.clone();
                        async move { h.upgrade(ws).await }
                    }
                }),
            )
            .route("/fe-config", get(Self::config_handler))
            .route("/events/stream", get(Self::handle_sse_stream))
            .route(
                "/events/subscriptions",
                get(Self::handle_list_subscriptions).post(Self::handle_create_subscription),
            )
            .route(
                "/events/subscriptions/{id}",
                get(Self::handle_get_subscription)
                    .put(Self::handle_update_subscription)
                    .delete(Self::handle_delete_subscription),
            )
            .route("/events/dlq", get(Self::handle_list_dlq))
            .route("/events/dlq/replay-all", post(Self::handle_replay_all_dlq))
            .route(
                "/events/dlq/{id}",
                get(Self::handle_get_dlq).delete(Self::handle_delete_dlq),
            )
            .route("/events/dlq/{id}/replay", post(Self::handle_replay_dlq))
            .route(
                "/events",
                get(Self::handle_list_events).post(Self::handle_publish_event),
            )
            .route("/events/{id}", get(Self::handle_get_event))
            .route("/incoming-notification", post(Self::incoming_notification))
            .route(
                "/catalog-offerings",
                post(Self::handle_create_dataset_offering),
            )
            .route("/did-json/{url}", get(Self::fetch_did_json))
            .route(
                "/federated-catalog/{url}",
                get(Self::fetch_federated_catalog),
            )
            .route("/{service_prefix}", any(Self::proxy_handler_without_extra))
            .route(
                "/{service_prefix}/{*extra}",
                any(Self::proxy_handler_with_extra),
            )
            .route(
                "/dsp/current/{service_prefix}/{*extra}",
                any(Self::proxy_dsp_handler),
            )
            .route(
                "/well-known/rpc/{*extra}",
                any(Self::proxy_well_known_rpc_handler),
            );

        let protected_api = api_router
            .layer(middleware::from_fn(move |req, next| {
                let mw = auth_middleware.clone();
                async move { mw.handle(req, next).await }
            }))
            .with_state(self.clone());

        Router::new()
            .nest("/admin/api", protected_api.clone())
            .nest("/api", protected_api)
            .fallback(Self::static_path_handler)
            .layer(cors)
    }

    async fn handle_sse_stream(
        State(state): State<GatewayHttpRouter>,
        Query(q): Query<SseQuery>,
    ) -> Response {
        if let Some(bus) = &state.ctx.event_bus {
            SseStreamHandler::stream_events(bus.clone(), q).into_response()
        } else {
            (StatusCode::NOT_IMPLEMENTED, "event bus not configured").into_response()
        }
    }

    async fn handle_list_events(
        State(state): State<GatewayHttpRouter>,
        Query(q): Query<ListEventsQuery>,
    ) -> Response {
        if let Some(bus) = &state.ctx.event_bus {
            BffEventFeedRouter::handle_list(bus.clone(), q).await
        } else {
            (StatusCode::NOT_IMPLEMENTED, "event bus not configured").into_response()
        }
    }

    async fn handle_get_event(
        State(state): State<GatewayHttpRouter>,
        Path(id): Path<String>,
    ) -> Response {
        if let Some(bus) = &state.ctx.event_bus {
            BffEventFeedRouter::handle_get(bus.clone(), id).await
        } else {
            (StatusCode::NOT_IMPLEMENTED, "event bus not configured").into_response()
        }
    }

    async fn handle_publish_event(
        State(state): State<GatewayHttpRouter>,
        Json(req): Json<PublishEventRequest>,
    ) -> Response {
        let bus = match &state.ctx.event_bus {
            Some(b) => b,
            None => {
                return (StatusCode::NOT_IMPLEMENTED, "event bus not configured").into_response()
            }
        };

        let topic = match Topic::new(&req.topic) {
            Ok(t) => t,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error": format!("{e:?}") })),
                )
                    .into_response()
            }
        };

        let source = req.source_crate.unwrap_or_else(|| "bff".to_string());
        let correlation_id = req.correlation_id.and_then(|s| {
            urn::Urn::from_str(&s)
                .ok()
                .or_else(|| urn::Urn::from_str(&format!("urn:uuid:{s}")).ok())
        });
        let envelope = EventEnvelope::new(
            topic,
            source,
            req.schema_version.unwrap_or(1),
            correlation_id,
            req.payload,
        );

        match bus.publish(envelope).await {
            Ok(record) => (StatusCode::CREATED, Json(record)).into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("{e:?}") })),
            )
                .into_response(),
        }
    }

    async fn handle_list_subscriptions(State(state): State<GatewayHttpRouter>) -> Response {
        let bus = match &state.ctx.event_bus {
            Some(b) => b,
            None => {
                return (StatusCode::NOT_IMPLEMENTED, "event bus not configured").into_response()
            }
        };
        match bus.subscription_repo().list_subscriptions().await {
            Ok(subs) => (StatusCode::OK, Json(subs)).into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("{e:?}") })),
            )
                .into_response(),
        }
    }

    async fn handle_create_subscription(
        State(state): State<GatewayHttpRouter>,
        Json(dto): Json<CreateSubscriptionDto>,
    ) -> Response {
        let bus = match &state.ctx.event_bus {
            Some(b) => b,
            None => {
                return (StatusCode::NOT_IMPLEMENTED, "event bus not configured").into_response()
            }
        };
        match bus.subscription_repo().create_subscription(dto).await {
            Ok(sub) => (StatusCode::CREATED, Json(sub)).into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("{e:?}") })),
            )
                .into_response(),
        }
    }

    async fn handle_get_subscription(
        State(state): State<GatewayHttpRouter>,
        Path(id): Path<String>,
    ) -> Response {
        let bus = match &state.ctx.event_bus {
            Some(b) => b,
            None => {
                return (StatusCode::NOT_IMPLEMENTED, "event bus not configured").into_response()
            }
        };
        match bus.subscription_repo().get_subscription(&id).await {
            Ok(Some(sub)) => (StatusCode::OK, Json(sub)).into_response(),
            Ok(None) => (StatusCode::NOT_FOUND, "subscription not found").into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("{e:?}") })),
            )
                .into_response(),
        }
    }

    async fn handle_update_subscription(
        State(state): State<GatewayHttpRouter>,
        Path(id): Path<String>,
        Json(dto): Json<UpdateSubscriptionDto>,
    ) -> Response {
        let bus = match &state.ctx.event_bus {
            Some(b) => b,
            None => {
                return (StatusCode::NOT_IMPLEMENTED, "event bus not configured").into_response()
            }
        };
        match bus.subscription_repo().update_subscription(&id, dto).await {
            Ok(sub) => (StatusCode::OK, Json(sub)).into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("{e:?}") })),
            )
                .into_response(),
        }
    }

    async fn handle_delete_subscription(
        State(state): State<GatewayHttpRouter>,
        Path(id): Path<String>,
    ) -> Response {
        let bus = match &state.ctx.event_bus {
            Some(b) => b,
            None => {
                return (StatusCode::NOT_IMPLEMENTED, "event bus not configured").into_response()
            }
        };
        match bus.subscription_repo().delete_subscription(&id).await {
            Ok(()) => StatusCode::NO_CONTENT.into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("{e:?}") })),
            )
                .into_response(),
        }
    }

    async fn handle_list_dlq(
        State(state): State<GatewayHttpRouter>,
        Query(q): Query<ListDlqQuery>,
    ) -> Response {
        let bus = match &state.ctx.event_bus {
            Some(b) => b,
            None => {
                return (StatusCode::NOT_IMPLEMENTED, "event bus not configured").into_response()
            }
        };
        let limit = q.limit.unwrap_or(50).min(100);
        let offset = q.offset.unwrap_or(0);
        match bus
            .dlq_repo()
            .list_dead_letters(q.status.as_deref(), limit, offset)
            .await
        {
            Ok(records) => (StatusCode::OK, Json(records)).into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("{e:?}") })),
            )
                .into_response(),
        }
    }

    async fn handle_get_dlq(
        State(state): State<GatewayHttpRouter>,
        Path(id): Path<String>,
    ) -> Response {
        let bus = match &state.ctx.event_bus {
            Some(b) => b,
            None => {
                return (StatusCode::NOT_IMPLEMENTED, "event bus not configured").into_response()
            }
        };
        match bus.dlq_repo().get_dead_letter(&id).await {
            Ok(Some(record)) => (StatusCode::OK, Json(record)).into_response(),
            Ok(None) => (StatusCode::NOT_FOUND, "dead letter record not found").into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("{e:?}") })),
            )
                .into_response(),
        }
    }

    async fn handle_replay_dlq(
        State(state): State<GatewayHttpRouter>,
        Path(id): Path<String>,
    ) -> Response {
        let bus = match &state.ctx.event_bus {
            Some(b) => b,
            None => {
                return (StatusCode::NOT_IMPLEMENTED, "event bus not configured").into_response()
            }
        };
        match bus.replay_dead_letter(&id).await {
            Ok(delivery) => (StatusCode::OK, Json(delivery)).into_response(),
            Err(e) => (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": format!("{e:?}") })),
            )
                .into_response(),
        }
    }

    async fn handle_replay_all_dlq(State(state): State<GatewayHttpRouter>) -> Response {
        let bus = match &state.ctx.event_bus {
            Some(b) => b,
            None => {
                return (StatusCode::NOT_IMPLEMENTED, "event bus not configured").into_response()
            }
        };
        match bus.replay_all_dead_letters().await {
            Ok(count) => (StatusCode::OK, Json(json!({ "replayed_count": count }))).into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("{e:?}") })),
            )
                .into_response(),
        }
    }

    async fn handle_delete_dlq(
        State(state): State<GatewayHttpRouter>,
        Path(id): Path<String>,
    ) -> Response {
        let bus = match &state.ctx.event_bus {
            Some(b) => b,
            None => {
                return (StatusCode::NOT_IMPLEMENTED, "event bus not configured").into_response()
            }
        };
        match bus.dlq_repo().delete_dead_letter(&id).await {
            Ok(()) => StatusCode::NO_CONTENT.into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("{e:?}") })),
            )
                .into_response(),
        }
    }

    async fn static_path_handler(uri: Uri) -> impl IntoResponse {
        let mut path = uri.path().trim_start_matches('/').to_string();
        if path.is_empty() {
            path = "index.html".to_string();
        }

        match ReactApp::get(&path) {
            Some(content) => {
                let mime_type = mime_guess::from_path(&path).first_or_octet_stream();
                Response::builder()
                    .header(header::CONTENT_TYPE, mime_type.as_ref())
                    .body(Body::from(content.data))
                    .unwrap()
            }
            None => match ReactApp::get("index.html") {
                Some(content) => {
                    let mime_type = mime_guess::from_path("index.html").first_or_octet_stream();
                    Response::builder()
                        .header(header::CONTENT_TYPE, mime_type.as_ref())
                        .body(Body::from(content.data))
                        .unwrap()
                }
                None => Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::from("<h1>404</h1><p>index.html not found</p>"))
                    .unwrap(),
            },
        }
    }

    async fn config_handler(State(state): State<GatewayHttpRouter>) -> impl IntoResponse {
        let gateway_base = state.ctx.config.common().hosts.get_host(HostType::Http);
        let json = json!({
            "gateway_base": gateway_base,
        });
        (StatusCode::OK, Json(json)).into_response()
    }

    async fn proxy_handler_with_extra(
        State(state): State<GatewayHttpRouter>,
        Path((service_prefix, extra)): Path<(String, String)>,
        req: Request<Body>,
    ) -> impl IntoResponse {
        state
            .ctx
            .proxy
            .proxy_request(service_prefix, Some(extra), req)
            .await
    }

    async fn proxy_handler_without_extra(
        State(state): State<GatewayHttpRouter>,
        Path(service_prefix): Path<String>,
        req: Request<Body>,
    ) -> impl IntoResponse {
        state
            .ctx
            .proxy
            .proxy_request(service_prefix, None, req)
            .await
    }

    async fn proxy_dsp_handler(
        State(state): State<GatewayHttpRouter>,
        Path((service_prefix, extra)): Path<(String, String)>,
        req: Request<Body>,
    ) -> impl IntoResponse {
        state
            .ctx
            .proxy
            .proxy_dsp_request(service_prefix, Some(extra), req)
            .await
    }

    async fn proxy_well_known_rpc_handler(
        State(state): State<GatewayHttpRouter>,
        Path(extra): Path<String>,
        req: Request<Body>,
    ) -> impl IntoResponse {
        state
            .ctx
            .proxy
            .proxy_well_known_rpc_request(extra, req)
            .await
    }

    async fn incoming_notification(
        State(state): State<GatewayHttpRouter>,
        Json(input): Json<Value>,
    ) -> impl IntoResponse {
        let value_str = match serde_json::to_string(&input) {
            Ok(s) => s,
            Err(_) => {
                return (StatusCode::BAD_REQUEST, "invalid notification payload").into_response()
            }
        };

        if let Some(bus) = &state.ctx.event_bus {
            let topic = Topic::new("incoming.notification")
                .unwrap_or_else(|_| Topic::new("notification").unwrap());
            let envelope = EventEnvelope::new(topic, "bff", 1, None, input);
            let _ = bus.publish(envelope).await;
        }

        let _ = state.ctx.legacy_notification_tx.send(value_str);
        StatusCode::ACCEPTED.into_response()
    }

    async fn fetch_did_json(Path(url): Path<String>) -> impl IntoResponse {
        let target_url = format!("{}/api/v1/wallet/did.json", url.trim_end_matches('/'));
        match reqwest::get(&target_url).await {
            Ok(resp) => match resp.json::<Value>().await {
                Ok(json) => (StatusCode::OK, Json(json)).into_response(),
                Err(e) => (
                    StatusCode::BAD_GATEWAY,
                    format!("Failed to parse DID JSON: {e}"),
                )
                    .into_response(),
            },
            Err(e) => (
                StatusCode::BAD_GATEWAY,
                format!("Failed to fetch DID JSON: {e}"),
            )
                .into_response(),
        }
    }

    async fn fetch_federated_catalog(Path(url): Path<String>) -> impl IntoResponse {
        let target_url = format!(
            "{}/.well-known/federated-catalog",
            url.trim_end_matches('/')
        );
        match reqwest::get(&target_url).await {
            Ok(resp) => match resp.json::<Value>().await {
                Ok(json) => (StatusCode::OK, Json(json)).into_response(),
                Err(e) => (
                    StatusCode::BAD_GATEWAY,
                    format!("Failed to parse catalog JSON: {e}"),
                )
                    .into_response(),
            },
            Err(e) => (
                StatusCode::BAD_GATEWAY,
                format!("Failed to fetch federated catalog: {e}"),
            )
                .into_response(),
        }
    }

    async fn handle_create_dataset_offering(
        State(state): State<GatewayHttpRouter>,
        Json(req): Json<crate::gateway::dataset_offering::CreateDatasetOfferingRequest>,
    ) -> Response {
        crate::gateway::dataset_offering::orchestrate_dataset_offering(state.ctx.clone(), req).await
    }
}
