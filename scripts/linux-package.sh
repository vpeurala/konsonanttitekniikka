#!/bin/bash
# Builds the game for Linux and packs it with the licenses into
# target/linux/lukuloitsu-linux.tar.gz. The release workflow uses it.
#
# The executable holds the fonts and everything else it draws, so it is one
# file. It loads the system's ALSA, X11 and OpenGL libraries when it runs,
# which every desktop has; it needs glibc 2.35 or newer, since the release
# is built on Ubuntu 22.04.
set -euo pipefail

cd "$(dirname "$0")/.."

cargo build --release

OUT=target/linux
rm -rf "$OUT"
mkdir -p "$OUT/lukuloitsu"
cp target/release/lukuloitsu LICENSE-MIT LICENSE-APACHE \
    assets/fonts/OFL-Nunito.txt assets/fonts/OFL-Fredoka.txt "$OUT/lukuloitsu/"
tar -C "$OUT" -czf "$OUT/lukuloitsu-linux.tar.gz" lukuloitsu
echo "Built $OUT/lukuloitsu-linux.tar.gz"
