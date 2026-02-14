#!/bin/env bash

# Record demo videos by reading <!-- record: args --> markers from README.md.
# Requires: wf-recorder, a running Wayland compositor

set -euo pipefail

ROOT="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
OUTDIR="$ROOT/.github/assets"
CONFET="$ROOT/target/release/confet"
README="$ROOT/README.md"

mkdir -p "$OUTDIR"

if ! command -v wf-recorder &>/dev/null; then
    echo "wf-recorder not found"
    exit 1
fi

if [[ ! -x "$CONFET" ]]; then
    echo "confet binary not found — run 'cargo build --release' first"
    exit 1
fi

# Parse <!-- record: args --> lines from README
mapfile -t entries < <(grep -oP '(?<=<!-- record: ).+?(?= -->)' "$README")

if [[ ${#entries[@]} -eq 0 ]]; then
    echo "no <!-- record: ... --> markers found in README.md"
    exit 1
fi

echo "Found ${#entries[@]} demos to record"

for args in "${entries[@]}"; do
    name="${args%% *}"
    out="$OUTDIR/$name.mp4"
    echo "Recording: $name ($args)"

    for i in 3 2 1; do
        printf "  %d...\n" "$i"
        sleep 1
    done

    wf-recorder -f "$out" &
    rec_pid=$!
    sleep 0.3

    # shellcheck disable=SC2086
    "$CONFET" $args

    kill "$rec_pid" 2>/dev/null || true
    wait "$rec_pid" 2>/dev/null || true

    echo "  saved $out"
    sleep 0.5
done

echo "Done. Recordings in $OUTDIR"
