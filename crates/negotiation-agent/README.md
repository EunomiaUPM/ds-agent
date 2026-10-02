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
├── data/            Repository traits, SQL repositories and migrations
├── http/            Management routers
├── grpc/            gRPC mirror of the management API
├── protocols/dsp/
│   ├── http/            DSP endpoints, RPC endpoints, BFF RPC endpoints
│   ├── orchestrator/    protocol/ (peer messages), rpc/ (outgoing messages), bff/ (chained steps)
│   ├── validator/       Payload, state transition and role checks
│   ├── persistence/     RPC persistence and process lookup by pid
│   ├── facades/         Well-known lookup of peer DSP addresses
│   └── protocol_types.rs  DSP messages and NegotiationProcessState
├── facades/         Catalog facade: checks that a requested offer is published
└── setup/           NegotiationAgentModule, admin module, AppContext, ports, boot
tests/
├── entities/        Filters
├── services/        Every service with mocked repositories
├── adapters/        gRPC services with mocked services and a stub validator
└── support/         Fixtures
```

## Negotiation

States: `REQUESTED`, `OFFERED`, `ACCEPTED`, `AGREED`, `VERIFIED`, `FINALIZED`, `TERMINATED`.
Every incoming message is validated against the JSON-LD payload, the process's current state and
the role the agent holds in it before anything is stored. A process is identified by its
`consumerPid` and `providerPid`.

Each step goes through one orchestrator step (`step_*.rs`). The step validates the message,
stores the process change, the message and the offer or agreement it carries, then answers
with the process ACK or an error message.

When a peer's contract request names an offer, the provider checks through the catalog facade
that the offer is published in the tenant's catalog.

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

A first request or offer answers 201 with the ACK, or 200 if that process already exists; the
rest answer 200. Errors carry the DSP error message: 404 for a missing or foreign process, so a
peer cannot probe other negotiations, and 400 otherwise. A missing or invalid peer token answers
401.

## RPC

Under the same prefix, bearer required. Each endpoint sends one DSP message to the peer on behalf
of the tenant, stores the result locally and returns `{ request, response, negotiationAgentModel }`.

| Path | Message sent |
|---|---|
| `/rpc/setup-request-init` | First contract request, to `providerAddress` |
| `/rpc/setup-request` | Counter request |
| `/rpc/setup-offer-init` | First contract offer |
| `/rpc/setup-offer` | Counter offer |
| `/rpc/setup-acceptance` | `ACCEPTED` event |
| `/rpc/setup-agreement` | Agreement |
| `/rpc/setup-verification` | Agreement verification |
| `/rpc/setup-finalization` | `FINALIZED` event |
| `/rpc/setup-termination` | Termination |
| `/tck/negotiations/requests` | Starts a negotiation from `datasetId`, `offerId`, `providerId`, `connectorAddress`, for the DSP TCK |

`/bff-rpc/setup-{request-init,offer-init,acceptance,agreement,termination}` are what the admin
GUI calls. They run the same steps, and `setup-agreement` chains agreement, verification and
finalization in one call.

## Integration

- `NegotiationAgentModule::agreement_service()` gives the agreements in-process to the transfer
  agent in the monolith; `AgreementView` and `OfferView` are exported for it.
- `NegotiationPorts::local(auth, offers)` uses the auth module and the catalog's ODRL offer
  service in-process; `NegotiationPorts::remote` calls the auth and catalog agents' APIs
  (`ssi_auth` and `catalog` in the config) with the service token.
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

It has no workers or seeders. Standalone, `NegotiationAgentBoot` composes it with remote ports
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

`tests/services` covers tenant isolation and roles in the four services. `tests/entities` covers
the filters. `tests/adapters` covers each gRPC service: auth, parsing, error mapping, nesting and
paging. No database is needed. The DSP flow between two agents runs in
`crates/monolith/tests/dsp/`; the orchestrators and validators have no crate tests.
