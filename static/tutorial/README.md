# Rainbow Negotiation Agent Tutorial

This folder contains Python notebooks demonstrating how to interact with the Rainbow Negotiation Agent and Transfer Agent.

## Prerequisites

- [Docker](https://docs.docker.com/get-docker/) installed and running.
- [Python 3.8+](https://www.python.org/downloads/) installed.
- [JupyterLab](https://jupyter.org/install) or VS Code with Jupyter extension.

## Deployment

To start the system with all necessary components (Mock Provider, Consumer, Keycloak, Vault, etc.), use the provided auto-onboarding script.

**From the project root directory:**

```bash
# For MacOS/Linux
./scripts/bash/auto-onboarding.sh

# For Windows (PowerShell)
./scripts/powershell/auto-onboarding.ps1
```

This script will:
1. Start the Docker containers using `docker-compose.dev.yaml`.
2. Wait for services to be ready.
3. Onboard participants and generate necessary secrets.

**Note:** Ensure you have `jq` and `curl` installed if running the bash script.

## Running the Notebooks

1. Install the required Python dependencies:

```bash
pip install -r requirements.txt
```

2. Start JupyterLab:

```bash
jupyter lab
```

3. Open the notebooks in this folder (e.g., `01_negotiation.ipynb`) and follow the instructions.

## Notebooks

- `01_negotiation.ipynb`: Demonstrates how to create a negotiation process, send messages, and reach an agreement.
- `02_transfer.ipynb`: Demonstrates how to initiate a data transfer after an agreement is established.
