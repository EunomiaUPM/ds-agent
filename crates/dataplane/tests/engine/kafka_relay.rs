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

//! Unit and integration tests for the Kafka `PUSH` relay engine (`KafkaLeaseManager`,
//! `KafkaRelaySupervisor`, `KafkaRelayWorker`, `KafkaRecoveryWorker`, `KafkaProviderPushConfigurator`,
//! `KafkaPubSubscriber`, and `DataplaneDriverFactory`).

use std::collections::VecDeque;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::Router;
use common::test_utils::config_fixtures::transfer_config_fixture;
use common::test_utils::scopes::TestUsers;
use connector::{
    AuthenticationConfig, ConnectorInstanceDto, ConnectorMetadata, InteractionConfig,
    KafkaOffsetReset, KafkaSpec, MockConnectorInstanceFacadeTrait, ProtocolSpec, PullLifecycle,
    PushLifecycle, TemplateVecString, TransferLifetime,
};
use dataplane::data::factory_trait::MockDataplaneRepoTrait;
use dataplane::data::repo::dataplane_transfer::MockDataplaneTransfersRepo;
use dataplane::data::sea_orm::orm::dataplane_transfers;
use dataplane::engine::dataplane_manager::dataplane_commands::{
    DataplaneInitCommandDirection, DataplaneInitCommandTypes,
};
use dataplane::engine::dataplane_manager::dataplane_context::DataplaneContext;
use dataplane::engine::dataplane_manager::dataplane_driver_factory::{
    DataplaneDriverFactory, DataplaneDriverFactoryTrait,
};
use dataplane::engine::dataplane_manager::dataplane_proxy::{
    DataplaneProxyEgress, DataplaneProxyIngress,
};
use dataplane::engine::dataplane_manager::dataplane_runtime::{
    DataplaneRuntime, DeliveryPolicy,
};
use dataplane::entities::dataplane_transfers::{
    DataplaneTransferDto, InteractionMode, TransferRole, TransferState,
};
use dataplane::proxy::DeliveryTracker;
use dataplane::relay::{
    KafkaConsumerConfig, KafkaConsumerFactory, KafkaConsumerSession, KafkaLeaseManager,
    KafkaRecord, KafkaRecoveryWorker, KafkaRelaySupervisor, RdkafkaConsumerFactory,
};
use dataplane::services::dataplane_transfers::MockDataplaneTransferServiceTrait;
use dataplane::{DataplaneAddress, DataplaneSignal, DataplaneSignalSender};
use tokio::net::TcpListener;
use tokio::sync::Notify;
use urn::Urn;
use ymir::errors::Outcome;

const DP_URN: &str = "urn:dataplane-transfer:kafka-1";
const TP_URN: &str = "urn:transfer-process:kafka-1";
const CONNECTOR_URN: &str = "urn:connector-instance:kafka-1";

#[derive(Clone, Debug)]
struct CapturedDelivery {
    headers: HeaderMap,
    body: Vec<u8>,
}

#[derive(Clone)]
struct SinkState {
    deliveries: Arc<Mutex<Vec<CapturedDelivery>>>,
    status: StatusCode,
    notify: Arc<Notify>,
}

async fn spawn_http_sink(status: StatusCode) -> (String, SinkState) {
    let state = SinkState {
        deliveries: Arc::new(Mutex::new(Vec::new())),
        status,
        notify: Arc::new(Notify::new()),
    };
    let state_for_route = state.clone();
    let app = Router::new()
        .route(
            "/sink",
            post(
                |State(st): State<SinkState>, headers: HeaderMap, body: Bytes| async move {
                    st.deliveries.lock().unwrap().push(CapturedDelivery {
                        headers,
                        body: body.to_vec(),
                    });
                    st.notify.notify_waiters();
                    st.status
                },
            ),
        )
        .with_state(state_for_route);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    (format!("http://{addr}/sink"), state)
}

/// Deterministic in-memory Kafka consumer factory for fast unit tests.
#[derive(Clone)]
struct MockKafkaBroker {
    queue: Arc<Mutex<VecDeque<KafkaRecord>>>,
    committed: Arc<Mutex<Vec<(String, i32, i64)>>>,
    item_ready: Arc<Notify>,
}

impl MockKafkaBroker {
    fn new(initial: Vec<KafkaRecord>) -> Self {
        Self {
            queue: Arc::new(Mutex::new(initial.into())),
            committed: Arc::new(Mutex::new(Vec::new())),
            item_ready: Arc::new(Notify::new()),
        }
    }

    fn publish(&self, record: KafkaRecord) {
        self.queue.lock().unwrap().push_back(record);
        self.item_ready.notify_waiters();
    }

    fn committed_offsets(&self) -> Vec<(String, i32, i64)> {
        self.committed.lock().unwrap().clone()
    }
}

struct MockKafkaSession {
    broker: MockKafkaBroker,
}

#[async_trait::async_trait]
impl KafkaConsumerSession for MockKafkaSession {
    async fn recv(&mut self) -> Result<KafkaRecord, String> {
        loop {
            let notified = self.broker.item_ready.notified();
            if let Some(rec) = self.broker.queue.lock().unwrap().pop_front() {
                return Ok(rec);
            }
            notified.await;
        }
    }

    async fn commit(&mut self, topic: &str, partition: i32, offset: i64) -> Result<(), String> {
        self.broker
            .committed
            .lock()
            .unwrap()
            .push((topic.to_string(), partition, offset));
        Ok(())
    }
}

#[async_trait::async_trait]
impl KafkaConsumerFactory for MockKafkaBroker {
    async fn preflight(&self, _cfg: &KafkaConsumerConfig) -> Outcome<()> {
        Ok(())
    }

    async fn create_session(
        &self,
        _cfg: &KafkaConsumerConfig,
    ) -> Outcome<Box<dyn KafkaConsumerSession>> {
        Ok(Box::new(MockKafkaSession {
            broker: self.clone(),
        }))
    }
}

fn kafka_connector_fixture(lifetime: TransferLifetime, max_retries: Option<u32>) -> ConnectorInstanceDto {
    ConnectorInstanceDto {
        id: Urn::from_str(CONNECTOR_URN).unwrap(),
        user_id: "tenant-1".to_string(),
        metadata: ConnectorMetadata {
            name: Some("kafka-telemetry".into()),
            author: Some("UPM".into()),
            description: None,
            version: Some("1.0.0".into()),
            created_at: None,
        },
        authentication_config: AuthenticationConfig::NoAuth,
        interaction: InteractionConfig::Push(PushLifecycle {
            lifetime,
            max_retries,
            subscribe: ProtocolSpec::Kafka(KafkaSpec {
                brokers: TemplateVecString::Value(vec!["127.0.0.1:9092".to_string()]),
                topic: "telemetry.v1".to_string(),
                group_id: Some("ds-test-group".to_string()),
                auto_offset_reset: KafkaOffsetReset::Earliest,
                ..Default::default()
            }),
            unsubscribe: None,
        }),
        distribution_id: Urn::from_str("urn:distribution:kafka-1").unwrap(),
    }
}

fn dataplane_model_fixture(sink_url: &str, policy: DeliveryPolicy) -> dataplane_transfers::Model {
    let egress = DataplaneProxyEgress::HttpProxy {
        path: sink_url.to_string(),
        token_type: Some("Bearer".to_string()),
        token: Some("sink-cap-token".to_string()),
    };
    let runtime = DataplaneRuntime {
        delivery_policy: policy,
        ..Default::default()
    };
    dataplane_transfers::Model {
        user_id: "tenant-1".to_string(),
        user_role: ymir::types::oauth::RolePath::root(),
        visibility: ymir::types::participants::Visibility::Private,
        id: DP_URN.to_string(),
        transfer_process_id: TP_URN.to_string(),
        role: TransferRole::Provider,
        interaction_mode: InteractionMode::Push,
        state: TransferState::Started,
        connector_instance_id: Some(CONNECTOR_URN.to_string()),
        ingress_config: serde_json::to_value(DataplaneProxyIngress::NoOp).unwrap(),
        egress_config: serde_json::to_value(egress).unwrap(),
        flow_control: Some(serde_json::to_value(runtime).unwrap()),
        created_at: chrono::Utc::now().into(),
        updated_at: None,
    }
}

async fn build_provider_context(
    connector: ConnectorInstanceDto,
    sink_url: &str,
    policy: DeliveryPolicy,
) -> DataplaneContext {
    let model = dataplane_model_fixture(sink_url, policy);
    let mut mock_svc = MockDataplaneTransferServiceTrait::new();
    mock_svc.expect_create().returning(move |_, _| {
        Ok(DataplaneTransferDto {
            inner: model.clone(),
            fields: Default::default(),
            logs: vec![],
        })
    });

    let mut ctx = DataplaneContext::from_init(
        Arc::new(mock_svc),
        Arc::new(MockConnectorInstanceFacadeTrait::new()),
        transfer_config_fixture(),
        DataplaneInitCommandTypes::AsProvider {
            owner: TestUsers::owner("tenant-1"),
            transfer_process_id: Urn::from_str(TP_URN).unwrap(),
            connector_instance: connector,
            direction: DataplaneInitCommandDirection::Push {
                data_address: Some(DataplaneAddress {
                    endpoint_type: "HTTP".to_string(),
                    endpoint: sink_url.to_string(),
                    authorization_type: Some("Bearer".to_string()),
                    authorization: Some("sink-cap-token".to_string()),
                }),
            },
        },
    )
    .await
    .unwrap();
    ctx.set_runtime(DataplaneRuntime {
        delivery_policy: policy,
        ..Default::default()
    });
    ctx
}

#[tokio::test]
async fn driver_factory_resolves_kafka_provider_push_and_rejects_kafka_pull() {
    let push_connector = kafka_connector_fixture(TransferLifetime::Infinite, None);
    let ctx = build_provider_context(
        push_connector,
        "http://127.0.0.1:9999/sink",
        DeliveryPolicy::default(),
    )
    .await;

    let factory = DataplaneDriverFactory::new();
    let driver = factory
        .get_or_create_driver(&ctx)
        .expect("Kafka Provider Push driver must resolve");
    assert!(driver.subscriber.is_some());

    let configured_ctx = driver
        .proxy_configurator
        .configure_proxy(&ctx)
        .await
        .expect("proxy configuration succeeds");
    let proxy = configured_ctx.proxy().expect("proxy set");
    assert!(matches!(proxy.ingress(), DataplaneProxyIngress::NoOp));
    assert!(matches!(
        proxy.egress(),
        DataplaneProxyEgress::HttpProxy { path, token, .. }
        if path == "http://127.0.0.1:9999/sink" && token.as_deref() == Some("sink-cap-token")
    ));

    // Kafka in PULL mode must be rejected by the driver factory as well.
    let mut pull_connector = kafka_connector_fixture(TransferLifetime::Infinite, None);
    pull_connector.interaction = InteractionConfig::Pull(PullLifecycle {
        lifetime: TransferLifetime::Infinite,
        max_retries: None,
        data_access: ProtocolSpec::Kafka(KafkaSpec {
            brokers: TemplateVecString::Value(vec!["127.0.0.1:9092".to_string()]),
            topic: "telemetry.v1".to_string(),
            ..Default::default()
        }),
    });
    let mut mock_svc = MockDataplaneTransferServiceTrait::new();
    mock_svc.expect_create().returning(|_, _| {
        Ok(DataplaneTransferDto {
            inner: dataplane_transfers::Model {
                user_id: "tenant-1".to_string(),
                user_role: ymir::types::oauth::RolePath::root(),
                visibility: ymir::types::participants::Visibility::Private,
                id: DP_URN.to_string(),
                transfer_process_id: TP_URN.to_string(),
                role: TransferRole::Provider,
                interaction_mode: InteractionMode::Pull,
                state: TransferState::Configuring,
                connector_instance_id: Some(CONNECTOR_URN.to_string()),
                ingress_config: serde_json::json!("NoOp"),
                egress_config: serde_json::json!("NoOp"),
                flow_control: None,
                created_at: chrono::Utc::now().into(),
                updated_at: None,
            },
            fields: Default::default(),
            logs: vec![],
        })
    });
    let pull_ctx = DataplaneContext::from_init(
        Arc::new(mock_svc),
        Arc::new(MockConnectorInstanceFacadeTrait::new()),
        transfer_config_fixture(),
        DataplaneInitCommandTypes::AsProvider {
            owner: TestUsers::owner("tenant-1"),
            transfer_process_id: Urn::from_str(TP_URN).unwrap(),
            connector_instance: pull_connector,
            direction: DataplaneInitCommandDirection::Pull { data_address: None },
        },
    )
    .await
    .unwrap();
    assert!(factory.get_or_create_driver(&pull_ctx).is_err());
}

#[tokio::test]
async fn kafka_lease_manager_exclusivity_renewal_and_expiry_failover() {
    let lease_a = KafkaLeaseManager::in_memory()
        .with_replica_id("replica-a")
        .with_ttl_ms(80);

    assert!(lease_a.acquire(DP_URN).await);
    // Idempotent re-acquire by the same replica succeeds.
    assert!(lease_a.acquire(DP_URN).await);
    assert!(lease_a.renew(DP_URN).await);

    // Explicit release makes the lease immediately available.
    lease_a.release(DP_URN).await;
    assert!(!lease_a.renew(DP_URN).await);
    assert!(lease_a.acquire(DP_URN).await);

    // Expiry allows renewal to fail once TTL elapses without heartbeats.
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(!lease_a.renew(DP_URN).await);
}

#[tokio::test]
async fn kafka_relay_streams_records_with_headers_and_commits_offsets_through_suspend_resume() {
    let (sink_url, sink) = spawn_http_sink(StatusCode::OK).await;
    let record_1 = KafkaRecord {
        topic: "telemetry.v1".to_string(),
        partition: 2,
        offset: 41,
        timestamp_ms: Some(1_762_000_123_000),
        key: Some(b"sensor-7".to_vec()),
        payload: br#"{"temp":22.4}"#.to_vec(),
    };
    let broker = MockKafkaBroker::new(vec![record_1]);

    let supervisor = Arc::new(
        KafkaRelaySupervisor::new(Arc::new(KafkaLeaseManager::in_memory()), Arc::new(broker.clone()))
            .with_allow_private_egress(true)
            .with_backoff_ms(vec![10]),
    );
    let factory = DataplaneDriverFactory::new().with_kafka_supervisor(supervisor.clone());

    let connector = kafka_connector_fixture(TransferLifetime::Infinite, Some(2));
    let ctx = build_provider_context(connector, &sink_url, DeliveryPolicy::default()).await;
    let driver = factory.get_or_create_driver(&ctx).unwrap();
    let ctx = driver.proxy_configurator.configure_proxy(&ctx).await.unwrap();
    let subscriber = driver.subscriber.as_ref().unwrap();

    // Start (subscribe) -> record_1 relayed to HTTP sink.
    let started_ctx = subscriber.subscribe(&ctx).await.expect("subscribe succeeds");
    assert_eq!(
        started_ctx.runtime().unwrap().subscription["protocol"],
        "KAFKA"
    );

    tokio::time::timeout(Duration::from_secs(2), async {
        while sink.deliveries.lock().unwrap().is_empty() {
            sink.notify.notified().await;
        }
    })
    .await
    .expect("first record delivered");

    {
        let deliveries = sink.deliveries.lock().unwrap();
        assert_eq!(deliveries.len(), 1);
        let d = &deliveries[0];
        assert_eq!(d.body, br#"{"temp":22.4}"#);
        assert_eq!(
            d.headers.get("authorization").unwrap(),
            "Bearer sink-cap-token"
        );
        assert_eq!(d.headers.get("content-type").unwrap(), "application/json");
        assert_eq!(d.headers.get("x-kafka-topic").unwrap(), "telemetry.v1");
        assert_eq!(d.headers.get("x-kafka-partition").unwrap(), "2");
        assert_eq!(d.headers.get("x-kafka-offset").unwrap(), "41");
        assert_eq!(
            d.headers.get("x-kafka-timestamp").unwrap(),
            "1762000123000"
        );
        assert_eq!(d.headers.get("x-kafka-key").unwrap(), "sensor-7");
    }

    // Wait briefly for commit after 2xx.
    tokio::time::sleep(Duration::from_millis(30)).await;
    assert_eq!(
        broker.committed_offsets(),
        vec![("telemetry.v1".to_string(), 2, 41)]
    );

    // Suspend (unsubscribe) -> worker stops cleanly.
    let stopped_ctx = subscriber
        .unsubscribe(&started_ctx)
        .await
        .expect("unsubscribe succeeds");
    assert_eq!(
        stopped_ctx.runtime().unwrap().unsubscription["stopped"],
        true
    );
    assert!(!supervisor.is_running(DP_URN));

    // Publish while suspended -> not delivered until resumed!
    broker.publish(KafkaRecord {
        topic: "telemetry.v1".to_string(),
        partition: 2,
        offset: 42,
        timestamp_ms: None,
        key: Some(vec![0xff, 0x00]),
        payload: b"raw-bytes".to_vec(),
    });
    tokio::time::sleep(Duration::from_millis(40)).await;
    assert_eq!(sink.deliveries.lock().unwrap().len(), 1);

    // Resume (subscribe) -> delivers offset 42 with binary base64 key + octet-stream!
    let resumed_ctx = subscriber
        .subscribe(&stopped_ctx)
        .await
        .expect("resume subscribe succeeds");
    tokio::time::timeout(Duration::from_secs(2), async {
        while sink.deliveries.lock().unwrap().len() < 2 {
            sink.notify.notified().await;
        }
    })
    .await
    .expect("second record delivered on resume");

    {
        let deliveries = sink.deliveries.lock().unwrap();
        let d2 = &deliveries[1];
        assert_eq!(d2.body, b"raw-bytes");
        assert_eq!(
            d2.headers.get("content-type").unwrap(),
            "application/octet-stream"
        );
        assert_eq!(d2.headers.get("x-kafka-key").unwrap(), "base64:/wA=");
    }

    // Terminate (unsubscribe).
    let _ = subscriber.unsubscribe(&resumed_ctx).await.unwrap();
    assert!(!supervisor.is_running(DP_URN));
}

#[tokio::test]
async fn kafka_relay_finite_lifetime_completes_after_single_record() {
    let (sink_url, sink) = spawn_http_sink(StatusCode::ACCEPTED).await;
    let broker = MockKafkaBroker::new(vec![
        KafkaRecord {
            topic: "telemetry.v1".to_string(),
            partition: 0,
            offset: 0,
            timestamp_ms: None,
            key: None,
            payload: br#"{"seq":1}"#.to_vec(),
        },
        KafkaRecord {
            topic: "telemetry.v1".to_string(),
            partition: 0,
            offset: 1,
            timestamp_ms: None,
            key: None,
            payload: br#"{"seq":2}"#.to_vec(),
        },
    ]);

    let (sig_tx, mut sig_rx) = DataplaneSignalSender::open(8);
    let supervisor = Arc::new(
        KafkaRelaySupervisor::new(Arc::new(KafkaLeaseManager::in_memory()), Arc::new(broker.clone()))
            .with_signals(sig_tx)
            .with_tracker(Arc::new(DeliveryTracker::in_memory()))
            .with_allow_private_egress(true),
    );
    let factory = DataplaneDriverFactory::new().with_kafka_supervisor(supervisor);

    let policy = DeliveryPolicy {
        lifetime: TransferLifetime::Finite,
        max_retries: 2,
    };
    let connector = kafka_connector_fixture(TransferLifetime::Finite, Some(2));
    let ctx = build_provider_context(connector, &sink_url, policy).await;
    let driver = factory.get_or_create_driver(&ctx).unwrap();
    let ctx = driver.proxy_configurator.configure_proxy(&ctx).await.unwrap();

    driver
        .subscriber
        .as_ref()
        .unwrap()
        .subscribe(&ctx)
        .await
        .unwrap();

    let sig = tokio::time::timeout(Duration::from_secs(2), sig_rx.recv())
        .await
        .expect("Completed signal received")
        .unwrap();
    assert_eq!(
        sig,
        DataplaneSignal::Completed {
            transfer_process_id: Urn::from_str(TP_URN).unwrap(),
        }
    );

    // Exactly 1 message was delivered and committed; the second remains unconsumed.
    tokio::time::sleep(Duration::from_millis(40)).await;
    assert_eq!(sink.deliveries.lock().unwrap().len(), 1);
    assert_eq!(
        broker.committed_offsets(),
        vec![("telemetry.v1".to_string(), 0, 0)]
    );
}

#[tokio::test]
async fn kafka_relay_transient_sink_failures_retry_and_emit_failed_when_max_retries_exceeded() {
    let (sink_url, sink) = spawn_http_sink(StatusCode::SERVICE_UNAVAILABLE).await;
    let broker = MockKafkaBroker::new(vec![KafkaRecord {
        topic: "telemetry.v1".to_string(),
        partition: 0,
        offset: 7,
        timestamp_ms: None,
        key: None,
        payload: br#"{"alert":true}"#.to_vec(),
    }]);

    let (sig_tx, mut sig_rx) = DataplaneSignalSender::open(8);
    let supervisor = Arc::new(
        KafkaRelaySupervisor::new(Arc::new(KafkaLeaseManager::in_memory()), Arc::new(broker.clone()))
            .with_signals(sig_tx)
            .with_tracker(Arc::new(DeliveryTracker::in_memory()))
            .with_allow_private_egress(true)
            .with_backoff_ms(vec![10, 10, 10]),
    );
    let factory = DataplaneDriverFactory::new().with_kafka_supervisor(supervisor);

    let policy = DeliveryPolicy {
        lifetime: TransferLifetime::Infinite,
        max_retries: 2,
    };
    let connector = kafka_connector_fixture(TransferLifetime::Infinite, Some(2));
    let ctx = build_provider_context(connector, &sink_url, policy).await;
    let driver = factory.get_or_create_driver(&ctx).unwrap();
    let ctx = driver.proxy_configurator.configure_proxy(&ctx).await.unwrap();

    driver
        .subscriber
        .as_ref()
        .unwrap()
        .subscribe(&ctx)
        .await
        .unwrap();

    let sig = tokio::time::timeout(Duration::from_secs(2), sig_rx.recv())
        .await
        .expect("Failed signal received")
        .unwrap();
    match sig {
        DataplaneSignal::Failed {
            transfer_process_id,
            code,
            ..
        } => {
            assert_eq!(transfer_process_id, Urn::from_str(TP_URN).unwrap());
            assert_eq!(code, "DATAPLANE_MAX_RETRIES_EXCEEDED");
        }
        other => panic!("expected Failed signal, got {other:?}"),
    }

    // 1 initial attempt + 2 retries = 3 HTTP attempts; 0 offsets committed!
    assert_eq!(sink.deliveries.lock().unwrap().len(), 3);
    assert!(broker.committed_offsets().is_empty());
}

#[tokio::test]
async fn kafka_relay_permanent_sink_rejection_emits_failed_immediately() {
    let (sink_url, sink) = spawn_http_sink(StatusCode::UNAUTHORIZED).await;
    let broker = MockKafkaBroker::new(vec![KafkaRecord {
        topic: "telemetry.v1".to_string(),
        partition: 0,
        offset: 0,
        timestamp_ms: None,
        key: None,
        payload: b"payload".to_vec(),
    }]);

    let (sig_tx, mut sig_rx) = DataplaneSignalSender::open(8);
    let supervisor = Arc::new(
        KafkaRelaySupervisor::new(Arc::new(KafkaLeaseManager::in_memory()), Arc::new(broker.clone()))
            .with_signals(sig_tx)
            .with_allow_private_egress(true)
            .with_backoff_ms(vec![10]),
    );
    let factory = DataplaneDriverFactory::new().with_kafka_supervisor(supervisor);

    let policy = DeliveryPolicy {
        lifetime: TransferLifetime::Infinite,
        max_retries: 3,
    };
    let connector = kafka_connector_fixture(TransferLifetime::Infinite, Some(3));
    let ctx = build_provider_context(connector, &sink_url, policy).await;
    let driver = factory.get_or_create_driver(&ctx).unwrap();
    let ctx = driver.proxy_configurator.configure_proxy(&ctx).await.unwrap();
    driver
        .subscriber
        .as_ref()
        .unwrap()
        .subscribe(&ctx)
        .await
        .unwrap();

    let sig = tokio::time::timeout(Duration::from_secs(2), sig_rx.recv())
        .await
        .expect("Failed signal received")
        .unwrap();
    assert!(matches!(
        sig,
        DataplaneSignal::Failed { ref code, .. } if code == "DATAPLANE_UPSTREAM_REJECTED"
    ));
    assert_eq!(sink.deliveries.lock().unwrap().len(), 1);
    assert!(broker.committed_offsets().is_empty());
}

#[tokio::test]
async fn kafka_relay_blocks_ssrf_when_private_egress_disabled() {
    let (sink_url, sink) = spawn_http_sink(StatusCode::OK).await;
    let broker = MockKafkaBroker::new(vec![KafkaRecord {
        topic: "telemetry.v1".to_string(),
        partition: 0,
        offset: 0,
        timestamp_ms: None,
        key: None,
        payload: b"secret".to_vec(),
    }]);

    let (sig_tx, mut sig_rx) = DataplaneSignalSender::open(8);
    let supervisor = Arc::new(
        KafkaRelaySupervisor::new(Arc::new(KafkaLeaseManager::in_memory()), Arc::new(broker))
            .with_signals(sig_tx)
            .with_allow_private_egress(false),
    );
    let factory = DataplaneDriverFactory::new().with_kafka_supervisor(supervisor);

    let connector = kafka_connector_fixture(TransferLifetime::Infinite, None);
    let ctx = build_provider_context(connector, &sink_url, DeliveryPolicy::default()).await;
    let driver = factory.get_or_create_driver(&ctx).unwrap();
    let ctx = driver.proxy_configurator.configure_proxy(&ctx).await.unwrap();
    driver
        .subscriber
        .as_ref()
        .unwrap()
        .subscribe(&ctx)
        .await
        .unwrap();

    let sig = tokio::time::timeout(Duration::from_secs(2), sig_rx.recv())
        .await
        .expect("SSRF Failed signal received")
        .unwrap();
    assert!(matches!(
        sig,
        DataplaneSignal::Failed { ref code, .. } if code == "DATAPLANE_FORBIDDEN_TARGET"
    ));
    assert!(sink.deliveries.lock().unwrap().is_empty());
}

#[tokio::test]
async fn kafka_recovery_worker_reconciles_started_kafka_push_transfers() {
    let (sink_url, sink) = spawn_http_sink(StatusCode::OK).await;
    let broker = MockKafkaBroker::new(vec![KafkaRecord {
        topic: "telemetry.v1".to_string(),
        partition: 1,
        offset: 99,
        timestamp_ms: None,
        key: None,
        payload: br#"{"recovered":true}"#.to_vec(),
    }]);

    let model = dataplane_model_fixture(&sink_url, DeliveryPolicy::default());
    let mut transfers_repo = MockDataplaneTransfersRepo::new();
    transfers_repo
        .expect_get_all_dataplane_transfers()
        .returning(move |_, _, _, _| Ok(vec![model.clone()]));

    let transfers_repo_arc = Arc::new(transfers_repo);
    let mut repo_factory = MockDataplaneRepoTrait::new();
    repo_factory
        .expect_get_dataplane_transfers_repo()
        .returning(move || transfers_repo_arc.clone());

    let connector = kafka_connector_fixture(TransferLifetime::Infinite, None);
    let mut connector_facade = MockConnectorInstanceFacadeTrait::new();
    connector_facade
        .expect_get_instance_by_id()
        .returning(move |_| Ok(Some(connector.clone())));

    let supervisor = Arc::new(
        KafkaRelaySupervisor::new(Arc::new(KafkaLeaseManager::in_memory()), Arc::new(broker))
            .with_allow_private_egress(true),
    );
    let recovery = KafkaRecoveryWorker::new(
        Arc::new(repo_factory),
        Arc::new(connector_facade),
        None,
        supervisor.clone(),
    );

    recovery.sweep_once().await;

    tokio::time::timeout(Duration::from_secs(2), async {
        while sink.deliveries.lock().unwrap().is_empty() {
            sink.notify.notified().await;
        }
    })
    .await
    .expect("recovered worker delivered record");

    assert_eq!(
        sink.deliveries.lock().unwrap()[0].body,
        br#"{"recovered":true}"#
    );
    supervisor.shutdown_all().await;
}

/// Real broker integration test (`Redpanda` / `Apache Kafka`).
/// Run with: `KAFKA_BROKERS=localhost:19092 cargo test -p dataplane --test engine redpanda_live -- --ignored`
#[tokio::test]
#[ignore]
async fn redpanda_live_kafka_push_end_to_end() {
    use rdkafka::config::ClientConfig;
    use rdkafka::producer::{FutureProducer, FutureRecord};

    let brokers = std::env::var("KAFKA_BROKERS").unwrap_or_else(|_| "127.0.0.1:19092".to_string());
    let topic = format!("eunomia-it-{}", uuid::Uuid::new_v4().simple());
    let (sink_url, sink) = spawn_http_sink(StatusCode::OK).await;

    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", &brokers)
        .create()
        .expect("build Kafka producer");
    producer
        .send(
            FutureRecord::to(&topic)
                .key("k-1")
                .payload(r#"{"live":true}"#),
            Duration::from_secs(5),
        )
        .await
        .expect("produce initial message");

    let supervisor = Arc::new(
        KafkaRelaySupervisor::new(
            Arc::new(KafkaLeaseManager::in_memory()),
            Arc::new(RdkafkaConsumerFactory::new()),
        )
        .with_allow_private_egress(true),
    );
    let factory = DataplaneDriverFactory::new().with_kafka_supervisor(supervisor.clone());

    let mut connector = kafka_connector_fixture(TransferLifetime::Infinite, Some(2));
    connector.interaction = InteractionConfig::Push(PushLifecycle {
        lifetime: TransferLifetime::Infinite,
        max_retries: Some(2),
        subscribe: ProtocolSpec::Kafka(KafkaSpec {
            brokers: TemplateVecString::Value(vec![brokers]),
            topic: topic.clone(),
            auto_offset_reset: KafkaOffsetReset::Earliest,
            ..Default::default()
        }),
        unsubscribe: None,
    });

    let ctx = build_provider_context(connector, &sink_url, DeliveryPolicy::default()).await;
    let driver = factory.get_or_create_driver(&ctx).unwrap();
    let ctx = driver.proxy_configurator.configure_proxy(&ctx).await.unwrap();
    let started = driver
        .subscriber
        .as_ref()
        .unwrap()
        .subscribe(&ctx)
        .await
        .expect("live subscribe succeeds");

    tokio::time::timeout(Duration::from_secs(10), async {
        while sink.deliveries.lock().unwrap().is_empty() {
            sink.notify.notified().await;
        }
    })
    .await
    .expect("live Redpanda message relayed to HTTP sink");

    assert_eq!(sink.deliveries.lock().unwrap()[0].body, br#"{"live":true}"#);
    driver
        .subscriber
        .as_ref()
        .unwrap()
        .unsubscribe(&started)
        .await
        .unwrap();
}
