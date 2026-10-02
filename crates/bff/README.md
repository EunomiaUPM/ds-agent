# bff

Gateway for the admin UI. It serves the admin single-page app embedded in the binary, and
forwards the app's API calls to the agent that owns each path. It also offers two browser
helpers that fetch public documents of remote connectors, which CORS would otherwise block.

The gateway does not authenticate proxied calls itself: it forwards the bearer token and the
agents check it.

It runs as its own binary or as a module of the `monolith`. It owns no tables.

## Structure

```
src/
├── lib.rs           Re-exports GatewayHttpRouter, HttpProxyDispatcher, BffModule, GatewayBoot
├── main.rs          AgentCli::<GatewayBoot>
├── gateway/
│   ├── router.rs        GatewayHttpRouter: API routes, proxy routes, SPA fallback
│   ├── frontend.rs      Embedded SPA and its runtime config
│   └── discovery.rs     DID document and federated catalog of a remote connector
├── proxy/
│   └── dispatcher.rs    HttpProxyDispatcher: path prefix to agent, streaming forward
├── setup/           BffModule, AppContext, GatewayBoot
└── static/admin/dist/   Built copy of `gui/admin`, embedded with rust-embed
tests/
├── adapters/        Auth middleware, proxy against a local upstream, the composed module
└── support/         Fixtures and a stub validator
```

## Routes

`BffModule` mounts the gateway under `/admin`. Inside it, the API is nested at both `/api` and
`/admin/api`, so it answers at `/admin/api/...` and at `/admin/admin/api/...`. Any other path
under `/admin` serves the SPA, falling back to `index.html` for client-side routes.

| Path under `/admin/api` | Effect |
|---|---|
| `GET /fe-config` | `{ "gateway_base": <http host> }`, the SPA's runtime config |
| `GET /did-json/{url}` | The remote connector's `{url}/api/v1/wallet/did.json`; bearer required |
| `GET /federated-catalog/{url}` | The remote connector's `{url}/.well-known/federated-catalog`; bearer required |
| `ANY /dsp/current/{catalogs\|negotiations\|transfers}/{*path}` | The owning agent's DSP endpoints |
| `ANY /well-known/rpc/{*path}` | `{own host}/rpc/.well-known/{path}` |
| `ANY /{prefix}[/{*path}]` | The agent API in the table below |

| Prefix | Agent | Upstream path |
|---|---|---|
| `catalogs`, `datasets`, `data-services`, `distributions`, `odrl-policies`, `peer-catalogs`, `dataset-offerings`, `datahub` | Catalog | `api/v1/catalog-agent/{prefix}` |
| `connector` | Catalog | `api/v1/connector` |
| `negotiations` | Negotiation | `api/v1/negotiation-agent` |
| `transfers` | Transfer | `api/v1/transfer-agent` |
| `dataplane` | Transfer | `api/v1/dataplane` |
| `mates`, `wallet`, `vc-request`, `gate`, `gaia` | Auth | `api/v1/{prefix}` |
| `peer-connection`, `onboard` | Auth | `api/v1/peer-connection` |
| `events` | Own host | `api/v1/events` |
| `oauth`, `auth` | Own host | `oauth` |
| `well-known` | Own host | `.well-known` |
| `v1` | Own host | `api/v1` |

An unknown prefix answers 404. Agent hosts come from the `catalog`, `contracts`, `transfer` and
`ssi_auth` sections of the config.

## Proxy

`HttpProxyDispatcher` streams the request body to the upstream and the response back:

- The path and query are appended to the upstream path.
- Hop-by-hop headers and `Host` are dropped both ways. The other headers pass as they are,
  `Authorization` included.
- `x-correlation-id` is kept, or set to a new `urn:uuid`, and returned in the response;
  each forward also gets a new `x-request-id`.
- Requests time out after 30 s, except `Accept: text/event-stream`, which stays open.
- An unreachable upstream answers 502.

CORS allows any origin, method and header.

## Setup

`BffModule::compose(config, root)` builds `AppContext`: the config, the dispatcher and the
process's token validator, which guards the two discovery helpers.

Standalone, `GatewayBoot` owns no migrations. It validates tokens with `OAuthModule::validator`
against the shared database:

```
cargo run -p bff -- start -e <config.yaml>
```

Its config section is `gateway`: `common`, `is_production`, `transfer`, `contracts`, `catalog`,
`is_catalog_datahub` and `ssi_auth`.

The embedded app is whatever sits in `src/static/admin/dist` at build time. After changing
`gui/admin`, build it and copy its `dist` there.

## Tests

```
cargo test -p bff
```

`tests/adapters` runs the HTTP plane on local sockets. It checks the auth middleware with a stub
validator: no token is a 401, and both a bearer header and a PAT in the query string pass. It
checks that a gateway path reaches a local upstream standing in for the catalog agent with a
correlation id, and the module's name, mount point and frontend config. No database is needed.
