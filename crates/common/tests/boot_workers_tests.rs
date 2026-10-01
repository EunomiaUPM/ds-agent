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

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use ymir::services::client::ClientTrait;
use ymir::utils::http_client;

use axum::routing::get;
use axum::Router;
use common::boot::servers::HttpServer;
use common::boot::workers::{BackgroundWorker, WorkerSet};
use common::module_loader::module_group::ModuleGroup;
use common::module_loader::service_composer::ServiceComposer;
use common::module_loader::service_module::ServiceModuleTrait;
use tokio_util::sync::CancellationToken;
use ymir::errors::{Errors, Outcome};

/// Runs until cancelled, recording that it saw the cancellation.
struct UntilCancelled {
    stopped: Arc<AtomicBool>,
}

#[async_trait::async_trait]
impl BackgroundWorker for UntilCancelled {
    fn name(&self) -> &'static str {
        "until-cancelled"
    }

    async fn run(self: Box<Self>, token: CancellationToken) -> Outcome<()> {
        token.cancelled().await;
        self.stopped.store(true, Ordering::SeqCst);
        Ok(())
    }
}

/// Stops on its own with the given result.
struct Finishes(Outcome<()>);

#[async_trait::async_trait]
impl BackgroundWorker for Finishes {
    fn name(&self) -> &'static str {
        "finishes"
    }

    async fn run(self: Box<Self>, _token: CancellationToken) -> Outcome<()> {
        self.0
    }
}

struct Panics;

#[async_trait::async_trait]
impl BackgroundWorker for Panics {
    fn name(&self) -> &'static str {
        "panics"
    }

    async fn run(self: Box<Self>, _token: CancellationToken) -> Outcome<()> {
        panic!("boom")
    }
}

/// Ignores cancellation entirely.
struct Stubborn;

#[async_trait::async_trait]
impl BackgroundWorker for Stubborn {
    fn name(&self) -> &'static str {
        "stubborn"
    }

    async fn run(self: Box<Self>, _token: CancellationToken) -> Outcome<()> {
        std::future::pending().await
    }
}

/// Module contributing one worker per call.
struct WorkerModule(&'static str);

impl ServiceModuleTrait for WorkerModule {
    fn name(&self) -> &'static str {
        self.0
    }

    fn workers(&self) -> Vec<Box<dyn BackgroundWorker>> {
        vec![Box::new(Finishes(Ok(())))]
    }
}

fn until_cancelled() -> (Box<UntilCancelled>, Arc<AtomicBool>) {
    let stopped = Arc::new(AtomicBool::new(false));
    (
        Box::new(UntilCancelled {
            stopped: stopped.clone(),
        }),
        stopped,
    )
}

#[tokio::test]
async fn wait_any_reports_worker_that_stops_on_its_own() {
    let mut workers = WorkerSet::new(CancellationToken::new());
    let (long, _) = until_cancelled();
    workers.spawn(long);
    workers.spawn(Box::new(Finishes(Ok(()))));

    let exit = workers.wait_any().await;

    assert_eq!(exit.name, "finishes");
    assert!(exit.result.is_ok());
    assert_eq!(workers.len(), 1);
}

#[tokio::test]
async fn wait_any_reports_worker_failure() {
    let mut workers = WorkerSet::new(CancellationToken::new());
    workers.spawn(Box::new(Finishes(Err(Errors::crazy("down", None)))));

    let exit = workers.wait_any().await;

    assert_eq!(exit.name, "finishes");
    assert!(exit.result.is_err());
}

#[tokio::test]
async fn wait_any_reports_panic_with_worker_name() {
    let mut workers = WorkerSet::new(CancellationToken::new());
    workers.spawn(Box::new(Panics));

    let exit = workers.wait_any().await;

    assert_eq!(exit.name, "panics");
    assert!(exit.result.is_err());
}

#[tokio::test]
async fn wait_any_pends_while_set_is_empty() {
    let mut workers = WorkerSet::new(CancellationToken::new());

    let waited = tokio::time::timeout(Duration::from_millis(50), workers.wait_any()).await;

    assert!(waited.is_err());
}

#[tokio::test]
async fn shutdown_cancels_and_awaits_every_worker() {
    let token = CancellationToken::new();
    let mut workers = WorkerSet::new(token.clone());
    let (first, first_stopped) = until_cancelled();
    let (second, second_stopped) = until_cancelled();
    workers.spawn_all([first as Box<dyn BackgroundWorker>, second]);

    workers.shutdown(Duration::from_secs(1)).await;

    assert!(token.is_cancelled());
    assert!(first_stopped.load(Ordering::SeqCst));
    assert!(second_stopped.load(Ordering::SeqCst));
}

#[tokio::test]
async fn shutdown_aborts_workers_past_grace() {
    let mut workers = WorkerSet::new(CancellationToken::new());
    workers.spawn(Box::new(Stubborn));

    let finished = tokio::time::timeout(
        Duration::from_secs(5),
        workers.shutdown(Duration::from_millis(50)),
    )
    .await;

    assert!(finished.is_ok());
}

#[test]
fn composer_collects_workers_from_nested_groups() {
    let composer = ServiceComposer::new().register(WorkerModule("a")).register(
        ModuleGroup::new("group")
            .register(WorkerModule("b"))
            .register(WorkerModule("c")),
    );

    assert_eq!(composer.workers().len(), 3);
}

#[tokio::test]
async fn http_server_serves_until_cancelled() {
    let router = Router::new().route("/ping", get(|| async { "pong" }));
    let server = HttpServer::bind("0".to_string(), router, None)
        .await
        .unwrap();
    let addr = server.local_addr().unwrap();
    let token = CancellationToken::new();
    let mut workers = WorkerSet::new(token.clone());
    workers.spawn(Box::new(server));

    let body = http_client()
        .get(&format!("http://{addr}/ping"), None)
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert_eq!(body, "pong");

    token.cancel();
    let exit = tokio::time::timeout(Duration::from_secs(5), workers.wait_any())
        .await
        .expect("server stops once cancelled");
    assert_eq!(exit.name, "http");
    assert!(exit.result.is_ok());
}
