# Reproducible End-to-End Examples (`HTTP PULL` & `HTTP PUSH`)

This directory provides a self-contained Docker Compose playground that reproduces both interaction modes of the **Eunomia `ds-protocol` connector and governed HTTP data plane** in under 15 minutes, with **zero external authority or cloud dependencies**.

## Architecture

```text
Host CLI (onboard.sh / pull-demo.sh / push-demo.sh)
   │
   ├──► Provider Agent (:1200)  ◄──── DSP 2025-1 + GNAP/OID4VP ────►  Consumer Agent (:1100)
   │      ├─ connector (templates & instances)                          ├─ transfer-agent
   │      ├─ keystore  (/edaccit/*, /ecostars/*)                        └─ HttpDataProxy (/dataplane/proxy/{id})
   │      └─ HttpDataProxy (/dataplane/proxy/{id})
   │              │                                                            │
   │              ▼                                                            ▼
   └────► Mock Backend & Sink (:18080, Python 3 stdlib) ◄──────────────────────┘
            ├─ GET    /arcgis/rest/services/edaccit/FeatureServer/0/query  (API_KEY in QUERY)
            ├─ POST   /esg/webhooks + POST /esg/emit + DELETE /esg/webhooks/{id} (BEARER_TOKEN)
            └─ POST   /consumer-sink/events                                (Consumer push sink)
```

Both participants use self-attested **Gaia-X `LegalPerson` Verifiable Credentials** stored in their local Fafnir wallets (`POST /api/v1/gaia/generate`), enabling real **GNAP + OID4VP** mutual authentication without spinning up a third SSI authority stack.

## Prerequisites

- Docker Engine ≥ 24 with `docker compose` v2
- `curl` and `jq` on the host (`task` optional)
- ~4 GB RAM available for the containers (first run builds `eunomia-local:examples` from `../deployment/Dockerfile`, or reuses `$EUNOMIA_IMAGE` if set)

## Quickstart (4 commands)

From the repository root:

```bash
# 1. Build (or pull) and start Provider (:1200), Consumer (:1100), and Mock Backend (:18080)
docker compose -f examples/docker-compose.yaml up -d --build --wait

# 2. Link wallets, self-issue Gaia-X LegalPerson VCs, and onboard Consumer <-> Provider
./examples/scripts/onboard.sh

# 3. Run the EDACCIT-inspired HTTP PULL scenario (INFINITE stream + single-shot FINITE snapshot)
./examples/scripts/pull-demo.sh
./examples/scripts/pull-finite-demo.sh

# 4. Run the ECOSTARS-inspired HTTP PUSH scenario
./examples/scripts/push-demo.sh
```

Or via `Taskfile.yml`:

```bash
task examples:up
task examples:pull
task examples:pull-finite
task examples:push
task examples:down
```

## What Each Script Verifies

### 1. `./examples/scripts/pull-demo.sh` (EDACCIT-style `HTTP PULL`)

1. Stores `/edaccit/arcgis-token` in the Provider `keystore` (`POST /api/v1/keystore/secrets`).
2. Registers `edaccit-arcgis-pull` (`ConnectorTemplate`) with `API_KEY` (`location: QUERY`, `content: "{{__RUNTIME_SECRET_{/edaccit/arcgis-token}__}}"`) and binds a `ConnectorInstance` to an `http+pull` DCAT `Distribution`.
3. Executes DSP 2025-1 catalog discovery, contract negotiation (`FINALIZED`), and transfer start (`STARTED`).
4. Verifies governance & security invariants:
   - Deleting the bound `ConnectorInstance` while the transfer is `STARTED` returns **`409 Conflict`**.
   - Calling the Provider `HttpDataProxy` without its per-transfer capability token returns **`401 Unauthorized`**.
   - Pulling through the Consumer's local `HttpDataProxy` returns **`200 OK`** with 3 GeoJSON features while keeping the upstream ArcGIS key strictly on the Provider.
5. Suspends (`SUSPENDED` → `403`), resumes (`STARTED` → `200`), and terminates (`TERMINATED` → `403`) the transfer.

### 2. `./examples/scripts/push-demo.sh` (ECOSTARS-style `HTTP PUSH`)

1. Stores `/ecostars/webhook-token` in the Provider `keystore`.
2. Registers `ecostars-webhook-push` (`ConnectorTemplate`) using:
   - `BEARER_TOKEN` resolved via `{{__RUNTIME_SECRET_{/ecostars/webhook-token}__}}`,
   - `subscribe`: `POST` registering `http://provider:1200{{__RUNTIME_INGRESS__}}` (which automatically embeds `?ingress_token=<256-bit-token>`),
   - `unsubscribe`: `DELETE {{__WEBHOOK_BASE_URL__}}/{{__RUNTIME_JSON_{.subscription.id}__}}`.
3. Negotiates an agreement (`FINALIZED`) and starts an `http+push` transfer whose Consumer `DataAddress` points to `http://mock-backend:8080/consumer-sink/events`.
4. Verifies that:
   - Spoofed webhook calls to `/dataplane/proxy/{id}` without `?ingress_token=` are rejected with **`401 Unauthorized`**.
   - Triggering `POST /esg/emit` delivers the ESG payload through Provider `HttpDataProxy` → Consumer `HttpDataProxy` → `/consumer-sink/events` with `?ingress_token` stripped and zero provider credential leakage.
   - Terminating the transfer automatically invokes `DELETE /esg/webhooks/sub-1` (leaving `0` active subscriptions) and transitions the dataplane to `Terminated`.

### 3. `./examples/scripts/pull-finite-demo.sh` (`FINITE` Single-Shot `HTTP PULL`)

1. Registers `edaccit-arcgis-pull-finite` with `"lifetime": "FINITE"` and `"maxRetries": 1`.
2. Negotiates an agreement and starts the `http+pull` transfer (`STARTED`).
3. Pulls the GeoJSON snapshot once via the Consumer's local `HttpDataProxy` (`200 OK`, 3 features) and verifies that `DataplaneSignal::Completed` emitted at stream EOF automatically sends `TransferCompletionMessage` over DSP 2025-1, transitioning **both** Provider and Consumer to `COMPLETED` and rejecting any second pull (`403` / `410`).

## Tear Down

```bash
docker compose -f examples/docker-compose.yaml down -v
```
