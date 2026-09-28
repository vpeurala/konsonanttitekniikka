#!/bin/bash
# Installs cargo-quad-apk, the APK builder used by scripts/android.sh, from
# the GitHub commit known to work, with scripts/cargo-quad-apk.patch
# applied. The crates.io release is too old for Rust 2024, and the GitHub
# version can't add new Android resources (like the launcher icon) without
# the patch.
set -euo pipefail

REPO=https://github.com/not-fl3/cargo-quad-apk
REV=d411c8fe1c08339e46d7dc6b40ac04ea5e3a01b6
PATCH="$(cd "$(dirname "$0")" && pwd)/cargo-quad-apk.patch"

export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH"
SRC="$(mktemp -d)"
trap 'rm -rf "$SRC"' EXIT

git clone --quiet "$REPO" "$SRC"
git -C "$SRC" checkout --quiet "$REV"
git -C "$SRC" apply "$PATCH"
cargo install --path "$SRC" --force
