# Eunomia DS Protocol Tutorial (Management API & RPC)

This folder contains Python Jupyter notebooks demonstrating how to interact with the Eunomia Data Space Agents across both the **Management API (`/api/v1/...`)** and the **RPC Control Plane (`/dsp/current/...` & `/rpc/.well-known/...`)**, as well as the **Dataplane Proxy (`/dataplane/proxy/...`)**.

## Prerequisites

- [Docker](https://docs.docker.com/get-docker/) or the local Authority (`:1500`), Consumer (`:1100`), and Provider (`:1200`) services running.
- [Python 3.10+](https://www.python.org/downloads/) installed.
- [JupyterLab](https://jupyter.org/install) or VS Code with the Jupyter extension.

## Deployment

To start the system with all necessary components (Authority / Heimdall, Consumer, Provider, Vault, Postgres, etc.), use the provided auto-onboarding script from the project root directory:

```bash
# For MacOS/Linux
./scripts/onboarding/auto-onboarding-mini.sh

# For Windows (PowerShell)
./scripts/onboarding/auto-onboarding.ps1
```

## Running the Notebooks

1. Install the required Python dependencies:

```bash
pip install -r requirements.txt
```

2. Start JupyterLab:

```bash
jupyter lab
```

3. Open any of the notebooks in this folder and run the cells sequentially. Each high-level notebook automatically chains its prerequisite setup notebooks via `%run`.

## Notebooks Overview

### 1. Setup & Configuration (`Auth`, `Keystore`, `Catalog`, `Connector`)
- `setup_auth.ipynb`:
  - Checks agent health (`/health`) and starts lightweight local demo HTTP servers (`8081` PULL backend, `8082` PUSH subscription backend, `1112` Consumer PUSH sink) if those ports are not already bound.
  - Manages wallets via the Keystore Management API (`/api/v1/wallet/is-linked`, `/api/v1/wallet/link`, `/api/v1/wallet/info`, `/api/v1/wallet/dids`).
  - Inspects public DID Document and Verifiable Presentation well-known endpoints (`/.well-known/did.json`, `/.well-known/vp.json`).
  - Requests Verifiable Credentials from the Authority (`/api/v1/credentials/request`), checks status (`/api/v1/credentials/requests/{id}`), and lists issued credentials (`/api/v1/credentials`).
  - Exchanges peer-to-peer access tokens (`/api/v1/auth/token`) and queries peer records (`/api/v1/peers`).
- `setup_catalog_policies.ipynb`:
  - Populates the Provider DCAT catalog via `/api/v1/catalog-agent/...` with JSON-LD documents in the catalog context (`https://w3id.org/eunomia/v1/catalog-context.jsonld`): the main Catalog and DataService (`GET /catalogs/main`, `/data-services/main`), a Dataset (`POST /datasets`, placed with `inCatalog`) and Distributions (`POST /distributions`, placed with `ofDataset`; `GET /distributions/dataset/{id}` answers a `@graph`) for both `PULL` (`http+pull`) and `PUSH` (`http+push`), the DSP transfer format kept in `dct:format`.
  - Asks the doctor whether the dataset conforms to DSP, DCAT 3 and DCAT-AP (`GET /datasets/{id}/doctor?deep=true`) and reads the public DCAT-AP catalog (`GET /public/dcat-ap/catalog`, no token).
  - Registers ODRL policy templates (`/api/v1/catalog-agent/odrl-policies/templates`) and instantiates ODRL policies attached to the dataset (`/api/v1/catalog-agent/odrl-policies`, `/api/v1/catalog-agent/odrl-policies/entity/{id}`).
  - The Consumer fetches the Provider's DSP Catalog and one Dataset through its catalog RPC (`POST /dsp/current/catalog/rpc/setup-catalog-request`, `setup-dataset-request`, eunomia bodies `RpcCatalogRequestMessage` / `RpcDatasetRequestMessage`, answered as an `RpcCatalogResult`).
  - Registers `NO_AUTH` connector instances (`/api/v1/connector-instances`, `/api/v1/connector-instances/slug/{slug}`, `/api/v1/connector-instances/distribution/{id}`) using the `generic-pull-rest-no-auth` and `generic-push-Subscription` templates (`/api/v1/connector-templates`).
- `setup_catalog_policies_oauth.ipynb`:
  - Same catalog & ODRL policy workflow as `setup_catalog_policies.ipynb`, plus secret provisioning in the Keystore (`POST/GET /api/v1/secrets`) at `/ecostars/api_key`.
  - Registers a `BEARER_TOKEN` PULL connector instance (`generic-pull-rest-bearer`) that injects the secret token from Vault when forwarding requests through the dataplane proxy.

### 2. Contract Negotiation (DSP Control Plane RPC & Management API)
- `contract_negotiation.ipynb`:
  - Full canonical DSP 2025-1 manual step-by-step Contract Negotiation sequence over `/dsp/current/negotiations/rpc/...` (`setup-request` -> `setup-offer` -> counter `setup-request` -> `setup-acceptance` -> `setup-agreement` -> `setup-verification` -> `setup-finalization`).
  - Inspects `NegotiationProcessView` (`correlation.consumerPid`, `correlation.providerPid`), message history (`/api/v1/negotiation-agent/negotiation-messages/process/{id}`), offers (`/api/v1/negotiation-agent/offers/process/{id}`), and stored ODRL agreements (`/api/v1/negotiation-agent/agreements/process/{id}`, `/api/v1/negotiation-agent/agreements/{id}`) on both Consumer and Provider.
  - Also demonstrates manual termination (`setup-termination`). Callable via `%run ./contract_negotiation.ipynb`.
- `bff_contract_negotiation.ipynb`:
  - Automated single-call Contract Negotiation flows (`POST /dsp/current/negotiations/rpc/request` from Consumer and `POST /dsp/current/negotiations/rpc/offer` from Provider) where the entire state machine executes automatically end-to-end.
  - Queries the resulting `NegotiationProcessView`, messages, offers, and `AgreementView` via `/api/v1/negotiation-agent/...`. Callable via `%run ./bff_contract_negotiation.ipynb`.

### 3. Transfer Process & Dataplane Execution
- `pull_flow.ipynb`:
  - Complete `PULL` data transfer lifecycle over `/dsp/current/transfers/rpc/...` (`setup-request`, `setup-start`, `setup-suspension`, restart, role/shape validation checks, and `setup-termination`).
  - Inspects `TransferProcessView` (`/api/v1/transfer-agent/transfer-processes/{id}`), message trail (`/api/v1/transfer-agent/transfer-messages/process/{id}`), and Dataplane state (`/api/v1/transfer-agent/dataplane/dataplane-processes/transfer-process/{id}`, `/info`, `/logs`, `/events`).
  - Fetches live data through the Provider's **Dataplane Proxy** (`GET /dataplane/proxy/{dataplaneId}`), verifying `200 OK` when `STARTED` and `403 Forbidden` when `SUSPENDED` or `TERMINATED`.
- `push_flow.ipynb`:
  - Complete `PUSH` data transfer lifecycle over `/dsp/current/transfers/rpc/...` (`setup-request` with Consumer sink `dataAddress`, `setup-start` triggering backend subscription, `setup-suspension` triggering unsubscribe, restart, role validation checks, and `setup-completion`).
  - Simulates backend data emission through `POST /dataplane/proxy/{providerDataplaneId}` -> Consumer `/dataplane/proxy/{consumerDataplaneId}` -> Consumer sink (`127.0.0.1:1112`) and inspects `TransferEvent` and `TransferLog` records on both peers.
- `bff_push_flow.ipynb`:
  - End-to-end `PUSH` data transfer flow driven by an agreement obtained from `bff_contract_negotiation.ipynb` and initiated via the **BFF Auto-Start route** (`POST /dsp/current/transfers/bff-rpc/setup-request`, which sets `"autoStart": true` / `eunomia:autoStart`).
  - Demonstrates how the Provider's `AutoStartWorker` (`transfer-auto-start`) automatically sends `TransferStartMessage` and subscribes to the backend (`REQUESTED -> STARTED`) without a manual `setup-start` call, verified through `/api/v1/transfer-agent/...` and `/dataplane/proxy/...`.
