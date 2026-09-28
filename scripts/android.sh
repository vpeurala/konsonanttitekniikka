#!/bin/bash
# Builds the game for Android, installs it on the phone connected over USB
# (with USB debugging enabled) and launches it.
#
# Usage: scripts/android.sh [--release]
#
# Needs the Android SDK and NDK in ~/Library/Android/sdk, Java 8 from
# SDKMAN (the APK tool compiles its Java glue for Java 7), and
# `cargo install cargo-quad-apk`.
set -euo pipefail

cd "$(dirname "$0")/.."

export ANDROID_HOME="$HOME/Library/Android/sdk"
export NDK_HOME="$ANDROID_HOME/ndk/27.2.12479018"
export JAVA_HOME="$HOME/.sdkman/candidates/java/8.0.504+1-zulu"
export PATH="$JAVA_HOME/bin:$ANDROID_HOME/platform-tools:/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH"

PACKAGE=fi.villepeurala.konsonanttitekniikka

cargo quad-apk build "$@"

PROFILE=debug
[[ " $* " == *" --release "* ]] && PROFILE=release
APK="target/android-artifacts/$PROFILE/apk/konsonanttitekniikka.apk"

# -d picks the phone connected over USB, even if emulators are listed.
adb -d install -r "$APK"
adb -d shell am start -n "$PACKAGE/.MainActivity"
