#!/bin/bash
# Netlify's build command (see netlify.toml): installs Rust if the build
# machine lacks it, adds the WebAssembly target and builds the web version
# into target/web, which Netlify then publishes.
set -euo pipefail

cd "$(dirname "$0")/.."

if ! command -v rustup >/dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --profile minimal --default-toolchain stable
fi
# shellcheck source=/dev/null
[[ -f "$HOME/.cargo/env" ]] && source "$HOME/.cargo/env"

rustup target add wasm32-unknown-unknown
scripts/web.sh
