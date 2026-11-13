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

//! Asynchronous batching audit worker for proxy transfer events.

use std::sync::Arc;
use std::time::Duration;

use common::test_utils::scopes::TestUsers;
use dataplane::data::factory_trait::MockDataplaneRepoTrait;
use dataplane::data::repo::transfer_event::{MockTransferEventRepo, TransferEventRepo};
use dataplane::data::sea_orm::orm::dataplane_transfers;
use dataplane::data::sea_orm::orm::transfer_event::{LogLevel, NewTransferEvent};
use dataplane::entities::dataplane_transfers::{
    DataplaneTransferDto, InteractionMode, TransferRole, TransferState,
};
use dataplane::proxy::audit::ProxyAuditWorker;
use dataplane::setup::DataplaneEvents;
use events::schemas::dataspace::PolicyDecisionOutcome;
use events::{EventSinkTrait, MockEventSinkTrait};
use serde_json::json;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;
use ymir::boot::workers::BackgroundWorker;

fn sample_event(seq: usize) -> NewTransferEvent {
    NewTransferEvent {
        owner: TestUsers::owner("tenant-1"),
        transfer_id: "urn:dataplane-transfer:audit-1".to_string(),
        level: LogLevel::Info,
        component: "HttpDataProxy".to_string(),
        message: format!("packet-{seq}"),
        data: None,
    }
}

#[tokio::test]
async fn proxy_audit_worker_batches_queued_events() {
    let flushed = Arc::new(Notify::new());
    let signal = flushed.clone();

    let mut events_repo = MockTransferEventRepo::new();
    events_repo
        .expect_create_transfer_events_batch()
        .withf(|batch| batch.len() == 3 && batch[0].message == "packet-1")
        .times(1)
        .returning(move |_| {
            signal.notify_one();
            Ok(())
        });

    let events_arc: Arc<dyn TransferEventRepo> = Arc::new(events_repo);
    let mut factory = MockDataplaneRepoTrait::new();
    factory
        .expect_get_transfer_events_repo()
        .return_const(events_arc);

    let (sender, worker) = ProxyAuditWorker::open(Arc::new(factory), 16);
    for i in 1..=3 {
        sender.try_send(sample_event(i));
    }

    let cancel = CancellationToken::new();
    let handle = tokio::spawn({
        let cancel = cancel.clone();
        async move { Box::new(worker).run(cancel).await }
    });

    tokio::time::timeout(Duration::from_secs(2), flushed.notified())
        .await
        .expect("proxy audit worker timed out");
    cancel.cancel();
    handle.await.unwrap().unwrap();
}

#[tokio::test]
async fn proxy_audit_worker_flushes_mixed_events_and_envelopes_in_batches() {
    let logs_flushed = Arc::new(Notify::new());
    let env_flushed = Arc::new(Notify::new());
    let logs_signal = logs_flushed.clone();
    let env_signal = env_flushed.clone();

    let mut events_repo = MockTransferEventRepo::new();
    events_repo
        .expect_create_transfer_events_batch()
        .withf(|batch| batch.len() == 2)
        .times(1)
        .returning(move |_| {
            logs_signal.notify_one();
            Ok(())
        });
    let events_arc: Arc<dyn TransferEventRepo> = Arc::new(events_repo);
    let mut factory = MockDataplaneRepoTrait::new();
    factory
        .expect_get_transfer_events_repo()
        .return_const(events_arc);

    let mut sink = MockEventSinkTrait::new();
    sink.expect_emit_envelopes()
        .withf(|batch| batch.len() == 2)
        .times(1)
        .returning(move |_| {
            env_signal.notify_one();
            Ok(())
        });
    let sink_arc: Arc<dyn EventSinkTrait> = Arc::new(sink);

    let dto = DataplaneTransferDto {
        inner: dataplane_transfers::Model {
            user_id: "tenant-1".to_string(),
            user_role: ymir::types::oauth::RolePath::root(),
            visibility: ymir::types::participants::Visibility::Private,
            id: "urn:dataplane-transfer:audit-1".to_string(),
            transfer_process_id: "urn:transfer-process:audit-1".to_string(),
            role: TransferRole::Provider,
            interaction_mode: InteractionMode::Pull,
            state: TransferState::Started,
            connector_instance_id: None,
            ingress_config: json!({}),
            egress_config: json!({}),
            flow_control: None,
            created_at: chrono::Utc::now().into(),
            updated_at: None,
        },
        fields: Default::default(),
        logs: vec![],
    };

    let (sender, worker) = ProxyAuditWorker::open(Arc::new(factory), 16);
    let worker = worker.with_events(sink_arc);
    sender.try_send(sample_event(1));
    sender.try_send_envelope(
        DataplaneEvents::build_policy_decision_envelope(
            &dto,
            PolicyDecisionOutcome::Permit,
            "data_transfer",
            Some("ingress_capability"),
            &[],
            None,
        )
        .unwrap(),
    );
    sender.try_send(sample_event(2));
    sender.try_send_envelope(
        DataplaneEvents::build_http_data_access_envelope(
            &dto,
            "GET",
            None,
            Some(200),
            Some(1),
            true,
            None,
        )
        .unwrap(),
    );

    let cancel = CancellationToken::new();
    let handle = tokio::spawn({
        let cancel = cancel.clone();
        async move { Box::new(worker).run(cancel).await }
    });

    tokio::time::timeout(Duration::from_secs(2), async {
        tokio::join!(logs_flushed.notified(), env_flushed.notified());
    })
    .await
    .expect("mixed audit worker timed out");
    cancel.cancel();
    handle.await.unwrap().unwrap();
}
