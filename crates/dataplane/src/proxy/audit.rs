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

//! Non-blocking bounded queue and background batch writer for `DataProxy` transfer audit events.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use events::{EventEnvelope, EventSinkTrait, NoopEventSink};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{error, warn};
use ymir::boot::workers::BackgroundWorker;
use ymir::errors::Outcome;

use crate::data::factory_trait::DataplaneRepoTrait;
use crate::data::sea_orm::orm::transfer_event::NewTransferEvent;

enum ProxyAuditItem {
    TransferEvent(NewTransferEvent),
    TelemetryEnvelope(EventEnvelope),
}

/// Non-blocking handle used by `HttpDataProxy` to enqueue diagnostic events off request paths.
#[derive(Clone)]
pub struct ProxyAuditSender {
    tx: mpsc::Sender<ProxyAuditItem>,
    dropped: Arc<AtomicU64>,
}

impl ProxyAuditSender {
    pub fn try_send(&self, event: NewTransferEvent) {
        self.enqueue(ProxyAuditItem::TransferEvent(event));
    }

    /// Enqueues a canonical dataplane event envelope for batched background emission.
    pub fn try_send_envelope(&self, envelope: EventEnvelope) {
        self.enqueue(ProxyAuditItem::TelemetryEnvelope(envelope));
    }

    fn enqueue(&self, item: ProxyAuditItem) {
        if self.tx.try_send(item).is_err() {
            let count = self.dropped.fetch_add(1, Ordering::Relaxed) + 1;
            if count == 1 || count.is_multiple_of(256) {
                warn!(dropped = count, "Dataplane audit event queue full; dropping event");
            }
        }
    }
}

/// Drains queued transfer logs and canonical envelopes and flushes them in batches.
pub struct ProxyAuditWorker {
    repo: Arc<dyn DataplaneRepoTrait>,
    events: Arc<dyn EventSinkTrait>,
    rx: mpsc::Receiver<ProxyAuditItem>,
    batch_size: usize,
}

impl ProxyAuditWorker {
    pub const DEFAULT_CAPACITY: usize = 4096;
    pub const DEFAULT_BATCH_SIZE: usize = 128;

    /// Returns the non-blocking sender and paired background batch worker.
    pub fn open(repo: Arc<dyn DataplaneRepoTrait>, capacity: usize) -> (ProxyAuditSender, Self) {
        let (tx, rx) = mpsc::channel(capacity.max(1));
        let sender = ProxyAuditSender {
            tx,
            dropped: Arc::new(AtomicU64::new(0)),
        };
        let worker = Self {
            repo,
            events: Arc::new(NoopEventSink),
            rx,
            batch_size: Self::DEFAULT_BATCH_SIZE,
        };
        (sender, worker)
    }

    /// Attaches the canonical event sink used to flush queued dataplane envelopes in batches.
    pub fn with_events(mut self, events: Arc<dyn EventSinkTrait>) -> Self {
        self.events = events;
        self
    }

    fn push_item(
        item: ProxyAuditItem,
        transfer_events: &mut Vec<NewTransferEvent>,
        envelopes: &mut Vec<EventEnvelope>,
    ) {
        match item {
            ProxyAuditItem::TransferEvent(ev) => transfer_events.push(ev),
            ProxyAuditItem::TelemetryEnvelope(env) => envelopes.push(env),
        }
    }

    async fn flush_batch(&mut self, first: ProxyAuditItem) {
        let mut transfer_events = Vec::with_capacity(self.batch_size);
        let mut envelopes = Vec::with_capacity(self.batch_size);
        Self::push_item(first, &mut transfer_events, &mut envelopes);
        let mut count = 1;
        while count < self.batch_size {
            match self.rx.try_recv() {
                Ok(next) => {
                    Self::push_item(next, &mut transfer_events, &mut envelopes);
                    count += 1;
                }
                Err(_) => break,
            }
        }
        tokio::join!(
            self.flush_transfer_events(transfer_events),
            self.flush_envelopes(envelopes),
        );
    }

    async fn flush_transfer_events(&self, batch: Vec<NewTransferEvent>) {
        if batch.is_empty() {
            return;
        }
        if let Err(err) = self
            .repo
            .get_transfer_events_repo()
            .create_transfer_events_batch(&batch)
            .await
        {
            error!(count = batch.len(), "Failed to log transfer event batch: {err:?}");
        }
    }

    async fn flush_envelopes(&self, batch: Vec<EventEnvelope>) {
        if batch.is_empty() {
            return;
        }
        let count = batch.len();
        if let Err(err) = self.events.emit_envelopes(batch).await {
            warn!(count, error = %err, "Failed to emit dataplane audit envelope batch");
        }
    }

    async fn drain_remaining(&mut self) {
        while let Ok(first) = self.rx.try_recv() {
            self.flush_batch(first).await;
        }
    }
}

#[async_trait::async_trait]
impl BackgroundWorker for ProxyAuditWorker {
    fn name(&self) -> &'static str {
        "dataplane-proxy-audit"
    }

    async fn run(mut self: Box<Self>, token: CancellationToken) -> Outcome<()> {
        loop {
            tokio::select! {
                _ = token.cancelled() => {
                    self.drain_remaining().await;
                    return Ok(());
                }
                maybe = self.rx.recv() => match maybe {
                    Some(first) => self.flush_batch(first).await,
                    None => return Ok(()),
                }
            }
        }
    }
}
