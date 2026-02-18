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
 *  * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */
use crate::gateway::execute_proxy;
use axum::body::Body;
use axum::extract::ws::{Message, Utf8Bytes, WebSocketUpgrade};
use axum::extract::{Path, Request, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{any, get, post};
use axum::{Json, Router};
use rainbow_common::config::services::GatewayConfig;
use rainbow_common::config::traits::CommonConfigTrait;
use reqwest::Client;
use serde_json::{json, Value};
use std::time::Duration;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use ymir::config::types::HostType;

pub struct GatewayRouter {
    config: GatewayConfig,
    client: Client,
    notification_tx: broadcast::Sender<String>,
}

impl GatewayRouter {
    pub fn new(config: GatewayConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to build reqwest client");
        let (notification_tx, _) = broadcast::channel(100);
        Self { config, client, notification_tx }
    }

    pub fn router(self) -> Router {
        let cors = CorsLayer::new().allow_methods(Any).allow_origin(Any).allow_headers(Any);

        let router = Router::new()
            .route("/admin/api/{service_prefix}/{*extra}", any(Self::proxy_handler_with_extra))
            .route("/admin/api/{service_prefix}", any(Self::proxy_handler_without_extra))
            .route("/admin/api/ws", get(Self::websocket_handler))
            .route("/admin/api/incoming-notification", post(Self::incoming_notification))
            .route("/admin/api/fe-config", get(Self::config_handler))
            .nest_service(
                "/admin",
                ServeDir::new("./react/dist")
                    .not_found_service(ServeFile::new("./react/dist/index.html")),
            );

        router.layer(cors).with_state((self.config, self.client, self.notification_tx))
    }

    async fn config_handler(
        State((config, _client, _notification_tx)): State<(
            GatewayConfig,
            Client,
            broadcast::Sender<String>,
        )>,
    ) -> impl IntoResponse {
        let gateway_host = config.common().hosts.http.url.clone();
        let gateway_port = config.common().hosts.http.port.clone().unwrap_or("80".to_string());
        let json = json!({
            "gateway_host": gateway_host,
            "gateway_port": gateway_port,
        });
        (StatusCode::OK, Json(json).into_response())
    }

    async fn proxy_handler_with_extra(
        State((config, client, notification_tx)): State<(
            GatewayConfig,
            Client,
            broadcast::Sender<String>,
        )>,
        Path((service_prefix, extra)): Path<(String, String)>,
        req: Request<Body>,
    ) -> impl IntoResponse {
        Self::execute_proxy((config, client, notification_tx), service_prefix, Some(extra), req)
            .await
    }

    async fn proxy_handler_without_extra(
        State((config, client, notification_tx)): State<(
            GatewayConfig,
            Client,
            broadcast::Sender<String>,
        )>,
        Path(service_prefix): Path<String>,
        req: Request<Body>,
    ) -> impl IntoResponse {
        Self::execute_proxy((config, client, notification_tx), service_prefix, None, req).await
    }

    async fn execute_proxy(
        (config, client, _notification_tx): (GatewayConfig, Client, broadcast::Sender<String>),
        service_prefix: String,
        extra_opt: Option<String>,
        req: Request<Body>,
    ) -> impl IntoResponse {
        let microservice_base_url = match service_prefix.as_str() {
            "dataplane" => config.transfer().get_host(HostType::Http),
            "connector" => config.catalog().get_host(HostType::Http),
            "catalogs" => config.catalog().get_host(HostType::Http),
            "datasets" => config.catalog().get_host(HostType::Http),
            "notifications" => config.transfer().get_host(HostType::Http),
            "data-services" => config.transfer().get_host(HostType::Http),
            "distributions" => config.transfer().get_host(HostType::Http),
            "odrl-policies" => config.transfer().get_host(HostType::Http),
            "datahub" => config.catalog().get_host(HostType::Http),
            "contract-negotiation" => config.contracts().get_host(HostType::Http),
            "mates" => config.ssi_auth().get_host(HostType::Http),
            "negotiations" => config.catalog().get_host(HostType::Http),
            "transfers" => config.transfer().get_host(HostType::Http),
            "auth" => config.ssi_auth().get_host(HostType::Http),
            "wallet" => config.ssi_auth().get_host(HostType::Http),
            "ssi-auth" => config.ssi_auth().get_host(HostType::Http),
            "subscriptions" => config.transfer().get_host(HostType::Http),
            _ => return (StatusCode::NOT_FOUND, "prefix not found").into_response(),
        };

        execute_proxy(client, microservice_base_url, service_prefix, extra_opt, req).await
    }

    async fn websocket_handler(
        State((_config, _client, notification_tx)): State<(
            GatewayConfig,
            Client,
            broadcast::Sender<String>,
        )>,
        ws: WebSocketUpgrade,
    ) -> impl IntoResponse {
        ws.on_upgrade(move |mut socket| async move {
            let mut notification_rx = notification_tx.subscribe();
            loop {
                tokio::select! {
                    Ok(msg_to_send) = notification_rx.recv() => {
                        if socket.send(Message::Text(Utf8Bytes::from(msg_to_send))).await.is_err() {
                            eprintln!("WS client disconnected or send error.");
                            break;
                        }
                    }
                    Some(Ok(ws_msg)) = socket.recv() => {
                        match ws_msg {
                            Message::Text(text) => {
                                println!("Received WS message from client: {}", text);
                            }
                            Message::Binary(_) => println!("Received binary message from client."),
                            Message::Ping(ping) => {
                                if socket.send(Message::Pong(ping)).await.is_err() { break; }
                            }
                            Message::Pong(_) => {}
                            Message::Close(_) => { eprintln!("WS client initiated close."); break; }
                        }
                    }
                    else => { eprintln!("WS connection or broadcast channel error/closed."); break; }
                }
            }
            println!("WebSocket connection handler finished.");
        })
    }

    async fn incoming_notification(
        State((_config, _client, notification_tx)): State<(
            GatewayConfig,
            Client,
            broadcast::Sender<String>,
        )>,
        Json(input): Json<Value>,
    ) -> impl IntoResponse {
        let value_str = match serde_json::to_string(&input) {
            Ok(value_str) => value_str,
            Err(_) => return (StatusCode::BAD_REQUEST, "Not able to deserialize").into_response(),
        };
        match notification_tx.send(value_str) {
            Ok(_) => StatusCode::ACCEPTED.into_response(),
            Err(_) => (StatusCode::BAD_REQUEST, "Not able to send notification").into_response(),
        }
    }
}
