#!/bin/bash
# Builds Lukuloitsu.app for macOS, for both Apple silicon and Intel Macs,
# and zips it with the licenses into target/macos/lukuloitsu-macos.zip.
# The release workflow uses it; it works on any Mac with Xcode's tools.
#
# The app is signed only "ad hoc", which Apple silicon requires of every
# program but which tells Gatekeeper nothing, so a downloaded copy is opened
# with a right click the first time (.github/release-notes.md says how).
set -euo pipefail

cd "$(dirname "$0")/.."
export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH"

VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
OUT=target/macos
APP="$OUT/Lukuloitsu.app"

rustup target add aarch64-apple-darwin x86_64-apple-darwin
for target in aarch64-apple-darwin x86_64-apple-darwin; do
    cargo build --release --target "$target"
done

rm -rf "$OUT"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
lipo -create -output "$APP/Contents/MacOS/lukuloitsu" \
    target/aarch64-apple-darwin/release/lukuloitsu \
    target/x86_64-apple-darwin/release/lukuloitsu

# The icon: the iOS one, at the sizes an .icns file wants.
ICONSET="$OUT/Lukuloitsu.iconset"
SOURCE=ios/Assets.xcassets/AppIcon.appiconset/AppIcon.png
mkdir -p "$ICONSET"
for size in 16 32 128 256 512; do
    sips -z "$size" "$size" "$SOURCE" --out "$ICONSET/icon_${size}x${size}.png" >/dev/null
    sips -z "$((size * 2))" "$((size * 2))" "$SOURCE" \
        --out "$ICONSET/icon_${size}x${size}@2x.png" >/dev/null
done
iconutil -c icns "$ICONSET" -o "$APP/Contents/Resources/Lukuloitsu.icns"
rm -rf "$ICONSET"

cat > "$APP/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>lukuloitsu</string>
    <key>CFBundleIdentifier</key>
    <string>fi.lukuloitsu.lukuloitsu</string>
    <key>CFBundleName</key>
    <string>Lukuloitsu</string>
    <key>CFBundleDisplayName</key>
    <string>Lukuloitsu</string>
    <key>CFBundleIconFile</key>
    <string>Lukuloitsu</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>$VERSION</string>
    <key>CFBundleVersion</key>
    <string>$VERSION</string>
    <key>LSApplicationCategoryType</key>
    <string>public.app-category.educational-games</string>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
PLIST

codesign --force --sign - "$APP"

# The licenses travel with the app: it contains the fonts too.
cp LICENSE-MIT LICENSE-APACHE assets/fonts/OFL-Nunito.txt assets/fonts/OFL-Fredoka.txt "$OUT/"
(cd "$OUT" && zip -qry lukuloitsu-macos.zip Lukuloitsu.app LICENSE-MIT LICENSE-APACHE OFL-Nunito.txt OFL-Fredoka.txt)
echo "Built $OUT/lukuloitsu-macos.zip"
