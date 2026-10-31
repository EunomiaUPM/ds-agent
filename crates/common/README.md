# common

What the ds-agent crates share beyond ymir: the YAML config and one typed section per agent,
the DSP and JSON-LD building blocks, the facades between agents, linked data, validation and
the well-known surface.

It has no binary and no tables. Boot, module composition, errors, paging, ownership, telemetry,
caches and the gRPC helpers live in ymir, shared with heimdall; this crate does not repeat them.

Each module documents its own API in its `mod.rs`, with examples from the most common case to
the most advanced. This README only maps them:

```
cargo doc -p common --no-deps --open
```

## Modules

| Area | Module | What it gives |
|---|---|---|
| Config | `config` | `ApplicationConfig`, one typed section per agent and `GlobalConfigLoader` |
| | `info_banner` | The DS-Agent ASCII banner logged at start |
| Agents and DSP | `facades` | Ports to the auth agent (participants, peer token verification) and their HTTP adapters |
| | `well_known` | `/.well-known/dspace-version`, the RPC that reads it from a peer, and `WellKnownModule` |
| | `routes` | Paths of every agent's API |
| | `dsp_common` | DSP 2025-1 pieces shared by the agents: context, data address, ODRL, actors, namespace normalizer, rules |
| | `tck` | The connector's own side of each DSP TCK scenario |
| Linked data | `rdf` | JSON-LD expansion, RDFC-1.0 canonicalization and typed extraction (`FromRdf`) |
| | `eunomia` | JSON-LD context of the agents' own RPC messages |
| | `validation` | Rules, stages and the failures they report, independent of any protocol |
| | `flow` | One skeleton for every controller: extract, check, load, apply, commit, respond |
| HTTP and tests | `middleware` | Debug middlewares that log the raw request body |
| | `test_utils` | Fixtures for other crates' tests: a `TransferConfig`, `TestScopes`, gRPC auth stubs |

From ymir: `ymir::boot` and `ymir::module_loader` (start-up and composition),
`ymir::config::types::CommonConfig`, `ymir::errors`, `ymir::paginated_spec` and `ymir::query`,
`ymir::oauth` (users, validators, `Owner`, `OwnerScope`, `GrpcAuth`), `ymir::grpc`,
`ymir::cache`, `ymir::telemetry`, `ymir::http_tracing`, `ymir::secret`, `ymir::registry`,
`ymir::json_schema`, `ymir::utils` and the `urn_id!`, `str_id!` and `impl_owned!` macros.

## How an agent uses it

An agent binary's `main` is one line: `AgentCli::<XxxBoot>::run(SERVICE_NAME, SERVICE_BIG_NAME)`.
`XxxBoot` implements ymir's `BootstrapServiceTrait` with its config type, its migrations, the
DS-Agent banner and `compose`, which returns a `ServiceComposer` with its modules and a
`WellKnownModule`. ymir's `Bootstrapper` serves the HTTP and gRPC planes, runs every worker, and
stops everything on SIGINT or SIGTERM.

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
