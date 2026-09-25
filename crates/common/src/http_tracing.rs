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

//! Server spans for the HTTP and gRPC planes: W3C trace context in, OTel semantic
//! attributes on the span, and one request-duration histogram per plane.

use std::sync::LazyLock;
use std::time::{Duration, Instant};

use axum::body::Body;
use axum::extract::MatchedPath;
use axum::middleware::Next;
use axum::response::Response as AxumResponse;
use http::{HeaderMap, Request, Response};
use opentelemetry::metrics::Histogram;
use opentelemetry::propagation::Extractor;
use opentelemetry::trace::TraceContextExt;
use opentelemetry::{global, KeyValue};
use tower_http::trace::{
    GrpcMakeClassifier, HttpMakeClassifier, MakeSpan, OnRequest, OnResponse, TraceLayer,
};
use tracing::field::Empty;
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use uuid::Uuid;

/// Read-only view of request headers for the global propagator.
pub struct TraceHeaders<'a>(pub &'a HeaderMap);

impl Extractor for TraceHeaders<'_> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|v| v.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(|k| k.as_str()).collect()
    }
}

#[derive(Clone, Copy)]
enum Plane {
    Http,
    Grpc,
}

/// Opens the server span, parented on the caller's `traceparent` when present.
#[derive(Clone, Copy)]
pub struct ServerSpan {
    plane: Plane,
}

impl<B> MakeSpan<B> for ServerSpan {
    fn make_span(&mut self, req: &Request<B>) -> Span {
        let request_id = req
            .headers()
            .get("x-request-id")
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned)
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let span = match self.plane {
            Plane::Http => {
                // Routed paths keep span names low-cardinality; unmatched ones fall back to the method.
                let route = req
                    .extensions()
                    .get::<MatchedPath>()
                    .map(|p| p.as_str().to_owned());
                let name = match &route {
                    Some(route) => format!("{} {route}", req.method()),
                    None => req.method().to_string(),
                };
                tracing::info_span!(
                    "request",
                    otel.name = %name,
                    otel.kind = "server",
                    otel.status_code = Empty,
                    id = %request_id,
                    trace_id = Empty,
                    http.request.method = %req.method(),
                    http.route = route.as_deref().unwrap_or_default(),
                    url.path = %req.uri().path(),
                    http.response.status_code = Empty,
                )
            }
            Plane::Grpc => {
                let path = req.uri().path().trim_start_matches('/');
                let (service, method) = path.split_once('/').unwrap_or((path, ""));
                tracing::info_span!(
                    "grpc",
                    otel.name = %path,
                    otel.kind = "server",
                    otel.status_code = Empty,
                    id = %request_id,
                    trace_id = Empty,
                    rpc.system = "grpc",
                    rpc.service = %service,
                    rpc.method = %method,
                    rpc.grpc.status_code = Empty,
                )
            }
        };
        let parent = global::get_text_map_propagator(|p| p.extract(&TraceHeaders(req.headers())));
        let _ = span.set_parent(parent);
        let trace_id = span.context().span().span_context().trace_id();
        if trace_id != opentelemetry::trace::TraceId::INVALID {
            span.record("trace_id", tracing::field::display(trace_id));
        }
        span
    }
}

impl<B> OnRequest<B> for ServerSpan {
    fn on_request(&mut self, req: &Request<B>, _span: &Span) {
        tracing::info!("{} {}", req.method(), req.uri());
    }
}

/// Records the outcome on the span; gRPC latency also lands on its histogram here.
#[derive(Clone)]
pub struct ServerOutcome {
    plane: Plane,
    grpc_duration: Option<Histogram<f64>>,
}

impl<B> OnResponse<B> for ServerOutcome {
    fn on_response(self, res: &Response<B>, latency: Duration, span: &Span) {
        let status = res.status();
        match self.plane {
            Plane::Http => {
                span.record("http.response.status_code", status.as_u16());
                if status.is_server_error() {
                    span.record("otel.status_code", "ERROR");
                }
            }
            Plane::Grpc => {
                // Trailers-only responses carry the status in the headers.
                let code = res
                    .headers()
                    .get("grpc-status")
                    .and_then(|v| v.to_str().ok());
                if let Some(code) = code {
                    span.record("rpc.grpc.status_code", code);
                    if code != "0" {
                        span.record("otel.status_code", "ERROR");
                    }
                }
                if let Some(histogram) = &self.grpc_duration {
                    let attributes: Vec<_> = code
                        .map(|c| KeyValue::new("rpc.grpc.status_code", c.to_owned()))
                        .into_iter()
                        .collect();
                    histogram.record(latency.as_secs_f64(), &attributes);
                }
            }
        }
        tracing::info!(
            status = status.as_u16(),
            latency_ms = latency.as_millis() as u64,
            "response"
        );
    }
}

/// Resolved on first request, which always follows `Telemetry::init`.
static HTTP_DURATION: LazyLock<Histogram<f64>> =
    LazyLock::new(|| HttpTracing::histogram("http.server.request.duration"));

pub struct HttpTracing;

impl HttpTracing {
    /// Layer for the composed axum router; `Router::layer` runs after routing, so
    /// `MatchedPath` is already in the request extensions.
    pub fn http_layer() -> TraceLayer<HttpMakeClassifier, ServerSpan, ServerSpan, ServerOutcome> {
        let plane = Plane::Http;
        TraceLayer::new_for_http()
            .make_span_with(ServerSpan { plane })
            .on_request(ServerSpan { plane })
            .on_response(ServerOutcome {
                plane,
                grpc_duration: None,
            })
    }

    /// Middleware recording `http.server.request.duration` by method, route and status;
    /// mount it with `Router::layer` so `MatchedPath` is known.
    pub async fn record_http_duration(req: Request<Body>, next: Next) -> AxumResponse {
        let method = req.method().to_string();
        let route = req
            .extensions()
            .get::<MatchedPath>()
            .map(|p| p.as_str().to_owned());
        let started = Instant::now();
        let res = next.run(req).await;
        let mut attributes = vec![
            KeyValue::new("http.request.method", method),
            KeyValue::new("http.response.status_code", res.status().as_u16() as i64),
        ];
        if let Some(route) = route {
            attributes.push(KeyValue::new("http.route", route));
        }
        HTTP_DURATION.record(started.elapsed().as_secs_f64(), &attributes);
        res
    }

    pub fn grpc_layer() -> TraceLayer<GrpcMakeClassifier, ServerSpan, ServerSpan, ServerOutcome> {
        let plane = Plane::Grpc;
        TraceLayer::new_for_grpc()
            .make_span_with(ServerSpan { plane })
            .on_request(ServerSpan { plane })
            .on_response(ServerOutcome {
                plane,
                grpc_duration: Some(Self::histogram("rpc.server.call.duration")),
            })
    }

    /// Built after `Telemetry::init`, so it binds to the real meter provider (or the no-op one).
    fn histogram(name: &'static str) -> Histogram<f64> {
        global::meter("common")
            .f64_histogram(name)
            .with_unit("s")
            .build()
    }
}
