# catalog-agent

The catalog side of a connector. Each tenant keeps a DCAT 3 catalog of datasets, distributions
and data services, with ODRL offers attached to them. Peers read it over the Dataspace Protocol
catalog endpoints (DSP 5, DSP 6). The owner manages it through a REST API and its gRPC mirror.

The agent also hosts the `connector` crate, since every connector instance belongs to one of its
distributions. It caches the catalogs it fetches from peers, and builds ODRL offers from policy
templates.

It runs as its own binary or as a module of the `monolith`.

## Structure

```
src/
├── lib.rs           Re-exports the migrations, the repository trait and the main DTOs
├── main.rs          AgentCli::<CatalogAgentBoot>
├── entities/        Catalogs, data services, datasets, distributions, ODRL offers, policy
│                    templates and their instantiation, dataset offerings
├── services/        One use case per entity, plus peer catalogs, dataset offerings, policy
│                    instantiation and tenant provisioning (service and bus listener)
├── data/            Repository traits, SQL repositories and migrations
├── cache/           Redis or no-op cache of entities and peer catalogs
├── http/            Management routers, one per entity
├── grpc/            gRPC mirror of the management API
├── protocols/dsp/   DSP catalog endpoints, the RPC that queries peers, validators, orchestrators
├── facades/         CatalogLocalFacade: distributions for the connector, in-process
└── setup/           CatalogAgentModule, admin module, AppContext, ports, seeders, boot
proto/               catalog_models.proto, catalog_service.proto
dsp_schema/          JSON schemas of the DSP catalog messages
tests/
├── entities/        Filters, policy instantiation against a template
├── services/        Every service with mocked repositories and a no-op cache
├── adapters/        gRPC services with mocked services and a stub validator
└── support/         Builders and fixtures
```

## Catalog model

Each tenant has one **main catalog** and one **main data service**. The main data service
points at the agent's DSP endpoint. Other catalogs of the tenant appear as sub-catalogs of the
main one.

Provisioning a tenant creates both if missing and is idempotent. It runs at boot for the admin
tenant, on every `oauth:user:create` event when the process has a bus, and on demand through
`POST /tenants/{tenant_id}/provision`.

| Entity | Belongs to | Notes |
|---|---|---|
| Catalog | Tenant | `main` marks the main catalog |
| Data service | Catalog | Endpoint of a DSP connector or of a back end |
| Dataset | Catalog | |
| Distribution | Dataset | Its `accessService` is a data service; a connector instance can be linked to it |
| ODRL offer | Catalog, dataset, distribution or data service | Created directly or from a template |
| Policy template | Tenant | Versioned; its parameters are typed and validated on instantiation |

A **dataset offering** creates a dataset, its distribution and, optionally, its policy in one
call, defaulting to the tenant's main catalog and data service. If a later step fails, the
dataset is deleted again.

## DSP

Mounted at `/dsp/current/catalog`:

| Method | Path | Auth | Response |
|---|---|---|---|
| POST | `/request` | Peer token | 200, the tenant's catalog |
| GET | `/datasets/{id}` | Peer token | 200, the dataset with its offers and distributions |
| POST | `/rpc/setup-catalog-request` | Bearer | Fetches a peer's catalog and caches it |
| POST | `/rpc/setup-dataset-request` | Bearer | Fetches one dataset from a peer |

The protocol endpoints check the peer's token with the auth agent (`SSIAuthFacadeTrait`). The
peer then reads, as `Reader`, the tenant it is associated with. The catalog it gets is the
tenant's main catalog with its data service, its datasets (offers and distributions included),
and the other catalogs as sub-catalogs. Messages pass through the DSP namespace normalizer and
the schemas in `dsp_schema/`.

The RPC endpoints are how the tenant's own clients reach peers. The agent resolves the peer's
DSP address through its well-known endpoint and sends the request with the tenant's token for
that peer, if it holds one. The fetched catalog is cached per tenant and peer, unless the
request sets `noCache`.

## Integration

- `CatalogAgentModule::local_connector_instances()` gives the connector instances in-process to
  the transfer agent's dataplane in the monolith.
- `dataset_service()`, `distribution_service()` and `odrl_policy_service()` give the same
  services in-process to the negotiation and transfer agents in the monolith.
- `CatalogPorts::local(auth)` uses the auth module in-process; `CatalogPorts::remote` calls the
  auth agent's API (`ssi_auth` in the config) with the service token.
- With an event bus, the services publish `catalog:{catalog,dataservice,dataset,distribution}:{create,edit,delete}`,
  `catalog:offer:{create,delete}` and `catalog:policy_template:{create,delete}`.

## Setup

`CatalogAgentModule::compose(config, root, bus, ports)` builds `AppContext` and groups three
modules:

| Module | Mounts |
|---|---|
| `DspModule` | `/dsp/current/catalog` |
| `CatalogAdminModule` | `{api}/catalog-agent` (`/api/v1/catalog-agent`) and the gRPC services |
| `ConnectorModule` | `{api}/connector` |

- `migrations()` lists the catalog tables, then the connector's, which reference distributions.
- `workers()` returns the tenant provisioning listener, only when there is a bus.
- `seeders()` provision the admin tenant and load every `*.json` in `policy_templates_folder`
  into it. Invalid or already registered templates are skipped.

The cache is chosen by `cache.cache_type`: `Redis` at the configured URL, or `Noop`.

Standalone, `CatalogAgentBoot` composes the module with remote ports, validates tokens with
`OAuthModule::validator`, and runs with:

```
cargo run -p catalog_agent -- setup -e <config.yaml>   # vault secrets and migrations
cargo run -p catalog_agent -- start -e <config.yaml>   # seeders, then serve
```

The `catalog` section of the config holds `common`, `cache`, `policy_templates_folder`,
`ssi_auth` (where the auth agent is) and `contracts` (whose host is the DSP endpoint of the main
data service).

## HTTP API

Under `{api}/catalog-agent`, bearer required. Reads are limited to the caller's tenant; lists are
paged with `limit` and `cursor`. `/batch` takes `{ "ids": [...] }` (at most 100) and returns the
ones found.

| Resource | Routes |
|---|---|
| `/catalogs` | `GET /`, `POST /`, `GET\|POST /main`, `POST /batch`, `GET\|PUT\|DELETE /{id}` |
| `/data-services` | `GET /`, `POST /`, `GET /catalog/{id}`, `GET\|POST /main`, `POST /batch`, `GET\|PUT\|DELETE /{id}` |
| `/datasets` | `GET /`, `POST /`, `GET /catalog/{id}`, `POST /batch`, `GET\|PUT\|DELETE /{id}` |
| `/distributions` | `GET /`, `POST /`, `GET /dataset/{id}`, `GET /dataset/{id}/format/{format}`, `POST /batch`, `GET\|PUT\|DELETE /{id}` |
| `/odrl-policies` | `GET /`, `POST /`, `GET\|DELETE /entity/{entity_id}`, `POST /batch`, `GET\|DELETE /{id}` |
| `/policy-templates` | `GET /`, `POST /`, `POST /batch`, `GET /{id}`, `GET\|DELETE /{id}/{version}`, `POST /instantiate-odrl-offer` |
| `/dataset-offerings` | `POST /` |
| `/peer-catalogs` | `GET /` (every cached peer catalog), `GET /{peer_id}` |
| `/tenants` | `POST /{tenant_id}/provision` |

The gRPC services in `proto/catalog_service.proto` mirror catalogs, data services, datasets,
distributions, ODRL offers and policy templates, with the same bearer token in the metadata.

## Tests

```
cargo test -p catalog_agent
```

`tests/services` covers tenant isolation and roles in every entity service, peer catalogs cached
per tenant, dataset offerings with their rollback, policy instantiation, and tenant provisioning,
including the listener on a real bus. `tests/entities` covers the filters and the validation of
instantiation parameters. `tests/adapters` covers each gRPC service: auth, URN parsing, error
mapping and paging. No database or Redis is needed. The DSP endpoints run end to end in
`crates/monolith/tests/dsp/`.
