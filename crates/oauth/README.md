# oauth

OAuth 2.0 authorization server for the agents. It keeps users, OAuth clients and personal
access tokens (PATs), and issues the bearer tokens that every agent's auth middleware accepts.
Calls between services use the client credentials grant with the service client it seeds at
boot.

A user is a tenant: its `tenant_id` is both its identifier and the `sub` of its tokens. Clients
and PATs belong to a tenant and act with its role (`Admin`, `Owner` or `Reader`).

The crate has no binary. The `monolith` and the standalone `transfer-agent` serve it; every
other agent only validates its tokens.

## Structure

```
src/
├── lib.rs           Re-exports the migrations; EVENT_PREFIX
├── config.rs        OAuthConfig: signing secret, token lifetimes, issuer, audience
├── entities/        User, Client, PersonalAccessToken, AuthCode, RefreshToken, OAuthError, commands, filters
├── services/
│   ├── token_service/   Grants, revocation, introspection; JWT claims in jwt.rs
│   ├── user_service/    Users
│   ├── client_service/  OAuth clients
│   ├── pat_service/     Personal access tokens
│   ├── password.rs      Argon2 hashing of passwords and client secrets
│   └── admin_seeder.rs  Admin user and service client seeding
├── data/
│   ├── repositories/    Repository traits and their error enums
│   └── sea_orm/         Tables, migrations and SeaORM repositories
├── http/            Token, users, clients and PATs routers; form/JSON payload extractor
└── setup/           OAuthModule, AppContext and AdminSeeder
tests/
├── entities/        Filters
└── services/        Token grants, introspection, users, clients and PATs with mocked repositories
```

## Tokens

Access, refresh and ID tokens are JWTs signed with HS256 and `jwt_secret`. All three carry a
`typ` claim, and validation only accepts `typ: access` as a bearer, so a refresh or ID token
never passes as an access token.

| Token | Claims | Lifetime |
|---|---|---|
| Access | `typ`, `sub` (tenant), `role`, `iat`, `exp`, `scope?`, `client_id?` | `access_token_ttl` (1 h) |
| Refresh | `typ`, `sub`, `role`, `jti`, `iat`, `exp`; the `jti` is stored in `oauth_tokens` | `refresh_token_ttl` (30 d) |
| ID | `typ`, `iss`, `sub`, `aud`, `email`, `role`, `iat`, `exp` and the user's extra fields | `access_token_ttl` |
| PAT | Opaque `pat_…` string; only its SHA-256 is stored | `expires_at`, or none |

A refresh token is single use: refreshing revokes it and returns a new one. A PAT is returned in
clear once, when it is created; afterwards only its first ten characters are shown.

Grants accepted by `POST /token`. Without `grant_type` the endpoint infers it from the fields
present (`code`, `assertion`, `refresh_token`, `username`).

| Grant | Input | Returns |
|---|---|---|
| `password` | `username` (email or tenant id), `password`, `scope?` | Access, refresh and ID tokens |
| `client_credentials` | Basic auth or `client_id` + `client_secret` in the body, `scope?` | Access token |
| `authorization_code` | `code`, `code_verifier`, `redirect_uri?`, `client_id?` | Access, refresh and ID tokens |
| `refresh_token` | `refresh_token`, `scope?` | Access, refresh and ID tokens |
| `urn:ietf:params:oauth:grant-type:jwt-bearer` | `assertion`, `scope?` | Access token |

For `client_credentials`, a client without a scope list may ask for any scope; otherwise every
requested scope must be in its list, and no scope means all of them. A `client_assertion` of
type `urn:ietf:params:oauth:client-assertion-type:jwt-bearer` is accepted in place of the
secret. Assertions, in both cases, are checked with the server's own `jwt_secret`, not with a
key of the client.

`GET|POST /authorize` implements the code step of PKCE without a login page: the user comes from
`user_id` or from the bearer token of the request, and the code is returned as JSON
(`{ code, state, redirect_uri }`) instead of a redirect. `code_challenge` is required;
`code_challenge_method` is `S256` (default) or `plain`. A code is valid for ten minutes, can be
exchanged once, and only by the client and redirect URI it was issued to.

## Integration

- `OAuthModule::validator(common, db)` builds the token validator each agent's boot hands to
  `common::boot`. It checks JWTs with `jwt_secret` and looks PATs up by hash in `db`, so every
  agent must share the secret with the server that issued the token.
- `TokenService` implements `common::auth::OauthTokenValidator`; the `/oauth` routes behind
  auth use it directly.
- Other agents call each other through `common::auth::service_client::ServiceHttpClient`, which
  gets and caches a client credentials token for `service_client` (`token_url` defaults to
  `{own http host}/oauth/token`).
- With an event bus, the services publish `oauth:user:{create,edit,delete}`,
  `oauth:client:{create,delete}` and `oauth:pat:{create,delete}`.

## Setup

`OAuthModule` implements `ServiceModuleTrait`:

- `OAuthModule::compose(common, root, bus)` builds `AppContext` on the shared `RootContext` and
  mounts every route under `/oauth`. The prefix is fixed; it does not use `{api}`.
- `OAuthModule::migrations()` creates `oauth_users`, `oauth_tokens`, `oauth_clients`,
  `oauth_auth_codes` and `oauth_pats`.
- `AdminSeeder` runs in `BootPhase::BeforeServe`, writing straight to the database, so it runs
  before any seeder that calls the API as these clients. It creates, if missing:
  - the admin user from `admin_seed` (`admin` / `admin@admin.local` / `admin` by default);
  - the client `eunomia-admin-gui` for the admin GUI, in tenant `admin`;
  - the service client from `service_client` (`eunomia-services` by default), with role `Admin`
    in the admin's tenant, so facades may read records of any tenant.

Settings come from `CommonConfig`:

| Field | Default | Use |
|---|---|---|
| `jwt_secret` | empty | HS256 key of every token; set it in any real deployment |
| `access_token_ttl` | `3600` | Access and ID token lifetime, in seconds |
| `refresh_token_ttl` | `2592000` | Refresh token lifetime, in seconds |
| `admin_seed` | see above | Seeded admin user |
| `service_client` | see above | Seeded service client and the token URL other agents use |

The issuer is the agent's HTTP host; the ID token audience is `client`.

## HTTP API

All routes sit under `/oauth`. Requests to the token endpoints may be form-encoded or JSON.

### Token endpoints (no bearer required)

| Method | Path | Response |
|---|---|---|
| GET, POST | `/authorize` | 200, `{ code, state, redirect_uri }`; 400 on a bad request |
| POST | `/token` | 200, `{ access_token, token_type, expires_in, refresh_token?, id_token?, scope? }`; OAuth error otherwise |
| POST | `/refresh` | Same as the refresh grant, with a body of only `refresh_token` |
| POST | `/revoke` | 200 always, whether the token existed or not (RFC 7009) |
| POST | `/introspect` | 200, `{ active, sub?, role?, scope?, client_id?, token_type?, exp?, iat? }`; `{ active: false }` for unknown tokens |
| GET | `/userinfo` | 200, the caller's user; also served at `/user-info` and `/user_info` |
| GET | `/.well-known/openid-configuration` | 200, discovery document |

Errors from these endpoints use the RFC 6749 §5.2 body (`invalid_request`, `invalid_client`,
`invalid_grant`, `invalid_scope`, `unsupported_grant_type`): 401 with `WWW-Authenticate: Basic`
for `invalid_client`, 400 for the rest.

### Management (bearer required)

Lists are paged with `limit` and `cursor` and filter by `tenant_id` (admins only), `role`,
`created_after` and `created_before`; users also by `email`, clients and PATs by `search`, PATs
by `status`.

| Method | Path | Who | Response |
|---|---|---|---|
| GET | `/users` | Any; non-admins only see their tenant | 200, page of users |
| POST | `/users` | Admin | 201, the user |
| GET | `/users/{tenant_id}` | The user itself or an admin | 200, the user |
| PATCH | `/users/{tenant_id}` | The user itself if `Owner`, or an admin; only admins change `role` | 200, the user |
| DELETE | `/users/{tenant_id}` | Admin | 204 |
| GET | `/clients` | Any; non-admins only see their tenant | 200, page of clients |
| POST | `/clients` | Owner or admin; only admins choose `tenantId` | 201, the client, without its secret |
| GET | `/clients/{client_id}` | Same tenant or admin | 200, the client; 404 otherwise |
| DELETE | `/clients/{client_id}` | Owner or admin of its tenant | 204 |
| GET | `/pats` | Any; non-admins only see their tenant | 200, page of PATs, without the token |
| POST | `/pats` | Owner or admin | 201, the PAT with its raw `token` |
| DELETE | `/pats/{id}` | Owner or admin of its tenant | 204 |

A PAT is created with the caller's role. Bodies are camelCase: `CreateUserCommand`
(`tenantId`, `email`, `password`, `role`, `extraFields`), `CreateClientCommand` (`clientId`,
`clientSecret`, `clientName`, `role`, `scopes`, `tenantId?`), `CreatePatCommand` (`name`,
`scopes`, `expiresAt?`).

## Tests

```
cargo test -p oauth
```

`tests/services` covers every grant (password, client credentials and its scope rules, JWT
bearer, PKCE codes with S256 and plain, refresh rotation), validation and introspection of
access, refresh, ID tokens and PATs, revocation, and tenant isolation and roles in the users,
clients and PATs services. `tests/entities` covers the list filters. Repositories are mocked;
no database is needed. The SeaORM repositories and the HTTP routers have no tests yet.
