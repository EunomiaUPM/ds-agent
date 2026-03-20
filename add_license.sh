#!/usr/bin/env bash
# Prepends the GPL-2 copyright banner to every .rs file that doesn't already have it.
# Usage: ./add_license.sh [directory]   (defaults to the script's own directory)

set -euo pipefail

ROOT="${1:-$(dirname "$(realpath "$0")")}"

BANNER='/*
 * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
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
'

added=0
skipped=0

while IFS= read -r -d '' file; do
    if head -n 5 "$file" | grep -q "Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM"; then
        skipped=$((skipped + 1))
    else
        tmp=$(mktemp)
        printf '%s\n' "$BANNER" > "$tmp"
        cat "$file" >> "$tmp"
        mv "$tmp" "$file"
        added=$((added + 1))
        echo "  + $file"
    fi
done < <(find "$ROOT" -name "*.rs" -not -path "*/target/*" -print0)

echo ""
echo "Done. Added banner to $added file(s), skipped $skipped file(s) (already had it)."
