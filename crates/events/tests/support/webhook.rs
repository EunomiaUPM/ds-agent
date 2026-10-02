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

//! Local webhook that records every request and answers with a fixed status.

use std::sync::{Arc, Mutex};

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::Router;
use tokio::net::TcpListener;

#[derive(Clone)]
pub struct Received {
    pub headers: HeaderMap,
    pub body: Bytes,
}

type Log = Arc<Mutex<Vec<Received>>>;

pub struct Webhook {
    pub url: String,
    received: Log,
}

impl Webhook {
    /// Serves `/hook` on a random local port, answering `status` to every POST.
    pub async fn start(status: StatusCode) -> Self {
        let received: Log = Arc::default();
        let app = Router::new()
            .route(
                "/hook",
                post(
                    move |State(log): State<Log>, headers: HeaderMap, body: Bytes| async move {
                        log.lock().unwrap().push(Received { headers, body });
                        status
                    },
                ),
            )
            .with_state(received.clone());
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        Self {
            url: format!("http://127.0.0.1:{port}/hook"),
            received,
        }
    }

    pub fn received(&self) -> Vec<Received> {
        self.received.lock().unwrap().clone()
    }
}
