#!/bin/env bash

# Record demo videos by reading <!-- record: args --> markers from README.md.
# Requires: wf-recorder, ffmpeg, a running Wayland compositor
#
# Marker format: <!-- record: <confet-args> [name=N] [scale=N] [crf=N] -->
#   name=   output filename (default: first confet arg)
#   scale=  vertical resolution (default: 720)
#   crf=    h264 quality, lower=better (default: 24)

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

for entry in "${entries[@]}"; do
    # Extract key=value params, rest is confet args
    confet_args="" name="" scale=720 crf=24
    for word in $entry; do
        case "$word" in
            name=*)  name="${word#name=}" ;;
            scale=*) scale="${word#scale=}" ;;
            crf=*)   crf="${word#crf=}" ;;
            *)       confet_args+="$word " ;;
        esac
    done
    confet_args="${confet_args% }"
    [[ -z "$name" ]] && name="${confet_args%% *}"

    out="$OUTDIR/$name.mp4"
    echo "Recording: $name (confet $confet_args) [scale=${scale}p crf=$crf]"

    for i in 3 2 1; do
        printf "  %d...\n" "$i"
        sleep 1
    done

    raw="$OUTDIR/.raw-$name.mp4"

    wf-recorder -f "$raw" &
    rec_pid=$!
    sleep 0.3

    # shellcheck disable=SC2086
    "$CONFET" $confet_args

    kill "$rec_pid" 2>/dev/null || true
    wait "$rec_pid" 2>/dev/null || true

    ffmpeg -y -i "$raw" -vf "scale=-2:$scale" -c:v libx264 -crf "$crf" -preset slow -an "$out" 2>/dev/null
    rm "$raw"

    echo "  saved $out ($(du -h "$out" | cut -f1))"
    sleep 0.5
done

echo "Done. Recordings in $OUTDIR"
notify-send "Done" "Done recording for confet"
