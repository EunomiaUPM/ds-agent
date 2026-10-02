# connector

Templates and instances that describe how the data plane reaches a back-end system.

A **template** is a reusable blueprint: how to authenticate, whether data is pulled or pushed,
over which protocol, and which parameters it needs. Its fields carry `{{__NAME__}}`
placeholders. An **instance** binds a template to one catalog distribution and fills those
placeholders with concrete values. The dataplane reads the instance of a distribution when a
transfer starts, and resolves the last placeholders (runtime values and keystore entries) at
that point.

The crate has no binary. The catalog agent mounts it, because every instance belongs to one of
its distributions. [`DESIGN.md`](DESIGN.md) is the plan for the next version of the DSL and the
WASM plugin architecture. This README describes what the code does today.

## Structure

```
src/
├── lib.rs              Re-exports the DTOs, facades, module and migrations
├── entities/
│   ├── connector_template/  ConnectorTemplateDto and ConnectorMetadata
│   ├── connector_instance/  ConnectorInstantiationDto (request) and ConnectorInstanceDto (resolved)
│   ├── auth_config/         AuthenticationConfig: none, basic, bearer, API key, OAuth 2.0
│   ├── interaction/         InteractionConfig: PULL (dataAccess) or PUSH (subscribe/unsubscribe)
│   ├── resource/            ProtocolSpec: HTTP or Kafka
│   ├── common/              SecretString and SecretSource
│   ├── parameters/          Placeholder extraction, validation and resolution
│   └── filters.rs           Template and instance filters
├── services/           Template and instance use cases
├── facades/
│   ├── connector_instance_facade/  Instances for other agents: local and remote (HTTP)
│   └── catalog_facade/             Port to the hosting catalog, to check distributions
├── data/               SeaORM entities, migrations and repositories
├── http/               Template and instance routers
└── setup/              ConnectorModule, ConnectorPorts and AppContext
tests/
├── entities/           Parameter extraction, validation, resolution and filters
├── services/           Template and instance services with mocked repositories
└── adapters/           Local and HTTP instance facades answer alike
```

## Templates

```json
{
  "name": "orders-api",
  "version": "1.0.0",
  "authentication": {
    "type": "BEARER_TOKEN",
    "token": { "type": "PLAIN", "content": "{{__RUNTIME_SECRET_{/orders/token}__}}" }
  },
  "interaction": {
    "mode": "PULL",
    "dataAccess": {
      "protocol": "HTTP",
      "urlTemplate": "{{__ACCESS_URL__}}",
      "method": "GET",
      "headers": {},
      "bodyTemplate": ""
    }
  },
  "parameters": [
    { "name": "ACCESS_URL", "title": "Back-end URL", "paramType": "STRING", "required": true }
  ]
}
```

More examples are in [`static/connector_blueprints`](../../static/connector_blueprints).

Placeholders, by when they are resolved:

| Placeholder | Filled from | When |
|---|---|---|
| `{{__NAME__}}` | The instance's `parameters`, or the declared `defaultValue` | On instantiation |
| `{{__SYS_URN__}}`, `{{__SYS_TOKEN__}}`, `{{__SYS_TIMESTAMP__}}`, `{{__SYS_ISO8601__}}` | Generated | On instantiation |
| `{{__SYS_OWN_URL__}}`, `{{__SYS_OWN_URL_DOCKER__}}` | The catalog agent's HTTP host | On instantiation |
| `{{__RUNTIME_JSON_{path}__}}` | jq `path` over the push subscribe response | At transfer time |
| `{{__RUNTIME_PARAMETER_{/key}__}}`, `{{__RUNTIME_SECRET_{/key}__}}` | Keystore of the instance's tenant | At transfer time |
| `{{__RUNTIME_INGRESS__}}` | The data plane's ingress URL | At transfer time |

When a template is created, every `{{__NAME__}}` it uses must be declared in `parameters` and
every declaration must be used, with a type that fits where it appears (`STRING`, `INT`,
`BOOLEAN`, `VEC<STRING>`, `MAP<STRING,STRING>`). Secret fields (`SecretString`) can come
`PLAIN`, `BASE64`, from `VAULT_REF` or from `ENV_VAR`, and their content can hold placeholders
too.

## Instances

`POST /instances` takes a `ConnectorInstantiationDto`:

```json
{
  "templateName": "orders-api",
  "templateVersion": "1.0.0",
  "distributionId": "urn:uuid:…",
  "parameters": { "ACCESS_URL": "https://backend.example/orders" },
  "dryRun": false
}
```

The service looks the template up in the caller's tenant, then in `system`; checks with the
catalog that the distribution exists in the tenant; validates the parameters against the
template's declarations (presence, type, unknown keys); adds system parameters and defaults;
and resolves the instance-time placeholders. With `dryRun` it returns the resolved instance
without storing it. Otherwise it stores it and links the distribution to it, replacing any
previous link.

`RuntimeParametersResolver` fills the transfer-time placeholders. The connector does not depend
on the keystore: the dataplane implements `KeystoreLookup` and passes it in.

## Integration

- `ConnectorModule::compose(config, root, bus, ports)` builds the services. `ConnectorPorts`
  carries the catalog facade, always in-process.
- `ConnectorModule::local_connector_instances()` serves instances in-process to agents that
  share the catalog's process (the monolith).
- `ConnectorInstanceRemoteFacade` reads them from the catalog agent's HTTP API with the service
  token. The dataplane and the transfer agent use it when they run apart. A 404 is reported as
  `None`, the same as the local facade.
- With an event bus, the services publish `connector:template:{create,delete}` and
  `connector:instance:{create,delete}`.

## Setup

`ConnectorModule` implements `ServiceModuleTrait` and mounts its routes under
`{api}/connector` (`/api/v1/connector` by default). `ConnectorModule::migrations()` creates the
`connector_templates`, `connector_instances` and distribution relation tables. The catalog
agent registers both.

## HTTP API

Every route requires a bearer token. Reads are limited to the caller's tenant; an admin may
create in another tenant with `tenant_id` or filter by it.

| Method | Path | Response |
|---|---|---|
| GET | `/templates` | 200, page of templates. Filters: `name`, `author`, `version`, `created_after`, `created_before`; `limit`, `cursor` |
| POST | `/templates` | 200, the stored template; 400 if placeholders and declarations disagree |
| GET | `/templates/{name}` | 200, every version of the template |
| GET | `/templates/{name}/{version}` | 200, the template; 404 if missing |
| DELETE | `/templates/{name}/{version}` | 202 |
| POST | `/instances` | 200, the resolved instance |
| GET | `/instances/{id}` | 200, the instance; 404 if missing |
| GET | `/instances/distribution/{distribution_id}` | 200, the instance linked to the distribution; 404 if none |
| DELETE | `/instances/{id}` | 202 |

## Tests

```
cargo test -p connector
```

`tests/entities` covers placeholder extraction per field type and protocol, template and
instance validation, the instance parameter map (defaults, system parameters, precedence) and
both resolvers, with jq and a fake keystore lookup. `tests/services` covers both services with
mocked repositories and catalog facade: tenant isolation and roles. `tests/adapters` checks
that the local and HTTP instance facades return the same, behind the real auth middleware. No
database is needed.
