# Kafka testing bridge — design & validation

> Status: **design proposal** (not implemented). Mirrors the structure of the
> existing `crates/dataplane/src/testing_proxy/http` so the two transports share
> the same decomposition style.

## 1. Motivation

The dataplane already ships a **testing HTTP proxy**
(`testing_proxy/http/http.rs`): an axum router that, for an active transfer,
forwards an incoming HTTP request to the upstream declared in the transfer's
egress config, injecting the resolved credentials.

We want the equivalent for **Kafka**. The goal of this document is to describe
*how* it would look, where the pieces go, what the connector-side
`resource/kafka.rs` spec should contain, and — most importantly — **how to
validate the whole thing**.

## 2. The conceptual difference (read this first)

HTTP and Kafka do **not** share an interaction model, so "proxy" does not
translate 1:1.

| | HTTP proxy | Kafka |
|---|---|---|
| Model | request / response, **synchronous** | publish / subscribe, **async streaming** |
| Trigger | reactive — does nothing until a request arrives | proactive — a long-running consume loop |
| "Forward" | send one HTTP request, return the response | consume from ingress topic → produce to egress topic |
| Public surface | `router() -> Router` | `run(shutdown) -> Outcome<()>` (a loop, not a router) |
| Failure handling | return a status code | commit / retry / dead-letter decision |

So the Kafka equivalent is a **bridge / relay**, not a protocol proxy. It maps
naturally onto the pub/sub abstraction the crate already has
(`DriverPubSubTrait` with `subscribe` / `unsubscribe`).

## 3. What is reused vs. what is new

The decomposition pattern from the HTTP refactor transfers almost entirely.
Only the two ends (ingress and egress) change.

| HTTP proxy method | Kafka bridge equivalent | Reuse |
|---|---|---|
| `router()` | `run(shutdown)` (consume loop) | new |
| `forward_request_*` (axum handlers) | `consume_loop` (poll consumer) | new |
| `parse_dataplane_id` (from URL path) | `extract_dataplane_id` (from message **key** or **header**) | adapted |
| `load_started_dataplane` | `load_started_dataplane` | **identical** |
| `parse_egress` (→ `HttpProxy`) | `parse_egress` (→ `KafkaProxy`) | adapted |
| `extract_outbound` (method/headers/body) | `extract_record` (key/headers/payload/timestamp) | adapted |
| `build_target_url` | `build_target_record` (topic + key/partition) | adapted |
| `resolve_credentials` (Bearer/OAuth) | `resolve_credentials` (SASL/SSL) | **mostly reused** |
| `forward` (HTTP request) | `produce` (send to egress topic) | new |
| `record_event` | `record_event` | **identical** |
| `map_response` / `relay_response` | `commit_offset` + delivery policy | new |
| `ProxyError: IntoResponse` | `BridgeError` (drives commit/retry/DLQ) | adapted |

`load_started_dataplane`, `parse_egress`, `resolve_credentials` and
`record_event` are transport-agnostic and should be **extracted into a shared
module** so HTTP and Kafka don't duplicate them.

## 4. File layout

```
crates/connector/src/entities/resource/kafka.rs   # KafkaSpec (already exists, extend)
crates/dataplane/src/entities/dataplane_manager/dataplane_proxy.rs
        #  +  DataplaneProxyEgress::KafkaProxy { ... }
crates/dataplane/src/entities/dataplane_drivers/pubsub/kafka.rs   # the pub/sub driver
crates/dataplane/src/testing_proxy/kafka/kafka.rs                 # the testing bridge
```

Recommended client crate: **`rdkafka`** (async binding over librdkafka, the de
facto standard in Rust).

## 5. The connector spec: `resource/kafka.rs`

The connector spec describes the **transport** (parameterisable, non-secret).
Credentials stay in `AuthenticationConfig` as `SecretString` — same separation
as HTTP. Today the spec is minimal:

```rust
pub struct KafkaSpec {
    pub brokers: TemplateVecString,
    pub topic: TemplateString,
    pub group_id: Option<TemplateString>,
}
```

### 5.1 Proposed extension

```rust
use crate::entities::parameters::{TemplateMapString, TemplateString};
use crate::TemplateVecString;
use serde::{Deserialize, Serialize};

/// Kafka protocol specification.
///
/// All string fields support `{{__PARAM__}}` placeholders. Secrets (SASL
/// password / OAUTHBEARER token) are NOT here — they live in
/// `AuthenticationConfig` as `SecretString`, exactly like HTTP.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KafkaSpec {
    /// Broker addresses (e.g. `["localhost:9092"]`).
    pub brokers: TemplateVecString,
    /// Topic name.
    pub topic: TemplateString,
    /// Consumer group ID. Required when the spec is used on the consume side.
    pub group_id: Option<TemplateString>,

    /// Wire security. Defaults to `Plaintext`.
    #[serde(default)]
    pub security_protocol: KafkaSecurityProtocol,
    /// SASL mechanism, when `security_protocol` uses SASL.
    pub sasl_mechanism: Option<KafkaSaslMechanism>,

    /// Extra librdkafka client properties passed through verbatim
    /// (e.g. `{"compression.type": "zstd"}`). Escape hatch — keep it small.
    pub client_properties: Option<TemplateMapString>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum KafkaSecurityProtocol {
    #[default]
    Plaintext,
    Ssl,
    SaslPlaintext,
    SaslSsl,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum KafkaSaslMechanism {
    Plain,
    ScramSha256,
    ScramSha512,
    OauthBearer,
}
```

### 5.2 How credentials map

The bridge resolves `AuthenticationConfig` → librdkafka SASL settings:

| `AuthenticationConfig` | SASL mechanism | librdkafka properties |
|---|---|---|
| `NoAuth` | — | `security.protocol=PLAINTEXT` |
| `BasicAuth { username, password }` | `PLAIN` / `SCRAM-SHA-*` | `sasl.username`, `sasl.password` |
| `OAuth2 { ... }` | `OAUTHBEARER` | token callback via `sasl.oauthbearer.*` |
| `BearerToken { token }` | `OAUTHBEARER` (static) | static token as bearer |
| `ApiKey { ... }` | n/a | reject — not a Kafka concept |

This mapping is the natural place to enforce "SASL mechanism requires a
compatible `AuthenticationConfig`" (see validation below).

## 6. The bridge skeleton (illustrative)

```rust
pub struct TestingKafkaBridge {
    consumer: StreamConsumer,
    producer: FutureProducer,
    dataplane_service: Arc<dyn DataplaneTransfersEntitiesTrait>,
    repo: Arc<dyn DataplaneRepoTrait>,
    keystore: Option<Arc<dyn KeystoreLookup>>,
}

impl TestingKafkaBridge {
    pub fn new(...) -> Self { /* build clients from resolved spec + auth */ }
    pub fn with_keystore(self, k: Arc<dyn KeystoreLookup>) -> Self { /* ... */ }

    /// Was `router()`. Runs the consume loop until cancelled.
    pub async fn run(self, shutdown: CancellationToken) -> Outcome<()> { /* ... */ }

    async fn consume_loop(&self, shutdown: CancellationToken) -> Outcome<()> {
        // poll → handle_record → commit, honouring `shutdown`
    }

    async fn handle_record(&self, msg: &BorrowedMessage<'_>) -> Result<(), BridgeError> {
        let id        = Self::extract_dataplane_id(msg)?;          // adapted
        let dataplane = self.load_started_dataplane(&id).await?;   // reused
        let egress    = Self::parse_egress(&dataplane)?;           // adapted
        let record    = Self::extract_record(msg)?;                // adapted
        let creds     = self.resolve_credentials(&dataplane, &egress).await; // reused
        let result    = self.produce(&egress, record, &creds).await;
        self.record_event(&dataplane, &id, &egress, &result).await; // reused
        result
    }

    // produce / commit_offset / dead_letter ...
}

/// Drives the per-record decision instead of mapping to an HTTP status.
enum BridgeError {
    MissingDataplaneId,   // skip + DLQ
    DataplaneNotFound,    // skip + DLQ
    NotStarted,           // skip (transfer not active)
    UnsupportedEgress,    // skip + DLQ
    ProduceFailed,        // retry (do NOT commit)
}
```

## 7. Kafka-specific concerns the HTTP proxy never had

These are the design decisions that have no HTTP analogue and must be made
explicit:

1. **Delivery guarantee & offset commit.** At-least-once (produce *then* commit
   → possible duplicates) vs at-most-once (commit *then* produce → possible
   loss). Default recommendation: **at-least-once + idempotent producer**
   (`enable.idempotence=true`).
2. **Consumer group & rebalances.** Handle rebalance callbacks; on partition
   revocation, commit what is pending.
3. **Poison messages / Dead Letter Topic.** In HTTP a bad request returns 502
   and is done. In Kafka, refusing to commit a permanently-failing message
   **blocks the partition forever** — a DLQ (or controlled skip) is mandatory.
4. **Key / partition / header preservation.** Reuse the source key when
   producing to keep per-key ordering; propagate headers.
5. **Auth lifetime.** HTTP resolves credentials per request. Kafka SASL is set
   when the **client is built**, not per message — so credential resolution
   happens at startup, and OAUTHBEARER needs a refresh callback.
6. **Backpressure.** You own the pace (poll size, in-flight produces).
7. **Graceful shutdown.** Drain in-flight produces before exiting; honour a
   `CancellationToken` (axum handled this for you before).

## 8. How to validate everything

Validation happens at **five layers**, cheapest first.

### 8.1 Type level (free, compile time)
- `ProtocolSpec` is a serde-tagged enum (`#[serde(tag = "protocol")]`), so an
  unknown protocol fails to deserialize.
- `KafkaSecurityProtocol` / `KafkaSaslMechanism` as enums reject invalid strings
  at parse time.
- Secrets are `SecretString` — the type system keeps them out of logs/specs.

### 8.2 Spec validation (`KafkaSpec::validate`)
Add a `validate(&self) -> Outcome<()>` invoked when a connector template is
registered, alongside the existing `TemplateParametersValidator`:

- `brokers` non-empty; each entry looks like `host:port`.
- `topic` non-empty and matches Kafka's legal charset (`[a-zA-Z0-9._-]`,
  ≤ 249 chars, not `.`/`..`).
- `group_id` present when the spec is used on the consume side.
- **SASL coherence**: `sasl_mechanism.is_some()` iff `security_protocol` is
  `SaslPlaintext | SaslSsl`; and the mechanism must be compatible with the
  transfer's `AuthenticationConfig` (§5.2). E.g. `SCRAM-*` requires `BasicAuth`;
  `OAUTHBEARER` requires `OAuth2`/`BearerToken`.

### 8.3 Template-parameter validation (existing machinery)
`KafkaSpec`'s `Template*` fields flow through the same
`ConnectorTemplateWalker` + `TemplateParametersValidator` used today: every
`{{__PARAM__}}` placeholder must be declared, and every declared parameter must
be used. No new validator needed — just make sure the walker visits the new
fields.

### 8.4 Preflight connection validation (startup)
Before the bridge enters its loop, run a cheap **admin/metadata check**:
- fetch cluster metadata with the resolved client config → proves brokers are
  reachable and auth works;
- confirm the ingress topic exists (and egress, or auto-create per policy);
- fail fast with a clear `DataplaneError` instead of looping on a broken config.

### 8.5 Runtime / integration validation (tests)
- **Unit**: `KafkaSpec::validate` and the auth→SASL mapping with table tests
  (mirror the routing tests in `dataplane_handlers_strategy.rs`).
- **Integration**: spin up a real broker with **`testcontainers`** (Redpanda is
  a fast, single-binary option), then assert end-to-end:
  - a record on the ingress topic appears on the egress topic;
  - **delivery guarantee** holds — kill the bridge mid-flight and verify
    at-least-once (no loss; duplicates tolerated);
  - a poison message lands in the DLQ and does **not** block the partition;
  - key/headers are preserved;
  - `record_event` rows are written for each forwarded message.
- **Contract**: reuse the `MockDataplaneTransfersEntitiesTrait` pattern to test
  `handle_record` without a real broker (mock the entity, feed a fake
  `BorrowedMessage`).

## 9. Open decisions for the team

1. Where does the **dataplane id** travel — message key, a header
   (`x-dataplane-id`), or is it implied by the topic? This drives
   `extract_dataplane_id`.
2. **DLQ topic** naming/policy and max-retry before dead-lettering.
3. Default **delivery guarantee** (recommend at-least-once + idempotent).
4. Should `parse_egress`/`resolve_credentials`/`load_started_dataplane` be
   lifted into a shared `proxy_common` module now (before duplication)?

---

*In one line: keep the small-method + error-enum decomposition from the HTTP
proxy; swap the reactive `Router` for a consume loop, and swap "return a
response" for "produce + commit under a chosen delivery policy" — and put the
real engineering budget into validation layers 8.2–8.5.*
