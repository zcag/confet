#!/bin/env bash

# Record demo videos by reading <!-- record: args --> markers from README.md.
# Requires: wf-recorder, ffmpeg, a running Wayland compositor
#
# Marker format: <!-- record: <confet-args> [name=N] [scale=N] [crf=N] [fps=N] -->
#   name=   output filename (default: first confet arg)
#   scale=  vertical resolution for mp4 (default: 720)
#   crf=    h264 quality, lower=better (default: 24)
#   fps=    gif framerate (default: 15)
#
# Outputs both .mp4 and .gif for each entry.
# GitHub README uses .gif (GitHub strips <video> tags).

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

for i in 3 2 1; do
    printf "  %d...\n" "$i"
    sleep 1
done


for entry in "${entries[@]}"; do
    # Extract key=value params, rest is confet args
    confet_args="" name="" scale=720 crf=24 fps=15
    for word in $entry; do
        case "$word" in
            name=*)  name="${word#name=}" ;;
            scale=*) scale="${word#scale=}" ;;
            crf=*)   crf="${word#crf=}" ;;
            fps=*)   fps="${word#fps=}" ;;
            *)       confet_args+="$word " ;;
        esac
    done
    confet_args="${confet_args% }"
    [[ -z "$name" ]] && name="${confet_args%% *}"

    mp4="$OUTDIR/$name.mp4"
    gif="$OUTDIR/$name.gif"
    echo "Recording: $name (confet $confet_args) [scale=${scale}p crf=$crf fps=$fps]"

    raw="$OUTDIR/.raw-$name.mp4"

    wf-recorder -f "$raw" &
    rec_pid=$!
    sleep 0.3

    # shellcheck disable=SC2086
    "$CONFET" $confet_args

    kill "$rec_pid" 2>/dev/null || true
    wait "$rec_pid" 2>/dev/null || true

    # mp4
    ffmpeg -y -i "$raw" -vf "scale=-2:$scale" -c:v libx264 -crf "$crf" -preset slow -an "$mp4" 2>/dev/null

    # gif (palettegen for quality)
    ffmpeg -y -i "$raw" -vf "fps=$fps,scale=-2:$scale:flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse" "$gif" 2>/dev/null

    rm "$raw"

    echo "  mp4: $(du -h "$mp4" | cut -f1)  gif: $(du -h "$gif" | cut -f1)"
    sleep 0.5
done

echo "Done. Recordings in $OUTDIR"
notify-send "Done" "Done recording for confet"
