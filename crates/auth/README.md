# auth

The SSI authentication agent. It holds the connector's wallet and DID, obtains verifiable
credentials from an authority, and onboards with peer connectors. It also guards access for
peers that onboard with this connector, and keeps the registry of known participants that the
DSP endpoints of every agent use to authenticate peers.

Onboarding uses GNAP grants: the side asking for access sends a grant request, the other side
answers with an OID4VP presentation request, and the grant continues once the presentation is
verified. Credentials are requested from an authority the same way, then redeemed over OID4VCI.

It runs as its own binary or as a module of the `monolith`. This crate does not issue user
tokens; that is `oauth`.

## Structure

```
src/
├── lib.rs           SERVICE_NAME (`ssi-auth-agent`)
├── main.rs          AgentCli::<AuthBoot>
├── core/            AuthCore: every service wired together; the trait bundling all modules
├── modules/         One trait per capability, with the use case logic as default methods
├── services/        GNAP gatekeeper, peer connector, VC requester, callbacks, Gaia-X
├── entities/        Participant and grant filters
├── types/           Request and response bodies
├── data/            Migrations and the SeaORM factory over ymir's repositories
├── facades/         In-process adapters of common's mates and SSI auth facades
├── http/            One router per capability; AuthRouter mounts them all
└── setup/           AuthModule, AppContext, SelfParticipantOnboarder, boot
tests/
├── modules/         Each capability over an AuthCore of doubles
├── entities/        Filters
└── support/         Builders, fixtures and a mock of every port of AuthCore
```

## Capabilities

| Module | What it does |
|---|---|
| Participant | Registry of the peers and authorities a tenant knows, and the connector itself (`myself`), derived from the wallet. `get_by_token` maps a peer's token to its participant |
| VC requester | Asks an authority for a credential (`beg`), follows its callback, redeems the credential over OID4VCI, or presents over OID4VP when asked |
| Peer connector | Asks a peer for access (`connect`), presents over OID4VP, follows the peer's callback and stores the peer with the token it grants |
| Gatekeeper | Answers a peer's GNAP grant request with a verification URI, and on continuation registers the verified peer with a new token |
| Verifier | Serves the presentation definition and verifies the peer's presentation (OID4VP draft 20), then closes the GNAP interaction |
| Gaia-X self-attester | Issues the connector's own Gaia-X credentials into its wallet; admin only, and only when `gaia_config` is set |

The DID is shared by every tenant. It advertises an `AuthorizationServer` service at
`{host}{api}/gate`, and peers append `/{tenant}/access` to reach the tenant they onboard with.

The wallet is Fafnir; the legacy `WaltId` option fails at boot.

## Integration

- `AuthModule::local_ports()` gives `AuthPorts` in-process: `MatesLocalFacade` (participants)
  and `SSIAuthLocalFacade` (peer token verification). The catalog, negotiation and transfer
  agents use them in the monolith; standalone, they call this agent's API with the service
  token.
- The local facades act as the service client's tenant (`admin_seed.tenant_id`) for
  cross-tenant lookups.

## Setup

`AuthModule::compose(config, root)` connects to the wallet and builds `AuthCore`. It mounts its
routes at the root of the HTTP plane, under `{api}`:

| Prefix | Protocol routes (no bearer) | User routes (bearer) |
|---|---|---|
| `/wallet` | | Wallet operations, from ymir's `WalletRouter` |
| `/mates` | | Participant registry |
| `/vc-request` | `/callback/{id}` | `/beg`, `/all`, `/{id}`, `/{id}/details`, `/oid4vci/{id}`, `/oid4vp/{id}` |
| `/gate` | `/{tenant}/access`, `/{tenant}/continue/{id}` | `/request/all`, `/request/{id}`, `/request/{id}/details` |
| `/peer-connection` | `/callback/{id}` | `/connect`, `/request/all`, `/request/{id}`, `/request/{id}/details`, `/oid4vp/{id}` |
| `/verifier` | `/pd/{state}`, `/verify/{state}` | |
| `/gaia` | | `/generate` |
| `/docs` | OpenAPI document | |

It also serves the wallet's well-known documents and a health route under `{api}`. Unknown
routes answer 404, and CORS allows any origin.

- `migrations()` creates the participant and resource request tables and the received and
  sent grant tables (grants, interactions, verifications), from ymir.
- `self_participant_onboarder()` is a seeder, registered by the monolith, that links the
  connector's own wallet as a participant on first boot.

Standalone, `AuthBoot` composes the module, validates bearer tokens with
`OAuthModule::validator`, and serves its local ports to itself:

```
cargo run -p auth -- setup -e <config.yaml>   # vault secrets and migrations
cargo run -p auth -- start -e <config.yaml>   # serve
```

Its config section is `ssi_auth`: `common`, `wallet_config`, `client_config`, `did_config`,
`verify_req_config` and the optional `gaia_config`.

## Participant API

Under `{api}/mates`, bearer required. Participants are always read and written in the caller's
tenant.

| Method | Path | Response |
|---|---|---|
| GET | `/all` | 200, page of participants |
| GET | `/myself` | 200, this connector as seen by the caller's tenant |
| GET | `/{id}` | 200, the participant |
| POST | `/batch` | 200, the participants among the given ids |
| POST | `/token` | 200, the participant owning a token |
| PUT | `/{id}` | 200, the participant with `extra_fields` merged |
| POST | `/` | 200, the participant |

## Tests

```
cargo test -p auth
```

`tests/modules` covers each capability over an `AuthCore` whose ports are mocks. A port without
expectations panics when called, so each test also shows what a flow does not touch. It covers
onboarding with peers and authorities, the gatekeeper and verifier answering them, and the
participant registry. `tests/entities` covers the filters. No wallet, database or authority is
needed. Onboarding against a running dev stack is in `crates/monolith/tests/auth/`
(`task test:onboarding`).
