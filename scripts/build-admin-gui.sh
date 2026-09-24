#!/bin/bash
# Builds the admin GUI and embeds it into the BFF (replaces the former `bff build` subcommand).
#
# Usage: scripts/build-admin-gui.sh
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DEST="$ROOT/crates/bff/src/static/admin"

(cd "$ROOT/gui" && npm run build -w admin)

rm -rf "${DEST:?}"/*
mkdir -p "$DEST/dist"
cp -r "$ROOT/gui/admin/dist/"* "$DEST/dist/"
echo "Admin GUI copied into $DEST/dist"
