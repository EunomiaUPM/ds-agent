#!/bin/bash
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

sh "$SCRIPT_DIR/populate_mock_data.sh"
sh "$SCRIPT_DIR/populate_catalog.sh"
sh "$SCRIPT_DIR/populate_mock_contracts.sh"
sh "$SCRIPT_DIR/populate_mock_transfers.sh"
#sh "$SCRIPT_DIR/populate_tck.sh"
