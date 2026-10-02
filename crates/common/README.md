# common

Shared library of every agent. It defines how a process boots and puts its modules together,
and the pieces they all rely on: configuration, auth, errors, paging, validation, caches,
gRPC helpers, telemetry, and the RDF and Dataspace Protocol building blocks.

It has no binary and no tables. Every other crate in the workspace depends on it. It builds on
`ymir` (HTTP client, errors, vault, shared repositories) and does not repeat what ymir already
provides.

Each module documents its own API in its `mod.rs`, with examples from the most common case to
the most advanced. This README only maps them:

```
cargo doc -p common --no-deps --open
```

## Modules

| Area | Module | What it gives |
|---|---|---|
| Boot | `boot` | `AgentCli` (`setup`, `start`), `BootstrapServiceTrait`, `Bootstrapper`, `BootSeeder`, `BackgroundWorker`, shutdown on SIGINT/SIGTERM |
| | `module_loader` | `ServiceModuleTrait`, `ModuleGroup`, `ServiceComposer`, `RootContext` (database, vault, validator, service client) |
| | `config` | `ApplicationConfig` and one typed section per agent, loaded from the YAML config |
| | `telemetry` | Logs, plus OTLP traces and metrics when `OTEL_EXPORTER_OTLP_ENDPOINT` is set |
| | `info_banner` | The ASCII banner logged at start |
| HTTP and gRPC | `http_tracing` | Server spans with W3C trace context and a request duration histogram per plane |
| | `http_global_404` | Fallback for unknown routes |
| | `middleware` | Debug middlewares that log the raw request body |
| | `grpc` | Status mapping (`IntoStatus`), field parsing, paging, JSON `Struct` conversion |
| | `well_known` | `/.well-known/dspace-version` and the RPC that reads it from a peer |
| Security | `auth` | `Claims`, `RbacRole`, `AccessScope`, the HTTP and gRPC auth middlewares (`GrpcAuth`), `OauthTokenValidator`, `ServiceHttpClient` |
| | `facades` | Ports to the auth agent (participants, peer token verification) and their HTTP adapters |
| | `vault_utils` | Picks the real Vault client or the in-memory fake from the config |
| Errors and data | `errors` | Helpers over `ymir::errors` |
| | `paginated_spec` | `Page`, `Paginated`, `Sort`, cursors, and the SeaORM paging extension |
| | `query` | `QuerySpec`: a filter, a page and a sort read from one query string |
| | `validation` | Rules, stages and the failures they report, independent of any protocol |
| | `cache` | Entity cache traits, their Redis implementation, and `NoopCache` |
| | `batch_requests` | The `{ "ids": [...] }` body of batch lookups |
| Linked data | `rdf` | JSON-LD expansion, RDFC-1.0 canonicalization and typed extraction (`FromRdf`) |
| | `dsp_common` | DSP 2025-1 pieces shared by the agents: context, data address, ODRL, actors, namespace normalizer, rules |
| Utilities | `utils` | URN and URL parsing, YAML and JSON helpers |
| | `serde_utils` | Base64 and hex serde helpers |
| | `id_mac` | Macros for URN and string newtype identifiers |
| | `test_utils` | Fixtures for other crates' tests: a `TransferConfig`, `TestScopes`, gRPC auth stubs |

## How an agent uses it

An agent binary's `main` is one line: `AgentCli::<XxxBoot>::run(SERVICE_NAME, SERVICE_BIG_NAME)`.
`XxxBoot` implements `BootstrapServiceTrait`: its config type, its migrations, its token
validator, and `compose`, which returns a `ServiceComposer` with its modules. Each module
implements `ServiceModuleTrait` and can expose HTTP routes, gRPC services, migrations, workers
and seeders. `Bootstrapper` serves the HTTP and gRPC planes, runs every worker, and stops
everything on SIGINT or SIGTERM. The `boot` and `module_loader` guides show it step by step.

## Tests

```
cargo test -p common
```

Each module with tests has a `tests.rs` declared last in its `mod.rs`; `rdf` keeps them in
`rdf/tests/`. They need no infrastructure.

`tests/otlp_export_tests.rs` is the only separate target, because it touches global telemetry
state. It is `#[ignore]` and needs the dev observability stack:

```
docker compose -f deployment/dev/docker-compose.observability.yaml up -d
cargo test -p common --test otlp_export_tests -- --ignored
```
