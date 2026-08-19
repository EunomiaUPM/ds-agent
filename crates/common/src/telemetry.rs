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

use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

pub fn init(service_name: &str) {
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy()
        .add_directive("sqlx::query=off".parse().expect("static directive"));

    let json = std::env::var("LOG_FORMAT").as_deref() == Ok("json");

    let fmt = tracing_subscriber::fmt::layer()
        //.with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_span_events(FmtSpan::CLOSE)
        .with_line_number(true);

    let fmt: Box<dyn Layer<_> + Send + Sync> = if json {
        Box::new(fmt.json().with_current_span(true).flatten_event(true))
    } else {
        Box::new(fmt)
    };

    tracing_subscriber::registry().with(filter).with(fmt).init();

    tracing::info!(service = service_name, "telemetry initialised");
}
