# negotiation-agent

Contract negotiation for the connector. It runs the Dataspace Protocol negotiation state machine
on both sides: as provider when a peer requests a contract, and as consumer when the tenant
negotiates with a peer. It keeps the processes, the messages exchanged, the offers and the
resulting agreements.

Peers drive it through the DSP negotiation endpoints. The tenant drives it through RPC endpoints
that send DSP messages to peers, and manages the records through a REST API and its gRPC mirror.
It runs as its own binary or as a module of the `monolith`.

## Structure

```
src/
├── lib.rs           Re-exports the migrations, AgreementView and OfferView
├── main.rs          AgentCli::<NegotiationAgentBoot>
├── entities/        Negotiation processes, messages, offers, agreements, filters
├── services/        One use case per entity
├── data/            Repository ports (repo/), SeaORM adapter (sea_orm/), DataFactory
├── http/            Management routers
├── grpc/            gRPC mirror of the management API
├── protocols/dsp/
│   ├── entities/        Message types, states (DSP 8.1 table), roles, the hydrated process,
│   │                    DSP and RPC stages, RDF extractors, ACK, error, TCK messages
│   ├── services/        dsp_flow/ (peer messages), rpc_flow/ (our messages), dsp_persistence/,
│   │                    validator/ (rules, dsp, rpc), tck/ (script, guard, player)
│   ├── http/            dsp/, rpc/ and bff/ routers with their flow adapters
│   └── setup/           DspContext (persistence, runners, TCK queue) and DspModule
├── facades/         The published contract served in-process (agreements as ODRL)
└── setup/           NegotiationAgentModule, admin module, AppContext, ports, boot
tests/
├── entities/        Filters
├── services/        Every service, the DSP persistence and the TCK script
├── protocols/       Message types, state machine, RDF extraction, stages and rules
├── adapters/        gRPC services, and the DSP routes end to end over an in-memory store
└── support/         TestContext and fixtures
```

## Negotiation

States: `REQUESTED`, `OFFERED`, `ACCEPTED`, `AGREED`, `VERIFIED`, `FINALIZED`, `TERMINATED`.
A process is identified by its `consumerPid` and `providerPid`; pids are opaque.

Every message, received or sent, runs through a `common::flow` runner. A peer's message is read
stage by stage (request, JSON, RDF expansion, typed fields), checked for its DSP shape, loaded
(process, our role, offers), checked against the DSP 8.1 table, our role and the pids, and
applied by one handler per message, which stores the state change, the message and the offer or
agreement it carries. A repeated message gets the first answer. Our own messages go the same
way: the RPC body is read through the eunomia context, the DSP message is built and checked,
sent with the peer token, and stored once the peer accepts it.

When a peer's contract request opens a negotiation, its offer must be one the catalog publishes,
on the target the request names.

## DSP

Mounted at `/dsp/current/negotiations`. Every endpoint requires a peer token, checked with the
auth agent; the peer acts on the tenant it is associated with.

| Method | Path | Section | Side |
|---|---|---|---|
| GET | `/{id}` | DSP 8.2.1, 8.3.2 | Both |
| POST | `/request` | DSP 8.2.2 | Provider: the consumer starts |
| POST | `/{id}/request` | DSP 8.2.3 | Provider: the consumer counters |
| POST | `/{id}/events` | DSP 8.2.4, 8.3.6 | Both: `ACCEPTED` or `FINALIZED` |
| POST | `/{id}/agreement/verification` | DSP 8.2.5 | Provider |
| POST | `/{id}/termination` | DSP 8.2.6, 8.3.7 | Both |
| POST | `/offers` | DSP 8.3.3 | Consumer: the provider starts |
| POST | `/{id}/offers` | DSP 8.3.4 | Consumer: the provider counters |
| POST | `/{id}/agreement` | DSP 8.3.5 | Consumer |

A first request or offer answers 201 with the ACK; the rest answer 200 with it. Errors carry the
`ContractNegotiationError`: 404 for a missing or foreign process or an unknown peer, so a peer
cannot probe other negotiations, 400 for a broken rule and 409 for a stale or conflicting
repeat.

## RPC

Under the same prefix, bearer required. Each endpoint sends one DSP message to the peer on behalf
of the user, stores the result locally and answers an eunomia `RpcNegotiationResult`
(`request`, `response`, `negotiationAgentModel`): 201 when it opened a negotiation, 200 otherwise.
Bodies are eunomia JSON-LD: `@context` `https://w3id.org/eunomia/v1/context.jsonld` and `@type`
the RPC; continuations name the negotiation by `@id`, its process id on this agent, never by a
DSP pid. A provider's offer given only by `@id` goes out as the catalog publishes it, with its
rules and `target`.

| Path | `@type` | Message sent |
|---|---|---|
| `/rpc/setup-request-init` | `RpcNegotiationRequestInitMessage` | First request, to `providerAddress`, with `offer` |
| `/rpc/setup-request` | `RpcNegotiationRequestMessage` | Counter request, with `offer` |
| `/rpc/setup-offer-init` | `RpcNegotiationOfferInitMessage` | First offer, to `consumerAddress`, with `offer` |
| `/rpc/setup-offer` | `RpcNegotiationOfferMessage` | Counter offer, with `offer` |
| `/rpc/setup-acceptance` | `RpcNegotiationAcceptanceMessage` | `ACCEPTED` event |
| `/rpc/setup-agreement` | `RpcNegotiationAgreementMessage` | Agreement on the last offer |
| `/rpc/setup-verification` | `RpcNegotiationVerificationMessage` | Agreement verification |
| `/rpc/setup-finalization` | `RpcNegotiationFinalizationMessage` | `FINALIZED` event |
| `/rpc/setup-termination` | `RpcNegotiationTerminationMessage` | Termination, with `code` and `reason` |

`/bff-rpc/setup-{request-init,offer-init,acceptance,agreement,termination}` are what the admin
GUI calls; each sends one message, as its RPC does. The consumer verifies and the provider
finalizes with their own RPCs.

With `DSP_TCK_MAPPING` set, `/tck/negotiations/requests` makes the consumer open a negotiation
for the DSP TCK (`datasetId`, `offerId`, `providerId`, `connectorAddress`), and a worker plays
the connector's part of each CN and CN_C test, keyed by the offer seeded for it.

## Integration

- `NegotiationAgentModule::negotiation_facade()` serves the published contract
  (`common::facades::negotiation`: agreements as ODRL) in-process to the transfer agent.
- `NegotiationPorts::local(auth, catalog)` takes the auth module and the catalog module's
  contract in-process; `NegotiationPorts::remote` calls the auth and catalog agents' APIs
  (`ssi_auth` and `catalog` in the config). The agent does not depend on the catalog crate.
- With an event bus, the services publish `negotiations:process:{create,edit,delete}`,
  `negotiations:message:{create,delete}`, `negotiations:offer:{create,delete}` and
  `negotiations:agreement:{create,edit,delete}`.

## Setup

`NegotiationAgentModule::compose(config, root, bus, ports)` builds `AppContext` and groups two
modules:

| Module | Mounts |
|---|---|
| `DspModule` | `/dsp/current/negotiations` (DSP, RPC and BFF RPC) |
| `NegotiationAdminModule` | `{api}/negotiation-agent` and the gRPC services |

It has no seeders; in TCK mode `DspModule` runs the TCK scenario worker. Standalone, `NegotiationAgentBoot` composes it with remote ports
and validates tokens with `OAuthModule::validator`:

```
cargo run -p negotiation_agent -- setup -e <config.yaml>   # vault secrets and migrations
cargo run -p negotiation_agent -- start -e <config.yaml>   # seeders, then serve
```

Its config section is `contracts`: `common`, `ssi_auth`, `catalog` and `is_catalog_datahub`.

## HTTP API

Under `{api}/negotiation-agent`, bearer required. Reads are limited to the caller's tenant;
lists are paged with `limit` and `cursor`; `/batch` takes `{ "ids": [...] }`.

| Resource | Routes |
|---|---|
| `/negotiation-processes` | `GET /`, `POST /`, `POST /batch`, `GET\|PUT\|DELETE /{id}`, `GET /{id}/key/{key_id}` (by one of its identifiers) |
| `/negotiation-messages` | `GET /`, `POST /`, `POST /batch`, `GET\|DELETE /{id}`, `GET /process/{process_id}` |
| `/offers` | `GET /`, `POST /`, `POST /batch`, `GET\|DELETE /{id}`, `GET /process/{process_id}`, `GET /offer-id/{offer_id}` |
| `/agreements` | `GET /`, `POST /`, `POST /batch`, `GET\|PUT\|DELETE /{id}`, `GET /process/{process_id}`, `GET /assignee/{assignee}`, `GET /assigner/{assigner}` |

The gRPC services mirror the four resources, with the bearer token in the metadata.

## Tests

```
cargo test -p negotiation_agent
```

`tests/services` covers tenant isolation and roles in the four services, the DSP persistence and
the TCK script. `tests/entities` covers the filters. `tests/protocols` covers message types, the
state machine, RDF extraction, the DSP and RPC stages and their rules. `tests/adapters` covers
each gRPC service and the DSP routes end to end over an in-memory store. No database is needed.
The DSP flow between two agents runs in `crates/monolith/tests/dsp/`.
