# Release and deploy

The repository includes GitHub Actions for checks, native packages, draft
releases, and optional documentation hosting. Packaging scripts also run locally,
so another forge or CI system can invoke them without changing the application.

Set repository Actions variable `REMAIN_DOCS_URL` to the HTTPS address of your
published documentation before building a release. Locally, set the environment
variable of the same name before `cargo build`. **Help → About REMAIN** uses
this link. Without an address, the bundled guide remains available.

## Automated release workflow

1. Run the [required checks](development.md#required-checks) and review the user guide.
2. Set the release version in the root `Cargo.toml` and refresh `Cargo.lock`.
    Packaging reads that version; there is no separate installer version to edit.
3. Push a matching `vMAJOR.MINOR.PATCH` tag. Alternatively, run **Release packages**
    manually to produce artifacts without creating a release.
4. The workflow tests and builds Linux x86-64, Windows x86-64, and macOS Apple
    silicon packages on native runners. The bundled documentation is built once
    and shared by all packages.
5. For a tag, it collects artifacts, calculates SHA-256 checksums, and creates a
    **draft** release. The tag must match the Cargo version.
6. Download and test installation, launch, demo export, upgrade, and uninstall on
    each supported platform. Review signing requirements below, then publish the draft.

The workflow also checks portable Linux installation, upgrade, and uninstall,
and Windows unattended installation, icon/shortcut presence, and uninstall.
The [installation guide](installation.md) lists package names and system requirements.
Rust and Python dependencies are cached between runs. Release packages share
one documentation artifact, and compressed installers are uploaded without
another compression pass. Caches improve repeat builds; a cold build still
compiles the full dependency graph.

Use the [GitHub artifact interface](https://docs.github.com/en/actions/how-tos/manage-workflow-runs/download-workflow-artifacts)
for manual workflow builds. End-user installation instructions are maintained
only in [Installation](installation.md).

## Local packaging

Build on the target OS. The following examples use version `0.1.0`; pass the
actual workspace version when releasing.

Build the bundled user guide before packaging: install the pinned documentation
tool as described in [Development](development.md#documentation-website), then
run `bash scripts/build-docs.sh`. CI, packages, and the hosted website use this
same build. Each package includes the offline guide and Rust API reference.

The application logo comes from `packaging/remain.svg`. Its committed PNG, ICO,
and ICNS variants supply the window/About logo, Windows executable and installer
icons, and macOS app icon. Linux installs the SVG with its desktop entry.
After editing the SVG, install Inkscape and Pillow, then run
`python scripts/generate-icons.py` and commit the regenerated icons.

### Linux packages

Build on Ubuntu 22.04 for the documented glibc baseline:

```sh
cargo build --release --locked -p remain-app
bash scripts/package-linux.sh 0.1.0
```

`dpkg-deb` builds the Debian package, including its desktop entry, SVG icon, and
the [runtime dependencies](installation.md#portable-archive).

The portable archive uses the same binary. Its optional installer supports a
`REMAIN_INSTALL_PREFIX` environment variable for packaging tests and custom
user locations; by default it uses `~/.local`. It never installs report data.
Building on a newer distribution can require a newer glibc version; test the
documented baseline before advertising compatibility.
On a non-Debian build machine, append `--portable-only` to the packaging command
to create just the archive without requiring `dpkg-deb`. Test a built archive with
`bash scripts/test-linux-installer.sh dist/remain-0.1.0-linux-x86_64.tar.gz`.

### Windows installer

Install [Inno Setup 6](https://jrsoftware.org/isinfo.php), then use PowerShell:

```powershell
$env:RUSTFLAGS = "-C target-feature=+crt-static"
cargo build --release --locked -p remain-app
./scripts/package-windows.ps1 -Version 0.1.0
```

The static C runtime avoids requiring a separate MSVC runtime installation.
The Inno Setup definition provides an English, German, and French wizard, current-user
installation, Start-menu entry, optional desktop shortcut, upgrades, and uninstall.
Keep its `AppId` stable across releases.
The installation smoke test is available as
`./scripts/test-windows-installer.ps1 -Installer PATH_TO_SETUP_EXE`.
Run it only in CI or a disposable Windows account because it registers and removes
the application's Start-menu shortcut.
[PrivilegesRequired=lowest](https://jrsoftware.org/ishelp/topic_setup_privilegesrequired.htm)
is deliberate: application files are installed to a user-writable location.

For a trusted publisher identity, sign `remain.exe` with your organization's
Authenticode certificate before packaging and the setup executable after
packaging, with SHA-256 and an approved timestamp server. Certificate provisioning
is distributor-specific; the default workflow has no signing credentials.
Recalculate release checksums after signing.

### macOS signing and notarization

On an Apple silicon Mac:

```sh
export MACOSX_DEPLOYMENT_TARGET=12.0
cargo build --release --locked -p remain-app
bash scripts/package-macos.sh 0.1.0
```

The script creates a native `REMAIN.app`, verifies its code signature, and places
it in a compressed disk image with an Applications shortcut. Without credentials
it uses an ad hoc signature suitable for local testing.

For public distribution, provision a Developer ID Application certificate in the
build machine's keychain and store notarization credentials in a keychain profile:

```sh
export REMAIN_SIGN_IDENTITY="Developer ID Application: Your Organization (TEAMID)"
export REMAIN_NOTARY_PROFILE="remain-notary"
bash scripts/package-macos.sh 0.1.0
```

The script signs with hardened runtime and a timestamp, submits the DMG with
`notarytool`, waits for the result, and staples and validates the ticket.
Follow [Apple's notarization documentation](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution)
when provisioning credentials. Never commit certificates, private keys, or
passwords. The default hosted workflow creates development packages; run the
script in a provisioned release environment for notarized distribution.

## Documentation hosting

**Checks** builds a strict Zensical site and Rustdoc into one `documentation`
artifact on pushes and pull requests. This validates the site without publishing.

### Activate GitHub Pages when ready

The **Publish documentation** workflow is prepared but disabled until the
repository is public and the Actions variable `REMAIN_PAGES_ENABLED` is `true`.
It never enables Pages or changes repository visibility itself.

1. When ready to launch, make the repository public.
2. In **Settings → Pages → Build and deployment**, choose **GitHub Actions**.
3. In **Settings → Environments → github-pages**, restrict deployments to the
   default branch; add required reviewers if desired.
4. In **Settings → Secrets and variables → Actions → Variables**, create
   `REMAIN_PAGES_ENABLED` with value `true`.
5. Run **Actions → Publish documentation → Run workflow** on the default branch.
   Subsequent documentation or core changes on `main` deploy automatically.
6. Open the URL shown by the deployment. For `HouGui/remain`, the expected address
   is `https://hougui.github.io/remain/`. Set `REMAIN_DOCS_URL` to the confirmed
   address before building release packages so **Help → About REMAIN** links there.

Only the generated `site/` is published. The workflow requires the default
branch, uses the `github-pages` environment, and skips private repositories even
if the variable is set. Set `REMAIN_PAGES_ENABLED` to `false` to stop future
deployments; that does not remove an already published site. If the default
branch is renamed, update the workflow's `push.branches` filter too.

See GitHub's [Pages workflow documentation](https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages)
for repository-side setup.

### Other hosts

For another host, upload `site/` after the
[documentation build](development.md#documentation-website). Preserve the
directory structure, including `api/`. No server-side application is needed.
