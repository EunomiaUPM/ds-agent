# OAuth 2.0 & 2.1 Authorization Server (`oauth`)

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Rust Edition](https://img.shields.io/badge/Rust-2021-orange.svg)](https://www.rust-lang.org/)
[![OAuth Specs](https://img.shields.io/badge/Specs-RFC%206749%20%7C%20RFC%207636%20%7C%20RFC%207523%20%7C%20RFC%207009%20%7C%20RFC%207662-blueviolet.svg)](#supported-grant-types--standards)

The **`oauth`** crate provides a modern, high-security **OAuth 2.0 / OAuth 2.1 and OpenID Connect (OIDC)** authorization server for the DS-Protocol ecosystem.

It is engineered for both **Machine-to-Machine (M2M)** autonomous agent communication and **delegated user access**, featuring **PKCE**, **RFC 7523 JWT Profile**, **Personal Access Tokens (PAT)**, **Argon2** secret hashing, **RFC 7009 token revocation**, **RFC 7662 introspection**, and seamless `OauthTokenValidator` middleware integration across all workspace crates.

---

## Table of Contents

- [Architecture Overview](#architecture-overview)
- [Supported Grant Types & Standards](#supported-grant-types--standards)
- [Core Features](#core-features)
  - [1. Machine-to-Machine (M2M) Authentication](#1-machine-to-machine-m2m-authentication)
  - [2. PKCE Authorization Code Flow (OAuth 2.1)](#2-pkce-authorization-code-flow-oauth-21)
  - [3. Personal Access Tokens (PAT)](#3-personal-access-tokens-pat)
  - [4. Token Revocation & Introspection](#4-token-revocation--introspection)
  - [5. Dual Content-Type Handling (Form & JSON)](#5-dual-content-type-handling-form--json)
- [Integration Guide for Other Crates](#integration-guide-for-other-crates)
  - [Validating Tokens in Protected Services](#validating-tokens-in-protected-services)
  - [Obtaining Tokens (Code Examples)](#obtaining-tokens-code-examples)
- [Setup & Module Composition](#setup--module-composition)
- [REST API Reference](#rest-api-reference)
- [Security Model](#security-model)
- [Testing Guide](#testing-guide)

---

## Architecture Overview

The crate follows a Clean Hexagonal Architecture separating HTTP transport, domain services, repository abstractions, and persistent storage:

```mermaid
flowchart TB
    subgraph Clients ["OAuth Clients & Callers"]
        BrowserApp["SPA / Browser / User<br/>(PKCE Flow)"]
        M2MAgent["Autonomous Data Plane Agent<br/>(Client Credentials / JWT Profile)"]
        DevCLI["Developer / Admin CLI<br/>(PAT Bearer Token)"]
    end

    subgraph HTTPTransport ["HTTP Layer (crates/oauth/src/http)"]
        TokenRouter["TokenRouter<br/>/token, /authorize, /revoke, /introspect"]
        PatsRouter["PatsRouter<br/>/pats (CRUD)"]
        ClientsRouter["ClientsRouter<br/>/clients (Admin CRUD)"]
        UsersRouter["UsersRouter<br/>/users, /userinfo"]
        Extractor["OAuthPayload&lt;T&gt;<br/>(Dual Form / JSON Extractor)"]
    end

    subgraph DomainServices ["Domain Services (crates/oauth/src/services)"]
        TokenSvc["TokenService<br/>(Issues JWTs & Validates Bearer Tokens)"]
        PatSvc["PatService<br/>(Generates & Revokes PATs)"]
        ClientSvc["ClientService<br/>(Argon2 Hashed Secrets & Scopes)"]
        UserSvc["UserService<br/>(Argon2 Password Auth)"]
    end

    subgraph Repositories ["Data Repositories (InMemory & SeaORM)"]
        ClientRepo[("ClientRepository")]
        AuthCodeRepo[("AuthCodeRepository")]
        PatRepo[("PatRepository")]
        TokenRepo[("TokenRepository (Refresh Tokens)")]
        UserRepo[("UserRepository")]
    end

    Clients --> Extractor
    Extractor --> TokenRouter & PatsRouter & ClientsRouter & UsersRouter

    TokenRouter --> TokenSvc
    PatsRouter --> PatSvc
    ClientsRouter --> ClientSvc
    UsersRouter --> UserSvc

    TokenSvc --> ClientRepo & AuthCodeRepo & PatRepo & TokenRepo & UserRepo
    PatSvc --> PatRepo
    ClientSvc --> ClientRepo
    UserSvc --> UserRepo

    subgraph MiddlewareIntegration ["Workspace Middleware"]
        TokenValidator["OauthTokenValidator Trait<br/>(common::auth::middleware)"]
    end

    TokenSvc -.->|Implements| TokenValidator
```

---

## Supported Grant Types & Standards

| Specification | Standard | Description | Endpoints |
|---|---|---|---|
| **Client Credentials** | RFC 6749 §4.4 | Direct M2M service authentication with `client_id` + `client_secret` (HTTP Basic or POST body). | `POST /token` |
| **PKCE Authorization Code** | RFC 7636 / OAuth 2.1 | Delegated user authorization requiring SHA-256 code challenge (`S256`) to protect public clients. | `GET /authorize`<br/>`POST /token` |
| **JWT Bearer Grant** | RFC 7523 §2.1 | Asymmetric/symmetric M2M assertion grant (`urn:ietf:params:oauth:grant-type:jwt-bearer`). | `POST /token` |
| **JWT Client Authentication** | RFC 7523 §2.2 | Authenticating the client via signed JWT (`client_assertion`) instead of shared secrets. | `POST /token` |
| **Refresh Token Rotation** | RFC 6749 §6 | Single-use refresh token exchange with automatic token invalidation. | `POST /token` |
| **Resource Owner Password** | RFC 6749 §4.3 | Direct user credential authentication (legacy / headless login). | `POST /token` |
| **Token Revocation** | RFC 7009 | Idempotent token revocation for access tokens, refresh tokens, and PATs. | `POST /revoke` |
| **Token Introspection** | RFC 7662 | Query token validity, issuer, client_id, scopes, user roles, and expiration. | `POST /introspect` |
| **OIDC Discovery** | RFC 8414 / OpenID Core | Metadata discovery document describing endpoints, grants, and signing algorithms. | `GET /.well-known/openid-configuration` |

---

## Core Features

### 1. Machine-to-Machine (M2M) Authentication

#### A. Client Credentials Grant (RFC 6749 §4.4)
Ideal for server-to-server agents storing a shared secret. Secret keys are verified using **Argon2id**:

```http
POST /oauth/token HTTP/1.1
Host: localhost:8080
Authorization: Basic dHJhbnNmZXItYWdlbnQtMTpjYjg0MzNhZDI...
Content-Type: application/x-www-form-urlencoded

grant_type=client_credentials&scope=transfer:read+transfer:write
```

#### B. RFC 7523 JWT Profile (Secretless M2M)
Allows federated agents or external microservices to authenticate without sharing long-lived secrets, using signed JWT assertions:

```http
POST /oauth/token HTTP/1.1
Host: localhost:8080
Content-Type: application/x-www-form-urlencoded

grant_type=urn:ietf:params:oauth:grant-type:jwt-bearer
&assertion=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
&scope=catalog:read
```

The server verifies:
- `iss` (Issuer): Registered client identifier.
- `sub` (Subject): Client identifier.
- `aud` (Audience): Matches this authorization server's issuer URL.
- `exp` (Expiration): Current UTC time within validity window.

---

### 2. PKCE Authorization Code Flow (OAuth 2.1)

```mermaid
sequenceDiagram
    autonumber
    participant Client as User Agent / Frontend
    participant OAuth as OAuth Server (/oauth)
    participant API as Protected Resource (e.g. transfer-agent)

    Note over Client: Generate code_verifier (random 32 bytes)<br/>code_challenge = BASE64URL(SHA256(verifier))
    Client->>OAuth: GET /oauth/authorize?response_type=code&client_id=web-client<br/>&code_challenge=E9Melhoa...&code_challenge_method=S256&redirect_uri=...
    OAuth-->>Client: 302 Found to redirect_uri?code=auth_code_xyz

    Client->>OAuth: POST /oauth/token<br/>grant_type=authorization_code&code=auth_code_xyz<br/>&code_verifier=dBjftJeZ...&client_id=web-client
    Note over OAuth: Verifies SHA256(verifier) == code_challenge<br/>Marks code as used (single-use protection)
    OAuth-->>Client: 200 OK { access_token: "jwt...", refresh_token: "..." }

    Client->>API: GET /api/v1/resource (Authorization: Bearer jwt...)
    API-->>Client: 200 OK
```

- Codes expire within 10 minutes and can only be redeemed once.
- Supports `code_challenge_method=S256` (recommended) and `plain`.

---

### 3. Personal Access Tokens (PAT)

Personal Access Tokens enable developer automation, CLI tooling, and long-lived system tokens without interactive OAuth flows.

```mermaid
flowchart LR
    TokenGen["Generate 24 Cryptographic Bytes<br/>Prefix: pat_..."] --> HashCalc["Compute SHA-256 Hash"]
    HashCalc --> DBStore[("Save to pat_tokens table:<br/>- token_hash (SHA-256)<br/>- token_prefix (e.g. pat_abc123)<br/>- user_id, scopes, expires_at")]
    TokenGen --> UserOutput["Return Raw Token to User Once:<br/>pat_abc123... (Never stored in plaintext)"]
```

#### Key Characteristics:
1. **Opaque & Typed**: Prefixed with `pat_` followed by URL-safe Base64.
2. **One-Way Storage**: Only the SHA-256 hash (`token_hash`) is stored in the database.
3. **Universal Validation**: `TokenService` implements `OauthTokenValidator`. When an incoming request contains `Authorization: Bearer pat_...`, it is transparently validated against `PatRepository`, updating `last_used_at` automatically. **No code changes are required in existing protected endpoints**.

---

### 4. Token Revocation & Introspection

#### Token Revocation (RFC 7009)
Revokes access tokens, refresh tokens, or PATs idempotently:
```http
POST /oauth/revoke HTTP/1.1
Content-Type: application/x-www-form-urlencoded

token=pat_4fa89b21f92e...&token_type_hint=access_token
```
Returns `200 OK` whether the token existed or was already invalid.

#### Token Introspection (RFC 7662)
Enables resource servers to query token status and metadata:
```http
POST /oauth/introspect HTTP/1.1
Content-Type: application/x-www-form-urlencoded

token=eyJhbGciOiJIUzI1Ni...
```
Response:
```json
{
  "active": true,
  "scope": "transfer:read transfer:write",
  "client_id": "transfer-agent-1",
  "sub": "user-uuid-123",
  "exp": 1757152000,
  "token_type": "Bearer",
  "roles": ["Admin", "TransferOperator"]
}
```

---

### 5. Dual Content-Type Handling (Form & JSON)

The custom extractor `OAuthPayload<T>` parses both:
- `application/x-www-form-urlencoded` (RFC 6749 standard)
- `application/json` (modern API clients)

Errors conform to RFC 6749 §5.2:
```json
{
  "error": "invalid_grant",
  "error_description": "PKCE verification failed: code_verifier does not match code_challenge"
}
```

---

## Integration Guide for Other Crates

### Validating Tokens in Protected Services

Host crates (like `transfer-agent-ref` or `monolith`) use the shared `OauthTokenValidator` trait:

```rust
use std::sync::Arc;
use common::auth::middleware::OauthTokenValidator;
use oauth::setup::OAuthSetup;

// In AppContext::build():
let oauth_validator: Arc<dyn OauthTokenValidator> =
    OAuthSetup::new().build_token_service(config.common().clone().into(), db.clone());
```

In Axum router definitions, apply the middleware:
```rust
use common::auth::middleware::require_auth;

let protected_router = Router::new()
    .route("/transfers", get(list_transfers))
    .layer(axum::middleware::from_fn_with_state(
        oauth_validator.clone(),
        require_auth,
    ));
```

The validator accepts both standard JWTs and Personal Access Tokens (`pat_...`) seamlessly.

---

### Obtaining Tokens (Code Examples)

#### Client Credentials (Rust `reqwest` Example)
```rust
let client = reqwest::Client::new();
let response = client
    .post("http://localhost:8080/oauth/token")
    .basic_auth("my-client-id", Some("my-client-secret"))
    .form(&[
        ("grant_type", "client_credentials"),
        ("scope", "transfer:read transfer:write"),
    ])
    .send()
    .await?;

let token_data: serde_json::Value = response.json().await?;
let access_token = token_data["access_token"].as_str().unwrap();
```

#### Creating a Personal Access Token via API
```rust
let response = client
    .post("http://localhost:8080/oauth/pats")
    .bearer_auth(admin_jwt)
    .json(&serde_json::json!({
        "name": "ci-automation-token",
        "scopes": ["transfer:read"],
        "expires_in_days": 90
    }))
    .send()
    .await?;

let pat_response: serde_json::Value = response.json().await?;
let raw_pat = pat_response["token"].as_str().unwrap(); // "pat_xyz..."
```

---

## Setup & Module Composition

### 1. `OAuthModule` (`ServiceModuleTrait`)

Mounts the `/oauth` router and SeaORM database migrations:

```rust
use oauth::setup::OAuthModule;

let oauth_module = OAuthModule::new(oauth_config, db_connection);

// Implements common::module_loader::service_module::ServiceModuleTrait:
// - name(): "oauth"
// - migrations(): get_oauth_migrations()
// - http(): Mounts "/oauth"
```

### 2. Standalone Router Composition

```rust
use oauth::setup::OAuthSetup;

let oauth_router = OAuthSetup::new().build_router(oauth_config, db_connection);
let app = Router::new().nest("/oauth", oauth_router);
```

---

## REST API Reference

All routes are mounted under `/oauth` (e.g. `http://localhost:8080/oauth`):

### Token & Authorization Endpoints

| Method | Endpoint | RFC / Spec | Description |
|---|---|---|---|
| `POST` | `/token` | RFC 6749 / 7523 | Issue access token (`client_credentials`, `authorization_code`, `refresh_token`, `password`, `jwt-bearer`). |
| `GET` / `POST` | `/authorize` | RFC 6749 / 7636 | PKCE authorization code request endpoint. |
| `POST` | `/revoke` | RFC 7009 | Revoke access tokens, refresh tokens, or PATs (`200 OK`). |
| `POST` | `/introspect` | RFC 7662 | Introspect active status, scopes, and claims of any token. |
| `GET` | `/userinfo` | OIDC Core | Returns authenticated user claims (requires Bearer token). |
| `GET` | `/.well-known/openid-configuration` | RFC 8414 | OpenID Connect metadata discovery document. |

### Client Management (`/clients`) — Admin Protected

| Method | Endpoint | Description |
|---|---|---|
| `POST` | `/clients` | Register a new client (`client_name`, `client_type`, `redirect_uris`, `allowed_grant_types`, `allowed_scopes`). Generates Argon2-hashed secret. |
| `GET` | `/clients` | List all registered OAuth clients. |
| `GET` | `/clients/{id}` | Get client metadata. |
| `DELETE`| `/clients/{id}` | Deactivate and delete client registration. |

### Personal Access Tokens (`/pats`) — Authenticated User

| Method | Endpoint | Description |
|---|---|---|
| `POST` | `/pats` | Create a new PAT (`name`, `scopes`, `expires_in_days`). Returns raw `pat_...` once. |
| `GET` | `/pats` | List caller's active PATs (includes prefix, last used, expiration). |
| `DELETE`| `/pats/{id}` | Revoke a specific PAT immediately. |

---

## Security Model

1. **Password & Client Secret Storage**: Hashed with **Argon2id** (memory-hard, resistant to GPU/ASIC cracking).
2. **PKCE Enforcement**: Mandatory SHA-256 (`S256`) code challenge validation on authorization code grants to prevent code interception attacks.
3. **PAT Security**: Raw token entropy (192 bits) is returned **only once** at creation. The database exclusively holds the SHA-256 digest (`token_hash`).
4. **Single-Use Authorization Codes**: Automatically marked as `used` on exchange; replay attempts fail immediately.
5. **Constant-Time Comparison**: Cryptographic comparisons avoid timing attack vulnerabilities.

---

## Testing Guide

The crate features a comprehensive integration test suite covering all RFC grant flows, PKCE edge cases, PAT life-cycles, and introspection:

```bash
cargo test -p oauth
```

### Verified Scenarios:
- `test_rfc7523_jwt_bearer_m2m`: RFC 7523 M2M assertion grant & client assertion authentication.
- `test_authorization_code_pkce_flow`: Full PKCE cycle, S256 verification, rejection of reused codes.
- `test_personal_access_tokens`: PAT creation, Bearer usage, introspection, revocation, 401 rejection.
- `test_client_credentials_basic_auth_and_body`: M2M flow via HTTP Basic and POST body credentials.
- `test_refresh_token_rotation_and_revocation`: Single-use refresh token rotation and revocation.
- `test_introspection_and_discovery`: OIDC discovery and RFC 7662 introspection responses.
- `test_password_grant_form_and_json`: Dual URL-encoded form and JSON body parsing.
- `test_client_crud_admin_endpoints`: Admin client management lifecycle.
