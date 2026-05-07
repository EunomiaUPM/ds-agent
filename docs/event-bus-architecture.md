# Event Bus Architecture — ds-protocol

## Estado actual del crate `events`

El crate ya tiene:
- `Subscription` con flags booleanos por topic (`transfer_process`, `catalog`, etc.)
- `Notification` persistida en DB con `category`, `subcategory`, `message_type`, `message_operation`, `status`
- `broadcast_notification`: hace HTTP POST síncrono a todos los callbacks registrados
- Sin Kafka, sin `IntoEvent`, sin bus interno asíncrono, sin wildcards de topic

La arquitectura propuesta extiende este crate sin romper la API HTTP existente.

---

## Diseño conceptual

```
┌──────────────────────────────────────────────────────────────────┐
│                        Domain Crates                             │
│   transfer-agent · negotiation-agent · dataplane · connector     │
│                                                                  │
│   define: DomainEvent + impl IntoEvent<DomainEvent> for MyType   │
└────────────────────────────┬─────────────────────────────────────┘
                             │  EventBus::publish(envelope)
                             ▼
┌──────────────────────────────────────────────────────────────────┐
│                     EventBus (events crate)                      │
│                                                                  │
│  ┌────────────────┐   ┌──────────────────┐   ┌───────────────┐  │
│  │  Outbox Writer │──▶│  Dispatch Worker │──▶│  Topic Router │  │
│  │  (DB first)    │   │  (tokio spawn)   │   │  (wildcards)  │  │
│  └────────────────┘   └──────────────────┘   └───────┬───────┘  │
│                                                       │          │
└───────────────────────────────────────────────────────┼──────────┘
                                                        │
                  ┌─────────────────┬──────────────────┤
                  ▼                 ▼                  ▼
         ┌──────────────┐  ┌──────────────┐  ┌────────────────┐
         │   DB Sink    │  │ Kafka Sink   │  │  HTTP Push     │
         │  (sea-orm)   │  │  (rdkafka)   │  │  + retry/DLQ   │
         │  event store │  │  optional    │  │  (tokio task)  │
         └──────────────┘  └──────────────┘  └────────────────┘
                  │                                    │
         ┌────────▼────────┐                 ┌─────────▼──────────┐
         │   BFF (interno) │                 │  Callbacks externos│
         │   channel sub   │                 │  (data-aggregator) │
         └─────────────────┘                 └────────────────────┘
```

---

## Tipos y traits clave

### `EventEnvelope` — sobre genérico inmutable

```rust
// events/src/bus/envelope.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub id: Urn,                          // unique event id
    pub topic: Topic,                     // "transfer-agent.transfer.started"
    pub source_crate: String,             // "transfer-agent"
    pub schema_version: u32,              // para evolución de schema
    pub timestamp: chrono::DateTime<Utc>,
    pub correlation_id: Option<Urn>,      // saga/trace id
    pub payload: serde_json::Value,
}
```

### `Topic` — naming jerárquico con wildcards

Formato: `{crate}.{entity}.{operation}`

```
transfer-agent.transfer.started
transfer-agent.transfer.completed
transfer-agent.transfer.failed
negotiation-agent.negotiation.initiated
negotiation-agent.negotiation.agreed
negotiation-agent.negotiation.rejected
dataplane.channel.opened
dataplane.channel.closed
dataplane.transfer.active
connector.connection.established
connector.connection.lost
catalog-agent.catalog.published
catalog-agent.asset.updated
```

Patrones de suscripción (con glob):
- `transfer-agent.*.*` - todos los eventos del crate transfer-agent
- `*.transfer.*` - todos los eventos de tipo "transfer" de cualquier crate
- `*.*.*` - todos los eventos

```rust
// events/src/bus/topic.rs
pub struct Topic(String);  // validated dot-separated path

pub struct TopicPattern(String);  // supports * wildcard per segment

impl TopicPattern {
    pub fn matches(&self, topic: &Topic) -> bool { /* glob matching */ }
}
```

### `IntoEvent` trait — lo implementa cada crate dominio

```rust
// events/src/bus/into_event.rs
pub trait IntoEvent {
    fn topic() -> Topic where Self: Sized;
    fn schema_version() -> u32 where Self: Sized { 1 }
    fn into_envelope(self) -> EventEnvelope;
}

// Macro helper para reducir boilerplate
#[macro_export]
macro_rules! impl_into_event {
    ($type:ty, $topic:expr, $version:expr) => {
        impl IntoEvent for $type {
            fn topic() -> Topic { Topic::new($topic) }
            fn schema_version() -> u32 { $version }
            fn into_envelope(self) -> EventEnvelope {
                EventEnvelope {
                    id: get_urn(None),
                    topic: Self::topic(),
                    source_crate: env!("CARGO_PKG_NAME").to_string(),
                    schema_version: Self::schema_version(),
                    timestamp: Utc::now(),
                    correlation_id: None,
                    payload: serde_json::to_value(self).unwrap(),
                }
            }
        }
    };
}
```

### Uso en un crate dominio

```rust
// transfer-agent/src/events.rs
#[derive(Serialize, Deserialize)]
pub struct TransferStartedEvent {
    pub transfer_id: Urn,
    pub consumer_id: String,
    pub asset_id: Urn,
    pub protocol: String,
}

impl_into_event!(TransferStartedEvent, "transfer-agent.transfer.started", 1);

// En el servicio:
let event = TransferStartedEvent { ... };
event_bus.publish(event.into_envelope()).await?;
```

### `EventBus` trait + implementación

```rust
// events/src/bus/mod.rs
#[async_trait]
pub trait EventBusTrait: Send + Sync + 'static {
    async fn publish(&self, envelope: EventEnvelope) -> Result<(), EventBusError>;

    // Suscripción interna (in-process, para BFF)
    fn subscribe(
        &self,
        pattern: TopicPattern,
    ) -> tokio::sync::broadcast::Receiver<EventEnvelope>;
}

pub struct EventBus {
    db: Arc<dyn EventBusRepo>,
    broadcaster: tokio::sync::broadcast::Sender<EventEnvelope>,
    kafka: Option<Arc<dyn KafkaSink>>,
    config: EventBusConfig,
}

impl EventBus {
    pub async fn publish(&self, mut envelope: EventEnvelope) -> Result<(), EventBusError> {
        // 1. Outbox: persistir primero (garantía at-least-once)
        self.db.insert_event(&envelope).await?;

        // 2. Broadcast interno (para suscriptores in-process como BFF)
        let _ = self.broadcaster.send(envelope.clone());  // ok si no hay receivers

        // 3. Kafka (si configurado) — fire and forget con retry en background
        if let Some(kafka) = &self.kafka {
            let kafka = kafka.clone();
            let env = envelope.clone();
            tokio::spawn(async move { kafka.send(env).await });
        }

        // 4. HTTP callbacks — dispatch en background con retry
        let db = self.db.clone();
        let config = self.config.clone();
        tokio::spawn(async move {
            dispatch_http_callbacks(db, envelope, config).await;
        });

        Ok(())
    }
}
```

---

## Cambios en DB

### Nueva tabla: `events` (event store)

```sql
CREATE TABLE events (
    id              TEXT PRIMARY KEY,         -- Urn
    topic           TEXT NOT NULL,
    source_crate    TEXT NOT NULL,
    schema_version  INT NOT NULL DEFAULT 1,
    timestamp       TIMESTAMP NOT NULL,
    correlation_id  TEXT,
    payload         JSONB NOT NULL,
    created_at      TIMESTAMP NOT NULL DEFAULT NOW()
);
CREATE INDEX events_topic_idx ON events(topic);
CREATE INDEX events_timestamp_idx ON events(timestamp DESC);
CREATE INDEX events_correlation_idx ON events(correlation_id) WHERE correlation_id IS NOT NULL;
```

### Modificar tabla: `subscriptions`

Reemplazar flags booleanos por `topic_pattern` (backward compat: migración convierte flags - pattern string):

```sql
ALTER TABLE subscriptions ADD COLUMN topic_pattern TEXT;
-- migración: transfer_process=true - topic_pattern = 'transfer-agent.*.*'
-- catalog=true - topic_pattern = 'catalog-agent.*.*'
-- etc.
ALTER TABLE subscriptions DROP COLUMN transfer_process;
-- ... etc
```

### Nueva tabla: `event_deliveries` (reemplaza `notifications`)

```sql
CREATE TABLE event_deliveries (
    id              TEXT PRIMARY KEY,
    event_id        TEXT NOT NULL REFERENCES events(id),
    subscription_id TEXT NOT NULL REFERENCES subscriptions(id),
    status          TEXT NOT NULL,           -- Pending | Delivered | Failed | DLQ
    attempts        INT NOT NULL DEFAULT 0,
    last_attempt_at TIMESTAMP,
    next_retry_at   TIMESTAMP,
    error_message   TEXT,
    delivered_at    TIMESTAMP,
    created_at      TIMESTAMP NOT NULL DEFAULT NOW()
);
CREATE INDEX deliveries_status_retry ON event_deliveries(status, next_retry_at)
    WHERE status IN ('Pending', 'Failed');
```

### Nueva tabla: `dead_letter_queue`

```sql
CREATE TABLE dead_letter_queue (
    id              TEXT PRIMARY KEY,
    event_id        TEXT NOT NULL,
    subscription_id TEXT NOT NULL,
    error_message   TEXT,
    payload         JSONB NOT NULL,
    failed_at       TIMESTAMP NOT NULL DEFAULT NOW()
);
```

---

## Workers asíncronos

### Dispatch Worker (HTTP callbacks con retry)

```rust
// events/src/bus/workers/http_dispatch.rs

// Política de retry: exponential backoff con jitter
// Intento 1: inmediato
// Intento 2: 5s
// Intento 3: 30s
// Intento 4: 5min
// Intento 5: 1h
// - DLQ

pub async fn run_retry_worker(db: Arc<dyn EventBusRepo>, config: RetryConfig) {
    loop {
        let pending = db.get_deliveries_due_for_retry().await;
        for delivery in pending {
            tokio::spawn(attempt_delivery(db.clone(), delivery, config.clone()));
        }
        tokio::time::sleep(Duration::from_secs(config.poll_interval_secs)).await;
    }
}

async fn attempt_delivery(db: Arc<dyn EventBusRepo>, delivery: EventDelivery, config: RetryConfig) {
    let result = http_client.post(&delivery.callback_url)
        .header("X-Event-Id", &delivery.event_id)
        .header("X-Event-Topic", &delivery.topic)
        .header("X-Hub-Signature-256", compute_hmac(&delivery.payload, &config.hmac_secret))
        .json(&delivery.payload)
        .timeout(config.timeout)
        .send()
        .await;

    match result {
        Ok(r) if r.status().is_success() => db.mark_delivered(&delivery.id).await,
        Ok(r) => {
            let attempts = delivery.attempts + 1;
            if attempts >= config.max_attempts {
                db.move_to_dlq(&delivery, r.status().to_string()).await;
            } else {
                db.schedule_retry(&delivery.id, backoff(attempts)).await;
            }
        }
        Err(e) => { /* idem */ }
    }
}
```

### Kafka Sink (feature-gated)

```toml
# Cargo.toml
[features]
kafka = ["rdkafka"]

[dependencies]
rdkafka = { version = "0.36", features = ["cmake-build"], optional = true }
```

```rust
// events/src/bus/sinks/kafka.rs
#[cfg(feature = "kafka")]
pub struct KafkaProducerSink {
    producer: FutureProducer,
    topic_prefix: String,  // e.g. "ds-protocol"
}

#[cfg(feature = "kafka")]
impl KafkaSink for KafkaProducerSink {
    async fn send(&self, envelope: EventEnvelope) -> Result<(), KafkaError> {
        // topic Kafka: "{prefix}.{crate}" e.g. "ds-protocol.transfer-agent"
        let kafka_topic = format!("{}.{}", self.topic_prefix, envelope.source_crate);
        // partition key = correlation_id ?? event_id (ordering por entidad)
        let key = envelope.correlation_id.as_ref()
            .map(|id| id.to_string())
            .unwrap_or_else(|| envelope.id.to_string());

        let payload = serde_json::to_string(&envelope)?;
        self.producer.send(
            FutureRecord::to(&kafka_topic)
                .key(&key)
                .payload(&payload)
                .headers(OwnedHeaders::new()
                    .insert(Header { key: "event-topic", value: Some(envelope.topic.as_str()) })
                    .insert(Header { key: "schema-version", value: Some(&envelope.schema_version.to_string()) })
                ),
            Duration::from_secs(5),
        ).await?;
        Ok(())
    }
}
```

---

## Suscripción interna (BFF y data-aggregator)

```rust
// bff/src/event_listener.rs

pub async fn setup_event_listeners(bus: Arc<EventBus>) {
    // Suscribirse a todos los topics de transfer-agent
    let mut rx = bus.subscribe(TopicPattern::new("transfer-agent.*.*"));

    tokio::spawn(async move {
        while let Ok(envelope) = rx.recv().await {
            match envelope.topic.as_str() {
                "transfer-agent.transfer.started" => {
                    let event: TransferStartedEvent = serde_json::from_value(envelope.payload)?;
                    // actualizar estado en BFF / notificar WebSocket
                }
                "transfer-agent.transfer.completed" => { /* ... */ }
                _ => {}
            }
        }
    });
}
```

---

## `EventBusConfig`

```rust
pub struct EventBusConfig {
    pub broadcast_capacity: usize,     // default: 1024
    pub retry: RetryConfig,
    pub kafka: Option<KafkaConfig>,
    pub hmac_secret: String,           // para firmar HTTP callbacks
}

pub struct RetryConfig {
    pub max_attempts: u32,             // default: 5
    pub poll_interval_secs: u64,       // default: 10
    pub timeout: Duration,             // default: 10s
    pub backoff_base_ms: u64,          // default: 5000
}

pub struct KafkaConfig {
    pub brokers: String,               // "localhost:9092"
    pub topic_prefix: String,          // "ds-protocol"
    pub compression: String,           // "snappy"
}
```

---

## Pasos de implementación

### Fase 1 — Tipos base (sin romper nada)

- [ ] `events/src/bus/envelope.rs` — `EventEnvelope`, `Topic`, `TopicPattern`
- [ ] `events/src/bus/into_event.rs` — trait `IntoEvent` + macro `impl_into_event!`
- [ ] `events/src/bus/error.rs` — `EventBusError`
- [ ] Re-exportar desde `events::bus`

### Fase 2 — Persistencia

- [ ] Migración SeaORM: nueva tabla `events`
- [ ] Migración SeaORM: nueva tabla `event_deliveries` (reemplaza `notifications`)
- [ ] Migración SeaORM: nueva tabla `dead_letter_queue`
- [ ] Migración SeaORM: modificar `subscriptions` - `topic_pattern` string + migrar datos booleanos
- [ ] `events/src/data/repo/event_store.rs` — impl `EventBusRepo` con sea-orm
- [ ] Entidades SeaORM para las nuevas tablas

### Fase 3 — Bus y workers

- [ ] `events/src/bus/mod.rs` — struct `EventBus` con `tokio::sync::broadcast`
- [ ] `events/src/bus/workers/http_dispatch.rs` — retry worker con backoff exponencial
- [ ] `events/src/bus/workers/retry_poller.rs` — tokio task que lee `event_deliveries` pendientes
- [ ] HMAC signing en HTTP callbacks (header `X-Hub-Signature-256`)
- [ ] Circuit breaker por subscriber (evitar retry storms)

### Fase 4 — Kafka (feature-gated)

- [ ] Añadir `rdkafka` como dependencia opcional (`features = ["kafka"]`)
- [ ] `events/src/bus/sinks/kafka.rs` — `KafkaProducerSink`
- [ ] `EventBusConfig::kafka: Option<KafkaConfig>`
- [ ] Wiring en `monolith` si `KAFKA_BROKERS` está en env

### Fase 5 — Domain events por crate

Para cada crate dominio:
- [ ] `transfer-agent/src/events.rs` — definir `TransferStartedEvent`, `TransferCompletedEvent`, etc.
- [ ] `negotiation-agent/src/events.rs` — `NegotiationInitiatedEvent`, etc.
- [ ] `dataplane/src/events.rs` — `ChannelOpenedEvent`, `TransferActiveEvent`, etc.
- [ ] `connector/src/events.rs` — `ConnectionEstablishedEvent`, etc.

Inyectar `Arc<EventBus>` en los servicios y publicar en los puntos clave del flujo.

### Fase 6 — BFF subscription interna

- [ ] `bff/src/event_listener.rs` — `setup_event_listeners(bus)`
- [ ] Conectar con WebSocket handler si existe para live updates
- [ ] Añadir `events` como dependencia de `bff`

### Fase 7 — Observabilidad

- [ ] Tracing spans: `publish`, `dispatch`, `retry`, `dlq`
- [ ] Contador de eventos publicados/entregados/fallidos (usando `tracing` + metrics)
- [ ] Endpoint HTTP: `GET /events/bus/stats` — métricas del bus (pending, dlq count, etc.)
- [ ] Health check: `GET /events/bus/health`

### Fase 8 — API HTTP backward compat

- [ ] Mantener `GET/POST /subscriptions` con adaptador - `topic_pattern`
- [ ] Mantener `GET /notifications` que lee de `event_deliveries` mapeado al formato antiguo
- [ ] Deprecate warning en headers de los endpoints legacy

---

## Consideraciones enterprise

| Propiedad | Mecanismo |
|-----------|-----------|
| **At-least-once delivery** | Outbox pattern: DB primero, luego dispatch |
| **Ordering por entidad** | `correlation_id` como Kafka partition key |
| **Schema evolution** | `schema_version` en envelope; cambios solo aditivos |
| **Seguridad callbacks** | HMAC-SHA256 en `X-Hub-Signature-256` |
| **Backpressure** | Canal `broadcast` con capacidad acotada; drops logueados |
| **Circuit breaker** | Por subscriber: pausa tras N fallos consecutivos |
| **DLQ** | Tabla `dead_letter_queue`; endpoint para re-drive manual |
| **Idempotencia** | `event.id` único; consumidores deben deduplicar si necesario |
| **Retry** | Exponential backoff: 5s - 30s - 5min - 1h - DLQ |
| **Observabilidad** | `tracing` spans por fase del lifecycle; métricas HTTP |
| **Multi-tenant** | `source_crate` + `topic` permiten filtrado por origen |
| **Data-aggregator** | Subscribe vía channel interno O consume desde Kafka |

---

## Dependencias a añadir en `events/Cargo.toml`

```toml
tokio = { workspace = true }              # ya existe en workspace
hmac = "0.12"                             # HMAC signing
sha2 = { workspace = true }               # ya existe
futures-util = { workspace = true }       # ya existe
rdkafka = { version = "0.36", features = ["cmake-build"], optional = true }

[features]
default = []
kafka = ["rdkafka"]
```

---

## Flujo completo de un evento

```
transfer-agent llama:
  event_bus.publish(TransferStartedEvent { ... }.into_envelope()).await

EventBus::publish():
  1. INSERT INTO events (id, topic, payload, ...) ← commit
  2. broadcaster.send(envelope.clone())           ← BFF recibe inmediatamente
  3. kafka_sink.send(envelope.clone())            ← spawn; si falla - log
  4. spawn dispatch_http_callbacks(envelope)
     - SELECT subscriptions WHERE topic_pattern matches
     - Para cada una: INSERT INTO event_deliveries (Pending)
     - HTTP POST - si 2xx: UPDATE status=Delivered
                 - si fallo: UPDATE attempts++, next_retry_at=now+backoff
                 - si attempts>=max: INSERT dead_letter_queue

retry_poller (loop tokio):
  - cada 10s: SELECT event_deliveries WHERE status=Failed AND next_retry_at<=now
  - re-intenta con attempt_delivery()
```
