#!/usr/bin/env bash
# Exercise a real portable package without touching the user's installation.
set -euo pipefail
archive="${1:?Usage: test-linux-installer.sh ARCHIVE}"
test -f "$archive"
stage="$(mktemp -d)"
trap 'rm -rf "$stage"' EXIT
tar -xzf "$archive" -C "$stage"
source_dir="$(find "$stage" -mindepth 1 -maxdepth 1 -type d -name 'remain-*-linux-x86_64')"
test -f "$source_dir/install.sh"
export REMAIN_INSTALL_PREFIX="$stage/user prefix"
mkdir -p "$REMAIN_INSTALL_PREFIX/bin" "$stage/workspace"
printf 'keep this unrelated file\n' > "$REMAIN_INSTALL_PREFIX/bin/remain"
printf 'keep report data\n' > "$stage/workspace/report.pdf"
if sh "$source_dir/install.sh"; then
    echo "Installer replaced an unrelated executable" >&2
    exit 1
fi
test "$(cat "$REMAIN_INSTALL_PREFIX/bin/remain")" = "keep this unrelated file"
rm "$REMAIN_INSTALL_PREFIX/bin/remain"
sh "$source_dir/install.sh"
test -x "$REMAIN_INSTALL_PREFIX/bin/remain"
test -f "$REMAIN_INSTALL_PREFIX/share/remain/guide/index.html"
cmp "$source_dir/remain" "$REMAIN_INSTALL_PREFIX/bin/remain"
test -f "$REMAIN_INSTALL_PREFIX/share/applications/remain.desktop"
cmp "$source_dir/remain.svg" "$REMAIN_INSTALL_PREFIX/share/icons/hicolor/scalable/apps/remain.svg"
grep -qx 'Icon=remain' "$REMAIN_INSTALL_PREFIX/share/applications/remain.desktop"
grep -qx 'StartupWMClass=remain' "$REMAIN_INSTALL_PREFIX/share/applications/remain.desktop"
sh "$source_dir/install.sh" # Upgrade the same installation.
sh "$source_dir/install.sh" --uninstall
test ! -e "$REMAIN_INSTALL_PREFIX/bin/remain"
test ! -L "$REMAIN_INSTALL_PREFIX/bin/remain"
test ! -e "$REMAIN_INSTALL_PREFIX/share/remain"
test ! -e "$REMAIN_INSTALL_PREFIX/share/applications/remain.desktop"
test ! -e "$REMAIN_INSTALL_PREFIX/share/icons/hicolor/scalable/apps/remain.svg"
test "$(cat "$stage/workspace/report.pdf")" = "keep report data"
echo "Portable installation, upgrade, overwrite protection, and uninstall passed."
