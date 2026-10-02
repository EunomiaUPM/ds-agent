# Contributing to Eunomia DS-Agent

Thanks for helping build Eunomia. This guide covers how to set up the project, how the code is
organised, and what a pull request needs before it can be merged.

## Setting up

- Rust (stable), [Task](https://taskfile.dev) and Docker with compose.
- [`ymir`](https://github.com/EunomiaUPM/ymir) checked out next to this repository (`../ymir`):
  the workspace depends on it by path.
- Node.js, only if you work on the admin console in `gui/`.

`cargo test --workspace` must pass on a clean checkout, without Postgres or Redis. To run the
agents themselves, see [Getting started](README.md#getting-started).

## Workflow

1. Branch off `main`. Name the branch after the kind of change: `feat/…`, `fix/…`,
   `refactor/…`, `docs/…`.
2. Keep each commit to one crate and one purpose, and write its message in the
   [Conventional Commits](https://www.conventionalcommits.org) style, scoped to the crate:
   `fix(dataplane): reject proxy calls to stopped transfers`.
3. Open a pull request against `main`. Say what changed and why, and link the issue if there is
   one.

## How the code is organised

Every service crate follows the same hexagonal layout. Once you know one, you know them all.

```
crates/<crate>/
├── src/
│   ├── entities/   Domain types, commands, queries, filters and events. No database or HTTP.
│   ├── services/   Use cases: one folder per service, the port (trait) in mod.rs and the
│   │               implementation in service.rs
│   ├── data/       Repository traits, their SeaORM implementation and the migrations
│   ├── http/       Axum routers (and grpc/ for the gRPC mirror)
│   └── setup/      Wiring: AppContext, the ServiceModuleTrait module and, in binaries, the boot
└── tests/          One folder per layer
```

- **Components live in folders.** A service, router or repository gets its own directory with a
  `mod.rs` for the public trait and separate files for the implementation (`service.rs`) and
  its output views (`views.rs`).
- **No free functions.** Every function belongs to a type, as a method or an associated
  function. HTTP handlers are associated functions of their router struct. Test helpers are
  the only exception.
- **Ports are traits.** Services receive their dependencies as `Arc<dyn Trait>`, and every port
  carries `#[mockall::automock]` so it can be mocked from `tests/`.
- **Binaries are one line.** `main.rs` calls `AgentCli::<XxxBoot>::run(...)`. CLI, migrations,
  servers, workers and shutdown all live in `common::boot`. Long-running tasks implement
  `BackgroundWorker` and are returned from the module's `workers()`; never `tokio::spawn` them
  directly.
- **Database changes edit the original migrations.** There is no production data yet, so do not
  add incremental or repair migrations.
- **Paging and filtering happen in the database**, never in memory.
- **HTTP calls go through ymir's client.** Do not add `reqwest` or another client.
- **Tenants are never empty.** Every record belongs to a real tenant; only admins act across
  tenants.

`crates/common` is the shared foundation. Each of its modules explains its API in its own
`mod.rs`; start there before writing a helper that might already exist.

## Code style

Format with the repository's `rustfmt.toml` and keep `cargo clippy` clean.

Comments are written for people, in English, and kept short:

- One short `//!` at the top of each module saying what it is for.
- One `///` line per public type, trait and method. A complex method may have a few more lines
  inside it, between its parts.
- At most two lines per comment. Say what the code does and, when it is not obvious, why.
- Never repeat the name of what you document ("Creates a client" above `create_client`).
- No separator banners (`// ===== X =====`), no arrows to chain ideas, no marketing adjectives.

```rust
// No
// ---------- Token validation ----------
/// Validates the token -> checks signature -> checks expiry -> returns claims.

// Yes
/// Checks signature and expiry and returns the claims, or an auth error.
```

When code follows the Dataspace Protocol, cite the exact section of
[DSP 2025-1](https://eclipse-dataspace-protocol-base.github.io/DataspaceProtocol/2025-1/), such
as `DSP 8.1.2.3`, never a paraphrase.

## Tests

| What you test | Where it goes |
|---|---|
| A service or use case | `crates/<crate>/tests/services/<service>.rs` |
| Logic in entities (validators, resolvers, filters) | `crates/<crate>/tests/entities/<thing>.rs` |
| DSP orchestrators and state validation | `crates/<crate>/tests/protocols/<thing>.rs` |
| HTTP routers and gRPC services | `crates/<crate>/tests/adapters/http_<resource>.rs`, `grpc_<resource>.rs` |
| A module of `common` | `crates/common/src/<module>/tests.rs` |
| Anything that needs Postgres, Redis or several agents | `crates/monolith/tests/<flow>/` |

- Each layer folder is one test binary: a `main.rs` that declares one file per tested thing.
  Shared helpers go in `tests/support/`.
- Start each file with a one or two line `//!` saying what it tests and with which doubles.
  Give each test a one line `///` stating the behaviour it checks, and a name that says the
  same: `reader_cannot_create_catalog`.
- Use `mockall` for ports. A mock without expectations panics when called, which also proves
  what a use case does not touch.
- Integration tests in `monolith` are `#[ignore = "needs ..."]`. Run them with
  `task test:integration` and `task test:onboarding`.

## Documentation

- Every crate has a README with its structure, how it integrates, its API and its tests. If
  your change adds a route, a module or a test target, update it.
- Each `lib.rs` opens with two short paragraphs on what the crate does and what it exports.

## Before you open a pull request

```bash
cargo fmt --all
cargo clippy --workspace --all-targets
cargo test --workspace
cargo doc --workspace --no-deps   # no broken intra-doc links
```

If you touched a DSP flow, also run `task test:integration`.

## License

By contributing, you agree that your contributions are licensed under the
[GPL-3.0](LICENSE.md), the same license as the project.
