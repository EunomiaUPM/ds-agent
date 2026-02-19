use crate::gateway::service::{GatewayService, GatewayServiceTrait};
use axum::extract::ws::{Message, Utf8Bytes, WebSocketUpgrade};
use axum::extract::{FromRef, Path, Request, State};
use axum::http::{header, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::routing::{any, get, post};
use axum::{body::Body, Json, Router};
use rainbow_common::config::services::GatewayConfig;
use rainbow_common::config::traits::CommonConfigTrait;
use rust_embed::Embed;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};

#[derive(Embed)]
#[folder = "src/static/admin/dist"]
pub struct RainbowProviderReactApp;

#[derive(Clone)]
pub struct GatewayHttpRouter {
    service: Arc<dyn GatewayServiceTrait>,
    config: GatewayConfig,
}

impl FromRef<GatewayHttpRouter> for Arc<dyn GatewayServiceTrait> {
    fn from_ref(state: &GatewayHttpRouter) -> Self {
        state.service.clone()
    }
}

impl FromRef<GatewayHttpRouter> for GatewayConfig {
    fn from_ref(state: &GatewayHttpRouter) -> Self {
        state.config.clone()
    }
}

impl GatewayHttpRouter {
    pub fn new(config: GatewayConfig) -> Self {
        let service = Arc::new(GatewayService::new(config.clone()));
        Self { service, config }
    }

    pub fn router(self) -> Router {
        let cors = CorsLayer::new().allow_methods(Any).allow_origin(Any).allow_headers(Any);

        let api_router = Router::new()
            .route("/{service_prefix}/{*extra}", any(Self::proxy_handler_with_extra))
            .route("/dsp/current/{service_prefix}/{*extra}", any(Self::proxy_dsp_handler))
            .route("/well-known/rpc/{*extra}", any(Self::proxy_well_known_rpc_handler))
            .route("/did-json/{url}", get(Self::fetch_did_json))
            .route("/{service_prefix}", any(Self::proxy_handler_without_extra))
            .route("/ws", get(Self::websocket_handler))
            .route("/incoming-notification", post(Self::incoming_notification))
            .route("/fe-config", get(Self::config_handler));

        let router = Router::new().nest("/api", api_router);

        let router = router.nest_service(
            "/admin",
            ServeDir::new("./react/dist")
                .not_found_service(ServeFile::new("./react/dist/index.html")),
        );

        router.layer(cors).with_state(self)
    }

    async fn static_path_handler(uri: Uri) -> impl IntoResponse {
        let mut path = uri.path().trim_start_matches('/').to_string();
        if path.is_empty() {
            path = "index.html".to_string();
        }

        match RainbowProviderReactApp::get(&path) {
            Some(content) => {
                let mime_type = mime_guess::from_path(&path).first_or_octet_stream();
                Response::builder()
                    .header(header::CONTENT_TYPE, mime_type.as_ref())
                    .body(Body::from(content.data))
                    .unwrap()
            }
            None => match RainbowProviderReactApp::get("index.html") {
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

    async fn config_handler(State(config): State<GatewayConfig>) -> impl IntoResponse {
        let gateway_host = config.common().hosts.http.url.clone();
        let gateway_port = config.common().hosts.http.port.clone().unwrap_or("80".to_string());
        let json = json!({
            "gateway_host": gateway_host,
            "gateway_port": gateway_port,
        });
        (StatusCode::OK, Json(json).into_response())
    }

    async fn proxy_handler_with_extra(
        State(service): State<Arc<dyn GatewayServiceTrait>>,
        Path((service_prefix, extra)): Path<(String, String)>,
        req: Request<Body>,
    ) -> impl IntoResponse {
        service.proxy_request(service_prefix, Some(extra), req).await
    }

    async fn proxy_handler_without_extra(
        State(service): State<Arc<dyn GatewayServiceTrait>>,
        Path(service_prefix): Path<String>,
        req: Request<Body>,
    ) -> impl IntoResponse {
        service.proxy_request(service_prefix, None, req).await
    }

    async fn proxy_dsp_handler(
        State(service): State<Arc<dyn GatewayServiceTrait>>,
        Path((service_prefix, extra)): Path<(String, String)>,
        req: Request<Body>,
    ) -> impl IntoResponse {
        service.proxy_dsp_request(service_prefix, Some(extra), req).await
    }

    async fn proxy_well_known_rpc_handler(
        State(service): State<Arc<dyn GatewayServiceTrait>>,
        Path(extra): Path<String>,
        req: Request<Body>,
    ) -> impl IntoResponse {
        service.proxy_well_known_rpc_request(extra, req).await
    }

    async fn websocket_handler(
        State(service): State<Arc<dyn GatewayServiceTrait>>,
        ws: WebSocketUpgrade,
    ) -> impl IntoResponse {
        ws.on_upgrade(move |mut socket| async move {
            let mut notification_rx = service.get_notification_sender().subscribe();
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
                            Message::Binary(_) => {
                                println!("Received binary message from client.");
                            }
                            Message::Ping(ping) => {
                                if socket.send(Message::Pong(ping)).await.is_err() {
                                   break;
                                }
                            }
                            Message::Pong(_) => {
                            }
                            Message::Close(_) => {
                                eprintln!("WS client initiated close.");
                                break;
                            }
                        }
                    }
                    else => {
                        eprintln!("WS connection or broadcast channel error/closed.");
                        break;
                    }
                }
            }
            println!("WebSocket connection handler finished.");
        })
    }

    async fn incoming_notification(
        State(service): State<Arc<dyn GatewayServiceTrait>>,
        Json(input): Json<Value>,
    ) -> impl IntoResponse {
        let value_str = match serde_json::to_string(&input) {
            Ok(value_str) => value_str,
            Err(_e) => return (StatusCode::BAD_REQUEST, "Not able to deserialize").into_response(),
        };
        match service.get_notification_sender().send(value_str) {
            Ok(_) => StatusCode::ACCEPTED.into_response(),
            Err(_e) => (StatusCode::BAD_REQUEST, "Not able to deserialize").into_response(),
        }
    }

    async fn fetch_did_json(Path(url): Path<String>) -> impl IntoResponse {
        let target_url = format!("{}/api/v1/wallet/did.json", url.trim_end_matches('/'));
        match reqwest::get(&target_url).await {
            Ok(resp) => match resp.json::<Value>().await {
                Ok(json) => (StatusCode::OK, Json(json)).into_response(),
                Err(e) => (
                    StatusCode::BAD_GATEWAY,
                    format!("Failed to parse JSON from {}: {}", target_url, e),
                )
                    .into_response(),
            },
            Err(e) => (
                StatusCode::BAD_GATEWAY,
                format!("Failed to fetch from {}: {}", target_url, e),
            )
                .into_response(),
        }
    }
}
