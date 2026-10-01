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

//! Process telemetry: logs always; OTLP traces and metrics when `OTEL_EXPORTER_OTLP_ENDPOINT`
//! is set. Every other knob comes from the standard `OTEL_*` variables.

use std::collections::HashMap;

use opentelemetry::propagation::TextMapCompositePropagator;
use opentelemetry::trace::{TraceContextExt, TracerProvider as _};
use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::{MetricExporter, SpanExporter};
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::propagation::{BaggagePropagator, TraceContextPropagator};
use opentelemetry_sdk::resource::{EnvResourceDetector, TelemetryResourceDetector};
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::Resource;
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

const OTLP_ENDPOINT_ENV: &str = "OTEL_EXPORTER_OTLP_ENDPOINT";

/// Guard of the OTel providers; `shutdown` flushes pending spans and metrics.
pub struct Telemetry {
    tracer: Option<SdkTracerProvider>,
    meter: Option<SdkMeterProvider>,
}

impl Telemetry {
    /// Installs the global subscriber. Must run inside the Tokio runtime (OTLP uses tonic).
    pub fn init(service_name: &str) -> Self {
        let filter = EnvFilter::builder()
            .with_default_directive(LevelFilter::INFO.into())
            .from_env_lossy()
            .add_directive("sqlx::query=off".parse().expect("static directive"))
            // The exporter's own transport must not trace itself.
            .add_directive("h2=off".parse().expect("static directive"))
            .add_directive("opentelemetry=warn".parse().expect("static directive"));

        let json = std::env::var("LOG_FORMAT").as_deref() == Ok("json");
        let fmt = tracing_subscriber::fmt::layer()
            .with_span_events(FmtSpan::CLOSE)
            .with_line_number(true);
        let fmt: Box<dyn Layer<_> + Send + Sync> = if json {
            Box::new(fmt.json().with_current_span(true).flatten_event(true))
        } else {
            Box::new(fmt)
        };

        let telemetry = match std::env::var(OTLP_ENDPOINT_ENV) {
            Ok(_) => Self::otlp(service_name),
            Err(_) => Self {
                tracer: None,
                meter: None,
            },
        };
        let otel = telemetry.tracer.as_ref().map(|provider| {
            tracing_opentelemetry::layer().with_tracer(provider.tracer(service_name.to_string()))
        });

        tracing_subscriber::registry()
            .with(filter)
            .with(fmt)
            .with(otel)
            .init();

        tracing::info!(
            service = service_name,
            otlp = telemetry.tracer.is_some(),
            "telemetry initialised"
        );
        telemetry
    }

    /// Flushes and stops the exporters; call once, after the process stops serving.
    pub fn shutdown(&self) {
        if let Some(tracer) = &self.tracer {
            if let Err(e) = tracer.shutdown() {
                eprintln!("tracer provider shutdown failed: {e}");
            }
        }
        if let Some(meter) = &self.meter {
            if let Err(e) = meter.shutdown() {
                eprintln!("meter provider shutdown failed: {e}");
            }
        }
    }

    /// Builds both providers; an exporter that cannot be built only disables its signal.
    fn otlp(service_name: &str) -> Self {
        global::set_text_map_propagator(TextMapCompositePropagator::new(vec![
            Box::new(TraceContextPropagator::new()),
            Box::new(BaggagePropagator::new()),
        ]));
        let resource = Self::resource(service_name);

        let tracer = match Self::signal_enabled("OTEL_TRACES_EXPORTER")
            .then(|| SpanExporter::builder().with_tonic().build())
        {
            None => None,
            Some(Ok(exporter)) => {
                let provider = SdkTracerProvider::builder()
                    .with_batch_exporter(exporter)
                    .with_resource(resource.clone())
                    .build();
                global::set_tracer_provider(provider.clone());
                Some(provider)
            }
            Some(Err(e)) => {
                eprintln!("OTLP span exporter disabled: {e}");
                None
            }
        };
        let meter = match Self::signal_enabled("OTEL_METRICS_EXPORTER")
            .then(|| MetricExporter::builder().with_tonic().build())
        {
            None => None,
            Some(Ok(exporter)) => {
                let provider = SdkMeterProvider::builder()
                    .with_periodic_exporter(exporter)
                    .with_resource(resource)
                    .build();
                global::set_meter_provider(provider.clone());
                Some(provider)
            }
            Some(Err(e)) => {
                eprintln!("OTLP metric exporter disabled: {e}");
                None
            }
        };
        Self { tracer, meter }
    }

    /// `OTEL_TRACES_EXPORTER=none` / `OTEL_METRICS_EXPORTER=none` switch a signal off.
    fn signal_enabled(env: &str) -> bool {
        std::env::var(env).map_or(true, |v| v != "none")
    }

    /// `OTEL_SERVICE_NAME` and `OTEL_RESOURCE_ATTRIBUTES` override these defaults.
    fn resource(service_name: &str) -> Resource {
        let name = std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| service_name.to_string());
        Resource::builder_empty()
            .with_service_name(name)
            .with_attributes([
                KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                KeyValue::new("service.instance.id", uuid::Uuid::new_v4().to_string()),
            ])
            .with_detectors(&[
                Box::new(TelemetryResourceDetector),
                Box::new(EnvResourceDetector::new()),
            ])
            .build()
    }
}

/// W3C `traceparent` of work that outlives its request (events, queued retries).
pub struct TraceParent;

impl TraceParent {
    const HEADER: &'static str = "traceparent";

    /// The current span's `traceparent`; `None` without an OTel layer or a sampled span.
    pub fn current() -> Option<String> {
        let cx = Span::current().context();
        let mut carrier = HashMap::new();
        global::get_text_map_propagator(|p| p.inject_context(&cx, &mut carrier));
        carrier.remove(Self::HEADER)
    }

    /// Links `span` to the trace that produced `traceparent`, keeping its own parent.
    pub fn link(span: &Span, traceparent: Option<&str>) {
        let Some(traceparent) = traceparent else {
            return;
        };
        let carrier = HashMap::from([(Self::HEADER.to_string(), traceparent.to_string())]);
        let cx = global::get_text_map_propagator(|p| p.extract(&carrier));
        let origin = cx.span().span_context().clone();
        if origin.is_valid() {
            span.add_link(origin);
        }
    }
}
