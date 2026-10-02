# keystore

Versioned parameters and secrets for connectors and services. A parameter is a non-secret JSON
setting; a secret is a credential that is never printed or returned in clear. Both are stored
per tenant under path-like keys such as `/connectors/http/token`, and every update must name the
version it replaces.

The dataplane reads the stores in-process to resolve the credentials of a transfer. The same
stores are served over HTTP for administration, together with a read-only view of the
application config.

## Structure

```
src/
├── lib.rs           Re-exports the stores, entries, keys and migrations
├── entities/        Key, KeyPrefix, Entry, Metadata, Version, SecretValue, commands, PrefixFilter
├── services/
│   ├── parameters/  ParameterStore port and implementation
│   ├── secrets/     SecretStore port and implementation
│   └── config/      ConfigStore: read-only application config, admins only
├── data/
│   ├── repo/        Repository traits and their error enums
│   ├── sea_orm/     Tables, migrations and SeaORM repositories
│   ├── vault/       VaultSecretRepo: secret values in Vault, metadata in the database
│   └── config/      ConfigPassthroughRepo over the loaded ApplicationConfig
├── http/            Admin routers for parameters, secrets and config
└── setup/           KeystoreModule and AppContext
tests/
├── entities/        Key validation, versions, prefix filters
├── services/        Stores with mocked repositories: tenant isolation and roles
└── support/         Fixtures
```

## Integration

`lib.rs` exports [`ParameterStore`](src/services/parameters/mod.rs),
[`SecretStore`](src/services/secrets/mod.rs), `Entry`, `SecretEntry`, `Key`, `KeyPrefix`,
`SecretValue`, `KeystoreModule` and `get_keystore_migrations`.

Every store method takes the caller's `AccessScope`. Reads are limited to the caller's tenant;
only an admin may create in or list another tenant (`tenant_id` in the command or filter).
Writes need a role that can write. `read` of a key from another tenant answers 404, not 403.

`SecretValue` prints and serializes as `*****`. Only `expose()` gives the content, so a secret
can reach logs or responses only if someone calls it on purpose.

Updates are optimistic: `EditParameterCommand` and `EditSecretCommand` carry `expectedVersion`,
and the update fails when it is not the stored version. Versions start at 1.

With an event bus, the stores publish `keystore:parameter:{create,update,delete}` and
`keystore:secret:{create,update,delete}`. Secret events carry the entry with its value masked.

## Setup

`KeystoreModule` implements `ServiceModuleTrait`:

- `KeystoreModule::compose(config, root, bus)` builds the stores and the config passthrough on
  the shared `RootContext`, and mounts the routes under `{api}/keystore` (`/api/v1/keystore`
  by default).
- `KeystoreModule::build_stores(root, bus)` returns only the two stores, for crates that read
  the keystore without serving it. The dataplane uses it.
- `KeystoreModule::migrations()` lists the `keystore_parameters` and `keystore_secrets` tables,
  so the hosting agent's migrator can run them.

Where secret values live depends on the Vault the root context was built with:

| Vault | Values | Metadata |
|---|---|---|
| Real | Vault, at `{tenant}/{key}` under field `value` | Database; the `value` column holds `"vault"` |
| Fake | Database | Database |

On update the database row is written first, so a version conflict fails before Vault is
touched.

The keystore has no binary. Today the `monolith` mounts it and the `dataplane` reads it.

## HTTP API

All routes sit under `{api}/keystore` and require a bearer token. `{key}` is the key without
its leading `/`: `/connectors/http/token` is `.../secrets/connectors/http/token`.

| Method | Path | Body | Response |
|---|---|---|---|
| GET | `/parameters` | | 200, parameters under `?prefix=`; admins may add `?tenant_id=` |
| POST | `/parameters` | `{ key, value, description?, tenant_id? }` | 201, the parameter |
| GET | `/parameters/{key}` | | 200, the parameter; 404 if missing |
| PUT | `/parameters/{key}` | `{ value, expectedVersion, description? }` | 200, `{ version }` |
| DELETE | `/parameters/{key}` | | 204 |
| GET | `/secrets` | | 200, secrets under `?prefix=`, values masked; admins may add `?tenant_id=` |
| POST | `/secrets` | `{ key, value, description?, tenant_id? }` | 201, the secret, value masked |
| GET | `/secrets/{key}` | | 200, the secret, value masked; 404 if missing |
| PUT | `/secrets/{key}` | `{ value, expectedVersion, description? }` | 200, `{ version }` |
| DELETE | `/secrets/{key}` | | 204 |
| GET | `/config` | | 200, the loaded `ApplicationConfig`; admins only |

The API never returns a secret value in clear. Services that need it use `SecretStore`
in-process.

Keys must start with `/`, have no empty segments or trailing `/`, and use only
`a-z A-Z 0-9 _ - .`. An invalid key answers 400.

## Tests

```
cargo test -p keystore
```

`tests/entities` covers key validation, versions and prefix filters. `tests/services` covers the
three stores with mocked repositories: tenant isolation, which roles may write, and admin access
to other tenants. No database or Vault is needed. The SeaORM and Vault repositories have no
tests yet.
