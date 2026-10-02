# monolith

Every agent in one process: catalog, negotiation, transfer with its dataplane, SSI auth,
OAuth, events, keystore and the admin gateway. They share one root context (database, vault,
token validator) and one event bus, and reach each other through in-process facades instead of
HTTP. It is the `monolith` binary and the Docker image built by `deployment/Dockerfile`.

## Structure

```
src/
├── lib.rs           SERVICE_NAME (`agent`)
├── main.rs          AgentCli::<CoreBoot>
└── setup/
    ├── composition.rs   MonolithModule: builds and groups every module
    └── boot.rs          CoreBoot: config, migrations, validator, infrastructure seeders
tests/
├── dsp/             Catalog and negotiation over DSP between two in-process participants
├── cache/           Catalog agent Redis caches against a real Redis
├── auth/            SSI onboarding against the running dev stack
└── support/         Participants, throwaway databases and stubbed peer authentication
```

## Composition

`MonolithModule::compose` builds providers before their consumers, so each port resolves to a
local adapter:

1. `EventsModule`, whose bus every other module gets.
2. `AuthModule`, whose `local_ports()` serve participants and peer tokens to the rest.
3. `CatalogAgentModule`, with the local auth ports.
4. `NegotiationAgentModule`, with the auth ports and the catalog's ODRL offer service.
5. `TransferAgentModule`, with the auth ports, the negotiation's agreements, the catalog's
   datasets, distributions and connector instances, and the admin tenant for cross-tenant
   lookups.
6. `OAuthModule`, `BffModule` and `KeystoreModule`.

All of them are registered in one `ModuleGroup`, which merges their routes, gRPC services,
workers and seeders. The HTTP plane then serves:

| Path | Module |
|---|---|
| `/dsp/current/{catalog,negotiations,transfers}` | Catalog, negotiation and transfer DSP endpoints |
| `{api}/catalog-agent`, `{api}/connector` | Catalog management and connectors |
| `{api}/negotiation-agent` | Negotiation management |
| `{api}/transfer-agent`, `{api}/transfer-agent-ref`, `{api}/transfer-agent/dataplane` | Transfer and dataplane management |
| `/dataplane/proxy` | Data proxy |
| `{api}/mates`, `{api}/wallet`, `{api}/gate`, `{api}/vc-request`, `{api}/peer-connection`, `{api}/verifier`, `{api}/gaia` | SSI auth |
| `/oauth` | OAuth server |
| `/api/v1/events` | Event bus (fixed prefix) |
| `{api}/keystore` | Keystore |
| `/admin` | Admin gateway and SPA |

`{api}` is the configured API prefix, `/api/v1` by default. `common::boot` adds the well-known,
health and fallback routes.

## Boot

`CoreBoot` reads the whole `ApplicationConfig` (`monolith`, `transfer`, `contracts`, `catalog`,
`ssi_auth`, `gateway`) and records its migrations in `seaql_ds_agent_migrations`.

- `migrations()` lists every module's tables in foreign-key order: catalog (with connector),
  negotiation, events, auth, OAuth, transfer (with dataplane), keystore.
- Before composing, it flushes Redis when the cache is Redis, so no stale entry outlives a
  restart, and seeds the OAuth admin user and service client.
- After composing, it links the connector's own wallet as a participant, then runs every
  module's seeders: the admin tenant's catalog and the policy templates.

```
cargo run -p monolith -- setup -e <config.yaml>   # vault secrets and migrations
cargo run -p monolith -- start -e <config.yaml>   # seeders, then serve
```

## Tests

Every test here needs infrastructure and is `#[ignore]`; `cargo test --workspace` skips them.

| Target | Needs | Covers |
|---|---|---|
| `dsp` | `DATABASE_URL`, optional `REDIS_URL` | A provider and a consumer composed in-process, each over its own `test_<uuid>` database and port, with fixed peer tokens. Catalog over DSP, and a negotiation from request to `FINALIZED` |
| `cache` | `REDIS_URL` | The catalog agent's Redis caches, with keys new to each run |
| `auth` | The dev stack running: authority, consumer, provider | The consumer gets a credential from the authority and onboards with the provider over GNAP and OID4VP |

```
task test:integration   # starts the provider's dev Postgres and Redis, runs dsp and cache
task test:onboarding    # runs auth against the running dev stack
```

There is no transfer flow yet; it will join `dsp` once the transfer agent's DSP side works.
