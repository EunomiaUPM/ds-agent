# dataplane

Data plane of the transfer agent. For each transfer process it keeps a dataplane process, moves
it through its states when the control plane sends a command, and relays the data through an
HTTP proxy with the back end's credentials.

The crate has no binary. The transfer agent registers it and drives it in-process through
`DataplaneManager`; there is no remote control protocol between them. It reads connector
instances from the catalog agent and credentials from the keystore.

## Structure

```
src/
├── lib.rs           Re-exports DataplaneManager, the commands, DataplaneAddress and the migrations
├── entities/        Dataplane processes, their logs and events, list filters
├── services/        Processes (cached in Redis), logs and events
├── engine/
│   ├── dataplane_manager/   DataplaneManager, commands, context, per-role handlers, proxy and
│   │                        runtime types, driver factory
│   └── dataplane_drivers/   Authenticators, proxy configurators, pub/sub subscribers,
│                            KeystoreClientImpl
├── testing_proxy/   The HTTP data proxy (used in production despite its name)
├── cache/           Redis cache of dataplane processes
├── data/            Repository traits, SeaORM repositories and migrations
├── http/            Control API: processes, logs and events
├── errors/          DataplaneError
└── setup/           DataplaneModule, DataplanePorts and AppContext
tests/
├── engine/          Authenticators, proxy, handlers per role and mode, strategy, manager
├── services/        Process, log and event services: tenant isolation and roles
└── support/         Contexts per auth config and shared mocks
```

## Processes and commands

A dataplane process belongs to one transfer process and one tenant. It records its role
(`Provider` or `Consumer`), interaction mode (`Pull` or `Push`), state, connector instance (provider
only), ingress and egress configuration, and `flow_control`, the runtime state of its driver.

The transfer agent calls `DataplaneManager::execute_command(DataplaneCommand)`:

| Command | Effect |
|---|---|
| `SetInit(AsProvider \| AsConsumer)` | Creates the process and walks it through `Configuring`, `Auth` and `Ready` |
| `SetConfiguring((continuation, address))` | Reconfigures with the peer's data address |
| `GetAssociated(continuation)` | Returns the process's address without changing it |
| `SetStarted`, `SetStopped` | Moves the state |
| `SetSubscribing`, `SetUnsubscribing` | Push only: subscribes or unsubscribes at the back end, then `Started` or `Stopped` |
| `SetTerminating` | `Terminated`, and deletes the runtime secrets of the process |

The answer is `Ok`, or `OkWithAddress(DataplaneAddress)` when the peer needs an address: the
control plane puts it in the DSP `DataAddress` (`endpointType`, `endpoint`, and the
`authType` and `authorization` endpoint properties). A failed subscribe or unsubscribe
terminates the process.

## Drivers

The driver factory picks three parts per process from its role, mode and connector instance:

| Part | Provider | Consumer |
|---|---|---|
| Authenticator | From the connector's `authentication`: none, basic, bearer, API key, OAuth 2.0 | None |
| Proxy configurator | HTTP pull or push; any other protocol is rejected | HTTP pull or push |
| Subscriber | Push over HTTP; Kafka is not implemented | Push: no-op |

Every process gets an ingress at `/dataplane/proxy/{dataplane_id}` on the agent's HTTP host.
The egress depends on the case:

| Case | Egress |
|---|---|
| Provider pull | The connector's `urlTemplate`, with the connector's credentials |
| Provider push | The consumer's address; the back end pushes to the provider's ingress |
| Consumer pull | The provider's address and token |
| Consumer push | The consumer's sink address, required |

The authenticator resolves the connector's credentials, reading keystore entries of the
process's tenant through `KeystoreClientImpl`. OAuth 2.0 access and refresh tokens are written
to the secret store under `/runtime/{id}/…` and replaced by placeholders in `flow_control`;
static credentials are kept as resolved.

## Data proxy

`ANY /dataplane/proxy/{dataplane_id}[/{path}]` forwards the request to the process's egress:

- The dataplane id in the URL is the capability: no bearer token is checked.
- Only processes in `Started` are served; any other state answers 403.
- The path and query are appended to the egress URL. Hop-by-hop headers and the incoming
  `Authorization` are dropped, and the process's credentials are added.
- Bodies are limited to 2 MiB; the upstream connection times out after 10 s and the exchange
  after 30 s.
- Each request is recorded as a transfer event of the process, with method, target and status.

The proxy's HTTP client accepts invalid upstream certificates.

## Setup

`DataplaneModule` implements `ServiceModuleTrait`:

- `DataplaneModule::compose(config, root, ports)` builds the services, the keystore stores
  (through `KeystoreModule::build_stores`) and the manager. It needs Redis at the transfer
  config's cache URL and fails at boot without it.
- `DataplanePorts::local(connector)` takes the catalog's in-process connector facade (monolith);
  `DataplanePorts::remote(config, root)` reads instances from the catalog agent's API with the
  service token.
- `local_manager()` returns the manager the transfer agent drives.
- `migrations()` creates `dataplane_transfers`, `dataplane_fields`, `dataplane_transfer_logs`
  and `transfer_events`.

The module mounts the control API under `{api}/transfer-agent/dataplane` behind the token
validator, and the proxy at `/dataplane/proxy` outside it.

## HTTP API

Under `{api}/transfer-agent/dataplane`, bearer required. Reads are limited to the caller's
tenant.

| Method | Path | Response |
|---|---|---|
| GET | `/dataplane-processes` | 200, page of processes |
| POST | `/dataplane-processes` | 201, the process |
| POST | `/dataplane-processes/batch` | 200, the processes with the given ids |
| GET | `/dataplane-processes/{id}` | 200, the process; 404 if missing |
| PUT | `/dataplane-processes/{id}` | 200, the process |
| DELETE | `/dataplane-processes/{id}` | 204 |
| GET | `/dataplane-processes/{id}/info` | 200, `{ id, interaction_mode, ingress_url }`; `ingress_url` only for pull |
| GET | `/dataplane-processes/transfer-process/{transfer_process_id}` | 200, the process of that transfer |
| GET | `/dataplane-processes/{id}/logs` | 200, its logs |
| GET | `/dataplane-processes/{id}/events` | 200, its transfer events |
| GET | `/transfer-events/{event_id}` | 200, the event |

## Tests

```
cargo test -p dataplane
```

`tests/engine` covers each authenticator, the proxy's credentials as headers and query
parameters, the handlers of each role and mode step by step, the strategy choice, and the
manager executing init and continuation commands. `tests/services` covers tenant isolation and
roles in the three services. Services and the connector facade are mocked; no database or
Redis is needed. The SeaORM repositories, the Redis cache and the HTTP routers have no tests
yet.
