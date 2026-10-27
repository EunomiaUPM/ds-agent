# transfer-agent

Control plane of the Dataspace Protocol transfer process. It keeps transfer processes and the
messages exchanged in them, receives the peer's DSP transfer messages, and hands the data side
to the `dataplane`, which it registers and drives in-process.

It runs as its own binary or as a module of the `monolith`. Standalone, it also serves the OAuth
server its clients authenticate against.

Every DSP message, inbound or sent through the RPC, runs through a `common::flow` runner:
decode, shape rules, load, fact rules, (send), apply, commit.

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
│   ├── entities/        Message types, states, ACK and error, the inbound input and facts,
│   │                    the eunomia RPC command and facts, RDF extractors, TCK steps
│   ├── services/
│   │   ├── dsp_flow/              Inbound flow: loader, one handler per message
│   │   ├── rpc_flow/              Outbound flow: loader, send, one handler per message, result view
│   │   ├── dsp_persistence/       DspPersistenceTrait: process writes and message history
│   │   ├── validator/             Pure rules, and the rules of the DSP and RPC flows
│   │   └── tck/                   TCK script table, guard and player over `common::tck`
│   ├── http/            DSP routes and RPC routes, each with its flow adapter
│   ├── setup/           DspContext (persistence and runners) and DspModule (routes, TCK worker)
│   └── facades/         Dataplane hooks (one strategy per role and mode)
└── setup/           TransferAgentModule, admin module, AppContext, ports, boot
proto/               transfer_process.proto, transfer_messages.proto
tests/
├── entities/        Message envelopes
├── services/        Process and message services by operation, TCK script
├── protocols/       Inputs, RDF extraction, ACK, idempotency keys, DSP and RPC rules
├── adapters/        gRPC services and the DSP routes over the flow
└── support/         An AppContext over mocks
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
| GET | `/{id}` | The `TransferProcess` of a process the peer takes part in |

`common::dsp_common::PeerAuth` checks the peer's token with the auth agent and looks its
participant up. A missing or rejected token answers 404, the same as a missing process, so
probing cannot reveal which processes exist (DSP 10.1.2.3).

Each message is extracted stage by stage: raw request, JSON, RDF expansion, then the typed
fields with the idempotency key (`TransferDspRaw` → `Parsed` → `Rdf` → `Typed`). The flow then:

1. Checks the shape: the body's type is the route's, pids and agreement are URNs.
2. Loads the domain stage (`TransferDspLoader`): process, agreement, connector, role and direction. An
   unknown process answers 404.
3. Replays a repeated message (`common::flow::IdempotencyGuard`): keyed on peer, version, type
   and pids, guarded by the canonical hash, with the process version telling a restart from a
   retry.
4. Checks the facts: state transition and role (DSP 9.1), who may resume a suspension, pid
   correlation, data address.
5. Applies: a request opens the process (201 with the `TransferProcess`), any other message
   moves it (200). The message is kept with its canonical form and the dataplane hooks run.

Failures answer a `TransferError` with every violation.

## RPC

`/dsp/current/transfers/rpc/setup-{request,start,completion,suspension,termination}` make the
agent send a message. Bodies are JSON-LD in the eunomia context (`common::eunomia`), typed
`RpcTransfer{Request,Start,…}Message`; continuations name the transfer by `@id`, its process
id on this agent, never by a DSP pid. The answer is an `RpcTransferResult` with the request, the
peer's answer and the process. The loader reads the process and the agreement it runs under;
a request must go to the agreement's provider. With `DSP_TCK_MAPPING` set, a worker plays the connector's part of
each TCK scenario and `/tck/transfers/requests` is mounted.

## Integration

`TransferPorts` gathers what the agent consumes:

| Port | `local` (monolith) | `remote` (standalone) |
|---|---|---|
| Auth | The auth module's adapters | Auth agent API (`ssi_auth`) |
| Negotiation | `common::facades::negotiation`, served by the negotiation module | Negotiation agent API (`contracts`) |
| Catalog | `common::facades::catalog`, served by the catalog module | Catalog agent API (`catalog`) |
| Connectors | The catalog module's connector instances | Connector instances over the catalog API |
| Dataplane | Always in-process | Same |

The agent depends on neither the negotiation nor the catalog crate: it reads them through the
contracts they publish in `common`. `TransferPorts::connector_for` resolves the connector
instance of a target in a format (dataset, distribution, instance).

Remote ports use the service token.

With an event bus, the services publish `transfers:process:{create,edit,delete}` and
`transfers:message:{create,delete}`.

## Setup

`TransferAgentModule::compose(config, root, bus, ports)` builds `AppContext` (config, repos,
event sink, ports and the two services; every service, router, flow and loader keeps it whole)
and groups three modules:

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
limited to what the caller's owner scope sees; lists are paged with `limit` and `cursor`.

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

`tests/protocols` covers the DSP layer: inbound inputs and eunomia RPC bodies, RDF extraction,
message types, the ACK shape (DSP 9.3.1), idempotency keys and process versions, and the
validation registries. `tests/adapters` runs the DSP routes through the flow over an in-memory
store, and the gRPC services. `tests/services` covers both services by operation, the agreement
resolver and the TCK scenarios. No database is needed; `crates/monolith/tests/dsp/transfer.rs`
runs the DSP and RPC flows between two agents.
