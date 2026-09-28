#!/bin/bash
# Builds the web version into target/web, ready to upload as it is to any
# static web host. With --serve, also serves it at http://localhost:8000.
#
# Usage: scripts/web.sh [--serve]
set -euo pipefail

cd "$(dirname "$0")/.."
export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH"

cargo build --release --target wasm32-unknown-unknown

OUT=target/web
rm -rf "$OUT"
mkdir -p "$OUT"
cp web/index.html web/lukuloitsu.js web/mq_js_bundle.js "$OUT/"
cp target/wasm32-unknown-unknown/release/lukuloitsu.wasm "$OUT/"
cp android/res/mipmap-xxxhdpi/ic_launcher.png "$OUT/icon.png"
echo "Built $OUT"

if [[ "${1:-}" == "--serve" ]]; then
    cd "$OUT"
    python3 -m http.server 8000
fi
