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

use axum::{
    body::{to_bytes, Body},
    extract::Request as ExtractRequest,
    http::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};
use http::StatusCode;
use tracing;

pub async fn log_raw_body(req: ExtractRequest, next: Next) -> Response {
    // Split the request into its parts (headers, uri, etc.) and the body.
    let (parts, body) = req.into_parts();

    // Read the raw body immediately, BEFORE any extractor runs. 2 MB limit.
    let bytes = match to_bytes(body, 2 * 1024 * 1024).await {
        Ok(b) => b,
        Err(err) => {
            println!(">>> Error reading raw bytes: {}", err);
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    if let Ok(body_str) = std::str::from_utf8(&bytes) {
        println!(">>> RAW BODY: {}", body_str);
    } else {
        println!(">>> RAW BODY (binary): {:?}", bytes);
    }

    // Reconstruct the request with the bytes so downstream handlers can read it.
    let req = Request::from_parts(parts, Body::from(bytes));

    next.run(req).await
}

pub async fn log_body_middleware(req: Request<Body>, next: Next) -> Response {
    let (parts, body) = req.into_parts();

    let bytes = match to_bytes(body, 2 * 1024 * 1024).await {
        Ok(b) => b,
        Err(err) => {
            tracing::error!("Error reading body or limit exceeded: {}", err);
            return axum::http::StatusCode::PAYLOAD_TOO_LARGE.into_response();
        }
    };

    if let Ok(body_str) = std::str::from_utf8(&bytes) {
        tracing::info!("Request body: {}", body_str);
    } else {
        tracing::info!("Request body: [non-UTF-8 bytes]");
    }

    let req = Request::from_parts(parts, Body::from(bytes));

    next.run(req).await
}
