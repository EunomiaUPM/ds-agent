# Runtime construction of governed dataplanes for dataspace connectors

**Draft manuscript — SoftwareX structure (Original Software Publication).**
This report describes the system *as currently designed and implemented* in this repository
(`EunomiaUPM/rainbow`, branch `refactor/transfer-agent`). Forward-looking material lives in
`crates/connector/DESIGN.md` and is only referenced in §6 (Future work).

---

## Required metadata

| Nr. | Code metadata description | Value |
|---|---|---|
| C1 | Current code version | (fill: e.g. v0.x — tag before submission) |
| C2 | Permanent link to code/repository | https://github.com/EunomiaUPM/rainbow |
| C3 | Permanent link to reproducible capsule | (optional — fill if a CodeOcean/Zenodo capsule is minted) |
| C4 | Legal code license | GPL-3.0-or-later |
| C5 | Code versioning system used | git |
| C6 | Software code languages, tools and services used | Rust (tokio, axum, sea-orm, serde, reqwest), TypeScript/React (admin GUI), OpenAPI (orval-generated client), PostgreSQL |
| C7 | Compilation requirements, operating environments | Rust stable toolchain, cargo workspace; PostgreSQL; Linux/macOS; Docker for pilot deployments |
| C8 | Link to developer documentation | `docs/` (fumadocs site, published under `/ds-agent`) |
| C9 | Support email for questions | (fill: institutional contact, UPM) |

---

## 1. Motivation and significance

Data spaces — federated data-sharing ecosystems built on the IDSA **Dataspace Protocol
(DSP)** — separate the *control plane* (catalog browsing, contract negotiation, transfer
process signalling) from the *data plane* (the actual movement of data between a provider's
backend and a consumer). The control-plane side of DSP is well specified; the data-plane
side deliberately is not: DSP only standardises the handshake by which endpoints and access
tokens (`DataAddress`) are exchanged. How a connector authenticates against a provider's
backend, how it subscribes to push feeds, and how data actually flows remain
implementation-defined.

Existing connector implementations (notably the Eclipse Dataspace Components, EDC) address
this with **compiled-in extensions**: supporting a new backend technology or authentication
scheme means writing a Java extension, rebuilding, and redeploying the connector. For the
long tail of dataspace participants — SMEs, public administrations, research
infrastructures — that is a significant operational barrier: the people who know the
backend API are rarely the people who can extend and redeploy a connector.

The software presented here takes a different position: **the dataplane is not compiled,
it is constructed at runtime**. A connector's behaviour towards a backend is captured in a
declarative, parameterised **connector template** (a JSON document validated and stored by
the connector service). Instantiating a template with concrete parameter values yields a
**connector instance** bound to a catalog distribution. When a DSP transfer process reaches
the data-flow phase, the runtime *assembles* a dataplane for that specific transfer —
authentication strategy, proxy configuration, and pub/sub lifecycle driver are selected and
composed from the instance's declarative description, executed through a persisted state
machine, and torn down when the transfer terminates. Adding support for a new REST API,
changing an authentication scheme, or repointing a subscription callback requires editing a
JSON template through the administration GUI — not recompiling the connector.

The scientific and practical significance is twofold: (i) it demonstrates that the
unspecified half of DSP — provider-side data-plane behaviour — can be captured in a small
declarative DSL with a well-defined parameter-resolution pipeline, and (ii) it provides a
memory-safe, statically-typed reference implementation (Rust) of the full DSP transfer
stack in which every per-transfer dataplane is reconstructible from persisted state,
enabling crash recovery and horizontal operation without in-memory session affinity.

## 2. Software description

### 2.1 Architecture overview

The system is a Rust cargo workspace of thirteen crates implementing a complete dataspace
connector agent. The crates relevant to the dataplane construction system are:

| Crate | Responsibility |
|---|---|
| `connector` | Connector template & instance model: DSL types, parameter pipeline, secret management, template/instance CRUD services |
| `dataplane` | Per-transfer dataplane: context, composite driver factory, lifecycle state machine, HTTP proxy, pub/sub drivers, runtime secret vault |
| `transfer-agent` / `transfer-agent-ref` | DSP transfer-process protocol engine (control plane) that commands the dataplane |
| `catalog-agent`, `negotiation-agent` | DSP catalog and contract negotiation (control plane) |
| `keystore`, `auth`, `oauth` | Secret storage and participant authentication |
| `common`, `events`, `bff`, `monolith` | Shared config, transfer events, GUI gateway, single-binary composition |

A React administration GUI consumes the services through an OpenAPI-described gateway; its
client types are generated (orval), so the DSL surface shown to operators is the same
serde-defined schema the Rust services validate.

Deployments are two-sided, as in the DSP model: a consumer-side agent and a provider-side
agent each run their own control plane and dataplane; only the provider side holds
connector instances (it owns the backend credentials), while the consumer side runs a
credential-less dataplane that terminates the DSP `DataAddress` handshake.

**[Fig. 1 — placeholder: two-plane deployment diagram: consumer app → consumer dataplane →
provider dataplane → provider backend, with DSP signalling between control planes.]**

### 2.2 The connector DSL: templates, parameters, instances

A **connector template** (`ConnectorTemplateDto`) is the reusable blueprint. It has four
sections:

1. **Metadata** — name, version, author, description (templates are addressed by
   name + version).
2. **Authentication** (`AuthenticationConfig`, internally tagged `"type"`):
   `NO_AUTH`, `BASIC_AUTH`, `BEARER_TOKEN`, `API_KEY` (header or query location), and
   `OAUTH2` (client-credentials or resource-owner-password grants, with a declarative
   `on_token_expire` policy). Secret-bearing fields are `SecretString` values whose source
   is one of `Plain`, `Base64`, `VaultRef {path, key}`, or `EnvVar` — so a template can
   reference a vault path rather than embed a credential.
3. **Interaction** (`InteractionConfig`, tagged `"mode"`): `PULL` (a single `dataAccess`
   protocol spec the dataplane proxies on demand) or `PUSH` (a `subscribe` spec that
   registers a callback with the remote system, plus an optional `unsubscribe` spec).
   Protocol specs are tagged `"protocol"`: `HTTP` (`urlTemplate`, `method`, `headers`,
   `bodyTemplate`) is fully implemented end-to-end; `KAFKA` is declared in the DSL with
   its spec type and is rejected by the runtime driver factory as not yet implemented.
4. **Parameters** — a list of `ParameterDefinition` entries (name, title, description,
   type ∈ {STRING, INT, BOOLEAN, VEC\<STRING\>, MAP\<STRING,STRING\>}, required flag,
   default value) declaring every placeholder the other sections use.

Template strings admit five placeholder forms, each resolved at a different binding time:

| Placeholder | Resolved | Source |
|---|---|---|
| `{{__NAME__}}` | at instantiation | operator-supplied instance parameter |
| `{{__SYS_*__}}` | at instantiation | system values: fresh URN, token, timestamp, ISO-8601 date, own callback URL (with a Docker-internal variant) |
| `{{__RUNTIME_JSON_{<jq>}__}}` | at transfer runtime | jq expression evaluated over JSON responses captured earlier in the same transfer (e.g. the subscribe response) |
| `{{__RUNTIME_PARAMETER_{/key}__}}` | at transfer runtime | per-transfer runtime parameter store |
| `{{__RUNTIME_SECRET_{/key}__}}` | at transfer runtime | keystore lookup, never persisted in clear |

The parameter pipeline is implemented as visitors over the typed template tree
(`ConnectorTemplateWalker`): an *extractor* collects every placeholder present in the
template, a *validator* enforces the bidirectional property — every placeholder is
declared and every declaration is used — at template-creation time, and an
*instance resolver* substitutes instantiation-time values to produce a
**connector instance** (`ConnectorInstanceDto`), which additionally binds the instance to
a catalog `distribution_id`. A separate *runtime resolver* substitutes the three
runtime-scoped forms during the transfer itself, including interpolated jq evaluation.
Because resolution is type-aware, numeric/boolean/list/map fields accept either a literal
value or a placeholder string (`TemplateInt`, `TemplateBoolean`, `TemplateVecString`,
`TemplateMapString` untagged unions).

Templates and instances are persisted with their spec as a JSON document (`spec` column),
so the DSL can evolve without schema migrations; CRUD services
(`ConnectorTemplateEntitiesTrait`, instance counterpart) expose them over the gateway API,
and instances are resolvable by contract-agreement id — the join point between the DSP
control plane and the dataplane.

### 2.3 Runtime dataplane construction

The central runtime object is the **`DataplaneContext`**: the per-transfer aggregate of
(a) the persisted dataplane transfer record (role ∈ {Provider, Consumer}, interaction
mode ∈ {Pull, Push}, lifecycle state, ingress/egress proxy config, flow-control blob),
(b) the resolved connector instance, if the local side owns one, (c) the assembled
driver, (d) runtime values, and (e) the effective forward `DataAddress`.

**Composite driver assembly.** A dataplane driver is a composite of three orthogonal
strategy axes, mirroring the questions every transfer must answer:

```rust
pub struct DataplaneDriver {
    pub authenticator:      Arc<dyn DriverAuthenticatorTrait>,   // how to obtain credentials
    pub proxy_configurator: Arc<dyn DriverProxyConfiguratorTrait>, // how to set up the data channel
    pub subscriber:         Option<Arc<dyn DriverPubSubTrait>>,  // push lifecycle, if any
}
```

The `DataplaneDriverFactory` resolves each axis at runtime from the declarative
description: the authenticator from the instance's `AuthenticationConfig` variant
(`NoAuth`, `BasicConfig`, `BearerToken`, `ApiKey`, `Oauth` — the OAuth authenticator
performs the token exchange and caches the token with its expiry in the runtime state — or
`NoOp` for the credential-less consumer side); the proxy configurator from the triple
(protocol, role, interaction mode) — currently the four HTTP combinations
{provider, consumer} × {pull, push}; and the subscriber only for push transfers
(`HttpPubSubscriber`, keystore-aware). Unsupported combinations are rejected with typed
errors at driver-construction time. The factory is behind a trait
(`DataplaneDriverFactoryTrait`) and every axis trait carries mockall-generated test
doubles, so lifecycle logic is tested independently of any real backend.

**Lifecycle state machine.** Transfer lifecycle logic lives in a state-machine trait
(`DataplaneCommandStateMachine`) whose *default methods* implement the shared transition
skeleton — a Template Method design: `set_init` composes configuring → authenticating →
ready; `set_subscribing` persists the `Subscribing` state, invokes the driver's
`subscribe`, and on success promotes to `Started` (on failure it degrades to
`Terminated`); `set_unsubscribing` mirrors it towards `Stopped`; `set_terminating` cleans
up per-transfer secrets. Concrete handlers per (role × mode) — e.g.
`DataplaneHandlerProviderPull`, `DataplaneHandlerConsumerPull` — override only what
differs. Observable states are `Init`, `Configuring`, `Auth`, `Ready`, `Subscribing`,
`Started`, `Unsubscribing`, `Stopped`, `Terminated`; every transition is persisted before
its side effect, and each transition step updates the transfer record through the entity
service, producing an auditable trail.

**Statelessness and rehydration.** Nothing about a live transfer exists only in memory.
The proxy ingress/egress configuration and the runtime values (`flow_control` column) are
persisted with the transfer record; secrets captured at runtime are exported to the
secret store through a `RuntimeSecretVault` rather than written to the database, and are
deleted on termination. `DataplaneContext::from_continuation` reconstructs the full
context — record, connector instance, driver (re-assembled through the factory), runtime,
proxy — from persistence alone. Any control-plane message (suspend, resume, terminate,
completion) and any proxied data request can therefore be served after a restart, by a
different process, with no session affinity.

**Push lifecycle with runtime data flow.** For push transfers, `HttpPubSubscriber`
resolves the subscribe spec's runtime placeholders (injecting, e.g., the dataplane's own
callback URL via `SYS_OWN_URL`), executes the HTTP subscribe call, captures the JSON
response into the runtime state, and persists it. The unsubscribe spec can then reference
values from that response — typically the remote subscription id — through
`{{__RUNTIME_JSON_{subscribe.data.ID}__}}`, closing the loop declaratively: the template
author never writes code to thread the subscription id from one call to the other.

**Data path.** The provider-side dataplane exposes an HTTP ingress per transfer (endpoint
+ bearer token materialised as the DSP `DataAddress` handed to the consumer). Incoming
consumer requests are validated against the transfer's lifecycle state (`Started`
required), authorised, and forwarded to the backend egress with credentials injected by
the authenticator — a governed L7 proxy. Transfer events (state changes, data-flow
activity) are recorded through the `events` crate for monitoring and compliance auditing.

**[Fig. 2 — placeholder: runtime construction sequence: DSP TransferRequest → resolve
instance by agreement id → DataplaneContext::from_init → driver factory (3 axes) → state
machine set_init → DataAddress returned → proxied data flow → rehydration on later
messages.]**

### 2.4 Software functionalities (summary)

- Declarative connector templates: 5 authentication schemes × pull/push lifecycles over
  parameterised HTTP specs, with typed, validated parameter declarations.
- Template → instance instantiation with bidirectional placeholder validation, system
  parameters, and vault-backed secret sources.
- Runtime dataplane assembly per transfer: composite strategy driver (authenticator /
  proxy configurator / subscriber) resolved from the declarative instance.
- Persisted lifecycle state machine with shared transition skeleton and per-(role×mode)
  handlers; full context rehydration from persistence (crash/restart safety).
- Runtime parameter resolution with jq extraction over captured responses; per-transfer
  secret vault with cleanup on termination.
- Governed HTTP proxy data path with state-gated access and OAuth token refresh.
- Administration GUI (React) over an OpenAPI gateway with generated, schema-faithful
  client types; transfer event trail for auditing.

## 3. Illustrative example

A provider exposes an event feed behind a webhook-subscription API that requires a bearer
token. The operator writes one template (abridged; exact serde wire format):

```jsonc
{
  "name": "webhook-events", "version": "1.0.0", "author": "UPM",
  "authentication": {
    "type": "BEARER_TOKEN",
    "token": { "source": { "plain": "{{__API_TOKEN__}}" } }
  },
  "interaction": {
    "mode": "PUSH",
    "subscribe": {
      "protocol": "HTTP",
      "urlTemplate": "https://{{__HOST__}}/api/v1/subscriptions",
      "method": ["POST"],
      "headers": { "Content-Type": "application/json" },
      "bodyTemplate": "{ \"callback\": \"{{__SYS_OWN_URL__}}\", \"dataset\": \"{{__DATASET__}}\" }"
    },
    "unsubscribe": {
      "protocol": "HTTP",
      "urlTemplate": "https://{{__HOST__}}/api/v1/subscriptions/{{__RUNTIME_JSON_{subscribe.data.ID}__}}",
      "method": ["DELETE"]
    }
  },
  "parameters": [
    { "name": "HOST",      "title": "API host",     "paramType": "STRING", "required": true },
    { "name": "API_TOKEN", "title": "Bearer token", "paramType": "STRING", "required": true },
    { "name": "DATASET",   "title": "Dataset id",   "paramType": "STRING", "required": true }
  ]
}
```

Template creation validates that the three declared parameters and the placeholders in
use match exactly. The operator instantiates it against a catalog distribution, supplying
`HOST`, `API_TOKEN`, `DATASET`; instance resolution substitutes them (the token via the
secret pipeline) and the instance is bound to the distribution.

At transfer time: the consumer's control plane negotiates a contract and sends a DSP
`TransferRequest`; the provider's transfer agent resolves the connector instance by
agreement id and initialises the dataplane. The state machine drives
Init → Configuring → Auth → Ready; on start, `set_subscribing` executes the subscribe
call — `SYS_OWN_URL` having been replaced by the dataplane's own callback ingress — and
captures the response `{"data": {"ID": "sub-42", ...}}` into the persisted runtime. The
remote system now pushes events to the dataplane, which forwards them to the consumer's
`DataAddress`. When the transfer terminates, `set_unsubscribing` resolves the unsubscribe
URL to `/api/v1/subscriptions/sub-42` from the persisted runtime — surviving any restart
in between — and the vaulted runtime secrets are deleted. No code was written or deployed
at any step.

## 4. Impact

- **Lowers the operational barrier of dataspace participation.** The persona able to
  onboard a backend into a dataspace changes from "connector developer" to "API-literate
  operator with a GUI". In the Eunomia pilots (energy/ecolabel and tourism deployments
  operated alongside this repository), connector behaviour is provisioned as
  configuration, not as software releases.
- **A reference answer to DSP's unspecified data plane.** The template/instance model and
  the three-axis driver decomposition (authenticate / configure channel / manage
  subscription lifecycle) give a reusable vocabulary for the part of the protocol the
  specification leaves open, grounded in a working implementation rather than a position
  paper.
- **Reliability properties uncommon in connector runtimes.** Because every per-transfer
  dataplane is reconstructible from persisted state (context rehydration, vaulted runtime
  secrets, state-gated proxy), the agent tolerates restarts mid-transfer — including
  between subscribe and unsubscribe of a push feed — which matters for long-lived
  transfers typical of institutional data sharing.
- **Research platform.** The declarative DSL is a substrate for studying dataplane
  extensibility: ongoing design work (documented in-repo) explores operation DAGs,
  long-lived governed channels for ten canonical transfer cases, and sandboxed WebAssembly
  extension points over this same runtime.

## 5. Limitations

The current DSL and runtime are HTTP-centric: the `KAFKA` protocol spec is declared but
its driver is not implemented, and the closed enums (authentication, interaction,
protocol) mean new technologies require extending the Rust crates. Pull transfers proxy a
single endpoint; multi-step flows (login → fetch → logout) are not yet expressible.
Authorization is state- and token-gated at the proxy; fine-grained policy-decision-point
integration is deployment-specific. These boundaries delimit the published system and
motivate the future work below.

## 6. Conclusions and future work

The software shows that a dataspace connector's dataplane can be constructed at runtime
from a declarative, parameterised template language with a typed resolution pipeline, and
operated through a persisted, rehydratable state machine — replacing the compile-and-
redeploy extension model with configuration. Planned evolution (design documented in
`crates/connector/DESIGN.md`) generalises the DSL to named operations with Airflow-style
dependency flows, a registry of protocol bindings validated by declared capabilities,
long-lived governed channels covering ten canonical transfer cases (databases, object
storage, streaming, tunnels), and sandboxed WebAssembly plugins with a fixed ABI for
third-party authenticators, transformers, and lifecycle hooks — all backwards-compatible
with the JSON-persisted template model described here.

## Acknowledgements

(fill: funding — Proyecto Eunomia, UPM; pilot partners.)

## References

(fill: IDSA Dataspace Protocol specification; Eclipse Dataspace Components; Gaia-X;
related connector literature. To be completed with the journal's citation style.)
