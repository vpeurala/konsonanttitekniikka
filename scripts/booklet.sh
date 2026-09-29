#!/usr/bin/env bash
# Builds the user instruction booklet: the game's own program writes it as
# HTML, and headless Chrome prints that to a PDF, on A4 pages.
#
#   scripts/booklet.sh [OUTPUT_DIR]     (default: target/booklet)
#
# Set CHROME to a Chrome or Chromium executable if it isn't found. Drawing
# the pictures opens a window briefly, so on a machine without a display run
# this under `xvfb-run -a`.
set -euo pipefail
cd "$(dirname "$0")/.."

out="${1:-target/booklet}"
mkdir -p "$out"
out="$(cd "$out" && pwd)"

if [ -z "${CHROME:-}" ]; then
    for candidate in \
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
        google-chrome google-chrome-stable chromium chromium-browser; do
        if command -v "$candidate" >/dev/null 2>&1; then
            CHROME="$candidate"
            break
        fi
    done
fi
if [ -z "${CHROME:-}" ]; then
    echo "Chrome or Chromium is needed to make the PDF; set CHROME to its path." >&2
    exit 1
fi

cargo run --release -- --render-booklet "$out/opas.html"
# On CI, Chrome runs in a container where its own sandbox can't start.
"$CHROME" --headless --disable-gpu --no-pdf-header-footer ${CI:+--no-sandbox} \
    --print-to-pdf="$out/lukuloitsu-opas.pdf" "file://$out/opas.html" 2>/dev/null
echo "wrote $out/lukuloitsu-opas.pdf"
