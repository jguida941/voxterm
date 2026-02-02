#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

strip_ansi() {
    sed -E 's/\x1b\[[0-9;]*m//g'
}

run_case() {
    local width="$1"
    local lines="${2:-24}"
    local output
    local stripped
    local line
    local length

    output=$(
        VOXTERM_STARTUP_ONLY=1 \
        VOXTERM_FORCE_COLUMNS="$width" \
        VOXTERM_FORCE_LINES="$lines" \
        "$ROOT_DIR/start.sh" --help
    )

    stripped="$(printf "%s" "$output" | strip_ansi)"

    while IFS= read -r line; do
        length=$(
            printf "%s" "$line" | python3 - <<'PY'
import sys
import unicodedata

text = sys.stdin.read()
width = 0
for ch in text:
    width += 2 if unicodedata.east_asian_width(ch) in ("W", "F") else 1
print(width)
PY
        )
        if [ "$length" -gt "$width" ]; then
            echo "FAIL: width ${width} line length ${length}" >&2
            echo "$line" >&2
            return 1
        fi
    done <<< "$stripped"
}

run_case 60
run_case 80
run_case 100

echo "PASS: startup output fits within target widths."
