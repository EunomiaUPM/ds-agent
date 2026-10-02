# events

Event bus shared by the modules of a process. Every published event is stored, broadcast to
in-process listeners and delivered to the webhooks of its tenant that subscribed to its topic.
Failed deliveries are retried with backoff and end in a dead letter queue, from which they can
be replayed.

The crate has no binary. The `monolith` serves it and hands its bus to the other modules;
standalone agents run without a bus, and their modules then publish nothing.

## Structure

```
src/
├── lib.rs           Re-exports the bus, envelope, topic types, views and the event traits
├── entities/
│   ├── topic.rs         Topic: a concrete name such as `transfers:started`
│   ├── topic_pattern.rs TopicPattern: `*` and `**` wildcards, and its SQL regex
│   ├── envelope.rs      EventEnvelope: metadata plus the JSON payload
│   ├── event.rs         Event trait for typed payloads
│   ├── mac.rs           event! and emit_action! macros
│   ├── subscription.rs, delivery.rs, dead_letter.rs   Stored records
│   └── commands.rs, queries.rs, dto.rs                API bodies and list filters
├── services/event_bus/
│   ├── service.rs       EventBus: publish, subscribe, dead letter replay
│   ├── dispatcher.rs    EventDispatcher: one signed webhook POST
│   ├── worker.rs        RetryWorker: redelivers due deliveries
│   └── policy.rs        RetryPolicy: backoff, retryable statuses, timeouts
├── data/            Repository traits, SeaORM repositories and migrations
├── http/            Events feed and SSE stream, subscriptions, dead letters
└── setup/           EventsModule and AppContext
tests/
├── entities/        Topics and topic patterns
├── services/        Bus, dispatcher, retry policy and worker, against a local webhook
└── support/         Fixtures, a local webhook server and its signals
```

## Topics and envelopes

A topic is a name of segments separated by `.` or `:`, without wildcards, such as
`oauth:user:create`. A subscription or a filter uses a `TopicPattern`:

| Pattern | Matches |
|---|---|
| `*` | Every topic |
| `transfers.*` | Every topic under `transfers` |
| `transfers.*.started` | One segment in the middle |
| `**` | Any number of segments, including none |
| `transfers:started` | That topic only |

Each event travels in an `EventEnvelope`: `id` (`urn:uuid`), `tenant_id`, `topic`,
`source_crate`, `schema_version`, `timestamp`, `correlation_id?`, `trace_context?` (the
`traceparent` of the publishing span) and `payload`. The tenant is the one that owns the record
the event is about.

## Publishing

Modules publish in one of two ways:

```rust
// A typed event: the struct carries `tenant_id` and the macro fixes its topic and source.
events::event! {
    #[derive(Serialize)]
    pub struct TransferStarted { pub tenant_id: String, pub process_id: String }
        => "transfers:started", "transfer-agent"
}
bus.publish_event(TransferStarted { tenant_id, process_id }).await?;

// A CRUD action on a record, on an optional bus: topic `<prefix><entity>:<action>`.
events::emit_action!(self.event_bus, &view.tenant_id, crate::EVENT_PREFIX, "user", "create", &view);
```

`emit_action!` does nothing when the bus is `None` and only logs a failed publish, so a module
works the same with or without a bus. In-process listeners call `EventBus::subscribe` and get a
`broadcast::Receiver` of every envelope published after that.

`publish` does three things, in order: it stores the envelope, broadcasts it, and creates a
pending delivery for each active subscription of the tenant whose pattern matches the topic. A
failure to store fails the publish before anything else happens.

## Webhook delivery

Each delivery is first attempted right after the publish, in a background task. The dispatcher
POSTs the event's `payload` (not the whole envelope) with these headers:

| Header | Value |
|---|---|
| `X-Event-Id`, `X-Event-Topic`, `X-Event-Timestamp` | From the envelope |
| `X-Correlation-Id` | When the envelope has one |
| `X-Hub-Signature-256` | `sha256=<hex>` HMAC-SHA256 of the body, when the subscription has a `secret` |
| `traceparent` | The delivery span, linked to the publisher's trace |
| The subscription's `headers` | As given |

A 2xx marks the delivery delivered. A 408, 429 or 5xx, or an unreachable webhook, schedules
another attempt; any other status goes straight to the dead letter queue. `RetryWorker` polls
for due deliveries, 50 at a time, and gives up after the subscription's `retry_limit`, or the
policy's `max_attempts` when it has none.

`RetryPolicy::default()`:

| Field | Default |
|---|---|
| `max_attempts` | 5 |
| `initial_backoff_secs`, `multiplier`, `max_backoff_secs` | 5 s, ×2, capped at 3600 s |
| `jitter_factor` | 0.2, never under one second |
| `timeout_secs` | 10, per webhook request |
| `poll_interval_secs` | 5, between worker batches |

A dead letter keeps the payload, the callback, the error and the attempt count. Replaying it
sends the stored event again to the current subscription and marks it replayed on success.

## Setup

`EventsModule` implements `ServiceModuleTrait`:

- `EventsModule::compose(root)` builds the bus and the retry worker on the shared database with
  the default policy.
- `event_bus()` returns the bus; the composition root passes it to every other module before
  registering them.
- `migrations()` creates `events`, `subscriptions`, `event_deliveries` and
  `dead_letter_queue`.
- `workers()` returns the `RetryWorker`, which the boot runs until shutdown.

The routes are mounted under `/api/v1/events`, behind the process's token validator.

## HTTP API

Lists are paged with `limit` and `cursor`. Reads are limited to the caller's tenant; admins see
every tenant unless they pin one with `x-tenant-id`. Writes need the `Owner` or `Admin` role,
and what they create always belongs to the caller's tenant.

| Method | Path | Response |
|---|---|---|
| GET | `/` | 200, page of stored events; filter `topic` (a pattern) |
| POST | `/` | 201, the published envelope. Body: `topic`, `payload`, `source_crate?`, `schema_version?`, `correlation_id?` |
| GET | `/stream` | Server-sent events of the caller's tenants as they are published; `topic` narrows them, `tenant` stands in for `x-tenant-id` |
| GET | `/{id}` | 200, the event; `id` is a UUID or `urn:uuid:…` |
| GET | `/{id}/deliveries` | 200, its deliveries |
| GET | `/subscriptions` | 200, page of subscriptions; filter `active` |
| POST | `/subscriptions` | 201, the subscription. Body: `callback_address`, `topic_pattern`, `secret?`, `headers?`, `retry_limit?`, `expiration_time?` |
| GET | `/subscriptions/{id}` | 200, the subscription; 404 if missing |
| PUT | `/subscriptions/{id}` | 200, the subscription; absent fields stay as they are, `active` can be set |
| DELETE | `/subscriptions/{id}` | 204 |
| GET | `/dlq` | 200, page of dead letters; filter `status` (`Unresolved`, `Replayed`, `Purged`) |
| GET | `/dlq/{id}` | 200, the dead letter |
| POST | `/dlq/{id}/replay` | 200, the resulting delivery; an error if the webhook fails again |
| POST | `/dlq/replay-all` | 200, `{ "replayed_count": n }` |
| DELETE | `/dlq/{id}` | 204 |

The SSE stream sends unnamed events, so a browser `EventSource` receives them in `onmessage`,
and a keep-alive every 15 seconds. Events missed while a slow client lags are skipped.

## Tests

```
cargo test -p events
```

`tests/entities` covers topic validation and pattern matching. `tests/services` covers the
retry policy, the dispatcher's headers and signature, publishing and the first delivery with
each outcome, dead letter replay, and the retry worker, with mocked repositories and a local
webhook server from `tests/support`. No database is needed. The SeaORM repositories, including
the SQL form of topic patterns, and the HTTP routers have no tests yet.
