#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.."
version="${1:?Usage: package-macos.sh VERSION}"
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "Expected numeric release version" >&2; exit 1; }
test -f target/release/remain
lipo target/release/remain -verify_arch arm64
test -f site/index.html
mkdir -p dist
stage="$(mktemp -d)"
trap 'rm -rf "$stage"' EXIT
app="$stage/REMAIN.app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
cp target/release/remain "$app/Contents/MacOS/remain"
chmod 755 "$app/Contents/MacOS/remain"
cp LICENSE "$app/Contents/Resources/"
cp packaging/remain.icns "$app/Contents/Resources/"
cp -R site "$app/Contents/Resources/guide"
cat > "$app/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleName</key><string>REMAIN</string>
<key>CFBundleDisplayName</key><string>REMAIN</string>
<key>CFBundleIdentifier</key><string>org.remain.desktop</string>
<key>CFBundleExecutable</key><string>remain</string>
<key>CFBundleIconFile</key><string>remain.icns</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleShortVersionString</key><string>$version</string>
<key>CFBundleVersion</key><string>$version</string>
<key>LSMinimumSystemVersion</key><string>12.0</string>
<key>NSHighResolutionCapable</key><true/>
</dict></plist>
EOF
if [ -n "${REMAIN_SIGN_IDENTITY:-}" ]; then
    codesign --force --options runtime --timestamp --sign "$REMAIN_SIGN_IDENTITY" "$app"
else
    codesign --force --sign - "$app"
fi
codesign --verify --strict "$app"
ln -s /Applications "$stage/Applications"
dmg="dist/remain-$version-macos-aarch64.dmg"
hdiutil create -volname REMAIN -srcfolder "$stage" -ov -format UDZO "$dmg"
if [ -n "${REMAIN_NOTARY_PROFILE:-}" ]; then
    test -n "${REMAIN_SIGN_IDENTITY:-}" || { echo "Notarization requires a Developer ID signing identity" >&2; exit 1; }
    xcrun notarytool submit "$dmg" --keychain-profile "$REMAIN_NOTARY_PROFILE" --wait
    xcrun stapler staple "$dmg"
    xcrun stapler validate "$dmg"
fi
