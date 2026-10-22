# Install REMAIN

Download the package for your operating system from
[GitHub Releases](https://github.com/HouGui/remain/releases). It includes the
application, offline user guide, and Rust API reference. Choose an asset from
this table; GitHub's **Source code** archives do not contain an installed application.

| System | Processor | Release asset (`VERSION` is the release number) |
| --- | --- | --- |
| Windows 10 or later | x86-64 | `remain-VERSION-windows-x86_64-setup.exe` |
| Linux with glibc 2.35 or newer | x86-64 | `remain_VERSION_amd64.deb` or `remain-VERSION-linux-x86_64.tar.gz` |
| macOS 12 or later | Apple silicon | `remain-VERSION-macos-aarch64.dmg` |

Other systems require a [source build](development.md#build-and-run).
Keep workspaces and exported reports in a separate data folder so upgrades and
uninstallation leave them intact. An Excel-compatible editor is needed only to
prepare or edit KPI workbooks.

## Windows

1. Run the installer and choose English, German, or French.
2. Accept the installation directory or choose another user-writable location.
3. Optionally create a desktop shortcut.
4. Finish the wizard and launch REMAIN from the Start menu.

The default is a per-user installation under `%LOCALAPPDATA%\Programs\REMAIN`.
Administrator permission is not required. Upgrades use the same application
identifier and replace application files. To uninstall, use **Settings → Apps → REMAIN**.

The automated Windows installer is unsigned and may show a publisher warning.
Use the linked GitHub release as the download source. If your organization's
policy requires a signed package, contact your IT team; the
[release guide](deployment.md#windows-installer) covers signing.

## Linux

Use a graphical desktop with OpenGL, on Wayland or X11. Ubuntu 22.04 and newer
meet the packaged glibc baseline.

### Debian or Ubuntu package

Open the `.deb` with your software installer, or install it from a terminal:

```sh
sudo apt install ./remain_VERSION_amd64.deb
```

Replace `VERSION` with the downloaded version. Launch **REMAIN** from the
application menu. Remove it through the software manager or `sudo apt remove remain`.
The package manager resolves required desktop libraries.

### Portable archive

Extract the archive, open a terminal in the extracted folder, and run `./remain`.
To add REMAIN and its icon to your application menu, run the included installer
from that same folder:

```sh
sh install.sh
```

This installs the executable under `~/.local/bin` and a desktop entry under
`~/.local/share/applications`. It refuses to overwrite an unrelated executable.
The portable archive needs the same desktop libraries as the Debian package:
`libgcc-s1`, `libx11-6`, `libxi6`, `libxkbcommon0`, `libwayland-client0`, `libegl1`,
and `libgl1` (names on Debian/Ubuntu). File dialogs also need your desktop's XDG
portal backend or `zenity`.

To upgrade, quit REMAIN and run `sh install.sh` from the newly extracted release.
Keep a copy of the archive's installer; run `sh install.sh --uninstall` to remove
the user installation.

## macOS

The macOS package is for Apple silicon only.

1. Open the disk image.
2. Drag **REMAIN.app** onto **Applications**.
3. Eject the disk image and open REMAIN from Applications.

To upgrade, quit REMAIN and replace the existing application. To uninstall, move
REMAIN.app to the Trash.

Automated macOS builds are ad hoc signed, not Apple-notarized. If macOS blocks
the app as an unidentified developer and you trust the downloaded release,
follow [Apple's instructions for opening it](https://support.apple.com/en-us/102445).
On current macOS versions, try opening the app, then use **System Settings →
Privacy & Security → Open Anyway**. Managed computers may require IT approval.
Maintainers can [sign and notarize packages](deployment.md#macos-signing-and-notarization)
with their own Developer ID credentials.

## After installation

Start with [the fictional demo](tutorials.md#try-the-demo).
