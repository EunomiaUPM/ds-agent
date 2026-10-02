# transfer-agent

Control plane of the Dataspace Protocol transfer process. It keeps transfer processes and the
messages exchanged in them, receives the peer's DSP transfer messages, and hands the data side
to the `dataplane`, which it registers and drives in-process.

It runs as its own binary or as a module of the `monolith`. Standalone, it also serves the OAuth
server its clients authenticate against.

The DSP side is being rebuilt (see `PLAN.md`). The endpoints receive and validate messages, but
they do not yet load the process from the domain, run the manager, or answer with the DSP ACK.
The sections below say which parts are wired.

## Structure

```
src/
├── lib.rs           Constants: SERVICE_NAME (`transfer-agent-ref`), EVENT_PREFIX
├── main.rs          AgentCli::<TransferBoot>
├── entities/        Transfer processes and their identifiers, messages and envelopes, filters
├── services/        Transfer process and transfer message use cases
├── data/            Repository traits, SeaORM repositories and migrations
├── http/            Management routers
├── grpc/            gRPC mirror of the management API
├── protocols/dsp/
│   ├── http/            DSP transfer endpoints and the peer auth middleware
│   ├── entities/        Message types, states, contexts per stage, ACK, idempotency, RDF extraction
│   ├── services/
│   │   ├── dsp_handler_pipeline/  Inbound stages: raw, parsed, RDF, typed, domain
│   │   ├── validator/             Payload rules and state transition rules, per message type
│   │   ├── dsp_domain_loader/     Process, agreement, connector and role of a message
│   │   ├── connector_resolver/    Agreement to dataset, distribution and connector instance
│   │   └── manager/               One strategy per message type over a fixed phase order
│   └── facades/         Catalog, negotiation and dataplane (one strategy per role and mode)
└── setup/           TransferAgentModule, admin module, AppContext, ports, boot
proto/               transfer_process.proto, transfer_messages.proto
openapi.yaml         Management API description
tests/
├── entities/        Message envelopes
├── services/        Process and message services, split by operation
├── protocols/       Contexts, RDF extraction, ACK, idempotency, validators, DSP routes
└── adapters/        gRPC services with mocked services and a stub validator
```

## DSP

Mounted at `/dsp/current/transfers`:

| Method | Path | Message |
|---|---|---|
| POST | `/request` | TransferRequestMessage |
| POST | `/{id}/start` | TransferStartMessage |
| POST | `/{id}/completion` | TransferCompletionMessage |
| POST | `/{id}/termination` | TransferTerminationMessage |
| POST | `/{id}/suspension` | TransferSuspensionMessage |
| GET | `/{id}` | Transfer process; returns a placeholder for now |

The peer's token is checked with the auth agent. A missing or rejected token answers 404, the
same as a missing process, so probing cannot reveal which processes exist (DSP 10.1.2.3).

Each message goes through `DSPHandlerPipeline`. Every stage consumes the previous context, so
the order cannot be skipped:

1. **Raw**: body, headers and the authenticated peer.
2. **Parsed**: JSON, with the protocol version and the message type the route fixes.
3. **RDF**: JSON-LD expansion and RDFC-1.0 canonical n-quads, through `common::rdf::dsp`.
4. **Typed**: the DSP 9.2 fields read off the expanded document, and the idempotency key.
5. Edge validation: payload rules such as pid correlation and when a data address may be sent.
6. **Domain**: process, agreement, connector instance, role and direction. Today this stage
   builds a placeholder context; `DspDomainLoader` exists but is not called.
7. Domain validation: which role may send each message in each state (DSP 9.1), and who may
   resume a suspension.

A message that passes answers 202 with no body; one that fails answers 400. Idempotency is keyed
by message type and pids, guarded by the canonical hash and the process version, so a restart
after a suspension is told apart from a retry. `TransferManager` runs one strategy per message
type over the same phases, and the dataplane facade has one strategy per role and mode. The
pipeline does not call the manager yet, and the strategies' phases are `todo!()`.

## Integration

`TransferPorts` gathers what the agent consumes:

| Port | `local` (monolith) | `remote` (standalone) |
|---|---|---|
| Auth | The auth module's adapters | Auth agent API (`ssi_auth`) |
| Negotiation | The negotiation agent's agreement service | Negotiation agent API (`contracts`) |
| Catalog | The catalog's dataset, distribution and offer services | Catalog agent API (`catalog`) |
| Dataplane | Always in-process; connector instances from the catalog module | Same; connector instances over the catalog API |

Remote ports use the service token. In the monolith, cross-tenant lookups run as the service
client's tenant.

With an event bus, the services publish `transfers:process:{create,edit,delete}` and
`transfers:message:{create,delete}`.

## Setup

`TransferAgentModule::compose(config, root, bus, ports)` builds `AppContext` and groups three
modules:

| Module | Mounts |
|---|---|
| `DspModule` | `/dsp/current/transfers` |
| `TransferAdminModule` | `{api}/transfer-agent-ref` and `{api}/transfer-agent`, plus the gRPC services |
| The dataplane's `DataplaneModule` | `{api}/transfer-agent/dataplane` and `/dataplane/proxy` |

`migrations()` lists the transfer tables, then the dataplane's. The dataplane needs Redis at the
`cache` URL.

Standalone, `TransferBoot` also registers `OAuthModule` and its `AdminSeeder`, and migrates
the OAuth tables:

```
cargo run -p transfer_agent -- setup -e <config.yaml>   # vault secrets and migrations
cargo run -p transfer_agent -- start -e <config.yaml>   # seeders, then serve
```

Its config section is `transfer`: `common`, `cache`, `contracts`, `catalog`, `ssi_auth` and
`is_catalog_datahub`.

## HTTP API

Under `{api}/transfer-agent-ref` (also `{api}/transfer-agent`), bearer required. Reads are
limited to the caller's tenant; lists are paged with `limit` and `cursor`.

| Resource | Routes |
|---|---|
| `/transfer-processes` | `GET /`, `POST /`, `POST /batch`, `GET /{id}` |
| `/transfer-messages` | `GET /`, `POST /`, `GET\|DELETE /{id}`, `GET /process/{process_id}` |

A process is returned with its identifiers (`consumerPid`, `providerPid` and others). The gRPC
services in `proto/` mirror both resources.

## Tests

```
cargo test -p transfer_agent
```

`tests/protocols` covers the DSP layer: raw and RPC contexts, RDF canonicalization and field
extraction, message types, the ACK shape (DSP 9.3.1), idempotency keys and process versions, the
payload and transition validators, and the DSP routes with an auth facade that accepts or rejects
every token. `tests/services` covers both services by operation, including tenant isolation and
roles. `tests/adapters` covers the gRPC services. No database is needed. There is no DSP
transfer flow between agents in `crates/monolith/tests/` yet.
