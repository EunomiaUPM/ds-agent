#!/usr/bin/env bash
# Normalises the GPL-3 licence banner at the top of every .rs and .proto file.
#
#   - If the file starts with a /* ... */ block containing "Copyright", that
#     block is REPLACED (this is what fixes the drifted variants).
#   - If it has no leading block comment, the banner is prepended.
#   - A leading block comment that is NOT a licence is left untouched (the
#     banner goes above it).
#   - Idempotent: running it twice changes nothing.
#
# Usage:
#   scripts/add_license.sh [dir]            # rewrite in place (default: crates)
#   scripts/add_license.sh [dir] --check    # report only, exit 1 if any drift

set -euo pipefail

ROOT="crates"
CHECK=0
for arg in "$@"; do
    case "$arg" in
        --check) CHECK=1 ;;
        *) ROOT="$arg" ;;
    esac
done

read -r -d '' BANNER <<'EOF' || true
/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */
EOF
export BANNER

# Strips a leading licence block (if any), then re-prepends the canonical banner
# followed by exactly one blank line.
NORMALISE='
    BEGIN { $b = $ENV{BANNER}; $b =~ s/\s+\z//; }
    s{\A/\*.*?\*/}{}s if m{\A(/\*.*?\*/)}s && $1 =~ /Copyright/i;
    s{\A\s+}{};
    $_ = $b . "\n\n" . $_;
'

changed=0
ok=0

while IFS= read -r -d '' file; do
    before=$(md5 -q "$file")

    if [ "$CHECK" -eq 1 ]; then
        after=$(perl -0777 -pe "$NORMALISE" "$file" | md5 -q)
    else
        perl -0777 -i -pe "$NORMALISE" "$file"
        after=$(md5 -q "$file")
    fi

    if [ "$before" = "$after" ]; then
        ok=$((ok + 1))
    else
        changed=$((changed + 1))
        echo "  ~ $file"
    fi
done < <(find "$ROOT" \( -name "*.rs" -o -name "*.proto" \) -not -path "*/target/*" -print0)

echo ""
if [ "$CHECK" -eq 1 ]; then
    echo "Check: $changed file(s) would change, $ok already canonical."
    [ "$changed" -gt 0 ] && exit 1
    exit 0
else
    echo "Done: rewrote $changed file(s), $ok already canonical."
fi
