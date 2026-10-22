#!/bin/sh
# Optional per-user installer for the portable Linux archive.
set -eu
prefix="${REMAIN_INSTALL_PREFIX:-$HOME/.local}"
app_dir="$prefix/share/remain"
bin_path="$prefix/bin/remain"
desktop_path="$prefix/share/applications/remain.desktop"
icon_path="$prefix/share/icons/hicolor/scalable/apps/remain.svg"
if [ "${1:-}" = "--uninstall" ]; then
    if [ -f "$app_dir/.remain-install" ]; then
        if [ -L "$bin_path" ] && [ "$(readlink "$bin_path")" = "$app_dir/remain" ]; then
            rm "$bin_path"
        fi
        rm -f "$desktop_path" "$icon_path"
        rm -rf "$app_dir"
        echo "REMAIN removed. Workspace data was not changed."
    fi
    exit 0
fi
source_dir="$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)"
if [ -e "$app_dir" ] && [ ! -f "$app_dir/.remain-install" ]; then
    echo "Refusing to replace an unrelated directory: $app_dir" >&2
    exit 1
fi
if [ -e "$bin_path" ] || [ -L "$bin_path" ]; then
    if [ ! -L "$bin_path" ] || [ "$(readlink "$bin_path")" != "$app_dir/remain" ]; then
        echo "Refusing to replace an unrelated executable: $bin_path" >&2
        exit 1
    fi
fi
mkdir -p "$app_dir" "$prefix/bin" "$prefix/share/applications" "$prefix/share/icons/hicolor/scalable/apps"
cp "$source_dir/remain" "$app_dir/remain"
cp "$source_dir/LICENSE" "$app_dir/LICENSE"
if [ -d "$app_dir/guide" ]; then rm -rf "$app_dir/guide"; fi
cp -R "$source_dir/guide" "$app_dir/guide"
chmod 755 "$app_dir/remain"
touch "$app_dir/.remain-install"
ln -sfn "$app_dir/remain" "$bin_path"
cp "$source_dir/remain.svg" "$icon_path"
# Desktop entries need an absolute path, escaped for their Exec syntax.
# shellcheck disable=SC2016 # The substitutions deliberately match literal shell characters.
escaped_app="$(printf '%s' "$app_dir/remain" | sed 's/\\/\\\\/g; s/"/\\"/g; s/`/\\`/g; s/\$/\\$/g; s/%/%%/g')"
{
    printf '[Desktop Entry]\nType=Application\nName=REMAIN\n'
    printf 'Exec="%s"\nIcon=remain\nTerminal=false\nCategories=Office;Science;\nStartupWMClass=remain\n' "$escaped_app"
} > "$desktop_path"
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$prefix/share/applications"
fi
echo "Installed REMAIN. Open it from your applications menu."
