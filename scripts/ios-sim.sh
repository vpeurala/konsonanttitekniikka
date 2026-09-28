#!/bin/bash
# Builds the game for the iOS Simulator, installs it on a simulated iPhone
# and launches it.
#
# Usage: scripts/ios-sim.sh ["Device name"]   (default: iPhone 17)
set -euo pipefail

DEVICE="${1:-iPhone 17}"
TARGET=aarch64-apple-ios-sim
NAME=Konsonanttitekniikka
BUNDLE_ID=fi.villepeurala.konsonanttitekniikka

cd "$(dirname "$0")/.."

cargo build --target "$TARGET"

APP="target/$TARGET/debug/$NAME.app"
rm -rf "$APP"
mkdir -p "$APP"
cp "target/$TARGET/debug/konsonanttitekniikka" "$APP/"
cp ios/Info.plist "$APP/"

# Boot the simulator if it isn't running yet, and show its window if the
# Simulator app is installed.
xcrun simctl boot "$DEVICE" 2>/dev/null || true
open -b com.apple.iphonesimulator 2>/dev/null || true

xcrun simctl install "$DEVICE" "$APP"
xcrun simctl launch --terminate-running-process "$DEVICE" "$BUNDLE_ID"
