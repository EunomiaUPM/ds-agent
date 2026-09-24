#!/bin/bash
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

bash "$SCRIPT_DIR/populate_mock_data.sh"
bash "$SCRIPT_DIR/populate_mock_contracts.sh"
bash "$SCRIPT_DIR/populate_mock_transfers.sh"
bash "$SCRIPT_DIR/populate_mock_mates.sh"

#bash "$SCRIPT_DIR/populate_tck.sh"
