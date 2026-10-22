#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.."
version="${1:?Usage: package-linux.sh VERSION [--portable-only]}"
mode="${2:-}"
[[ "$mode" = "" || "$mode" = --portable-only ]] || { echo "Unknown packaging option: $mode" >&2; exit 1; }
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "Expected numeric release version" >&2; exit 1; }
test -f target/release/remain
test -f site/index.html
mkdir -p dist
stage="$(mktemp -d)"
trap 'rm -rf "$stage"' EXIT
portable="$stage/remain-$version-linux-x86_64"
mkdir -p "$portable"
cp target/release/remain LICENSE packaging/remain.svg packaging/linux/install.sh "$portable/"
cp -R site "$portable/guide"
chmod 755 "$portable/remain" "$portable/install.sh"
tar -czf "dist/remain-$version-linux-x86_64.tar.gz" -C "$stage" "remain-$version-linux-x86_64"
if [[ "$mode" = --portable-only ]]; then exit 0; fi
deb="$stage/deb"
mkdir -p "$deb/DEBIAN" "$deb/usr/bin" "$deb/usr/share/applications" "$deb/usr/share/icons/hicolor/scalable/apps" "$deb/usr/share/doc/remain"
cp target/release/remain "$deb/usr/bin/remain"
chmod 755 "$deb/usr/bin/remain"
cp packaging/linux/remain.desktop "$deb/usr/share/applications/"
cp packaging/remain.svg "$deb/usr/share/icons/hicolor/scalable/apps/"
cp LICENSE "$deb/usr/share/doc/remain/copyright"
cp -R site "$deb/usr/share/doc/remain/guide"
cat > "$deb/DEBIAN/control" <<EOF
Package: remain
Version: $version
Section: science
Priority: optional
Architecture: amd64
Maintainer: REMAIN developers
Depends: libc6 (>= 2.35), libgcc-s1, libx11-6, libxi6, libxkbcommon0, libwayland-client0, libegl1, libgl1, xdg-desktop-portal | zenity
Description: Offline project report manager
 Assemble project PDFs and aggregate configurable Excel indicators.
EOF
dpkg-deb --root-owner-group --build "$deb" "dist/remain_${version}_amd64.deb"
