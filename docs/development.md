# Contribute

## Architecture

The Cargo workspace has two crates:

| Module | Responsibility |
| --- | --- |
| `remain-core/config` | Serializable, versioned model and configuration validation |
| `remain-core/projects`, `submissions` | CSV/JSON roster import and deterministic file discovery |
| `remain-core/workspace` | Rooted input resolution and workflow coordination |
| `remain-core/excel` | Calamine import and rust_xlsxwriter export |
| `remain-core/analysis` | Polars numeric aggregation over typed observations |
| `remain-core/pdf` | PDF validation, contents pagination, embedded TrueType text, and assembly through lopdf |
| `remain-core/export` | Bundle staging, publication, and escaped HTML rendering |
| `remain-core/locale` | Shared language catalogs and number presentation |
| `remain-core/demo` | Synthetic fixtures generated entirely in Rust |
| `remain-app/app`, `jobs` | Desktop state and background workers |
| `remain-app/views`, `editor` | egui display and configuration forms |
| `remain-app/appearance` | Desktop theme, bundled fonts, and local display preferences |

The GUI depends on the core. File adapters produce typed observations;
aggregation has no file or UI knowledge. Organization-specific terminology
belongs in workspace configuration. Add dependencies only for responsibilities
the existing libraries cannot serve.

`Config::indicators()` defines the shared result-column order: each KPI total
followed by its classes. The Excel reader partitions observations into these
columns; analysis, GUI views, and exports use that same ordering.

An analysis holds observations and parsed PDFs in memory. Resource usage scales
with input size; processing is designed for ordinary reporting collections, not
unbounded streaming archives. The GUI runs one job at a time on a worker thread;
Polars supplies internal numeric parallelism. Optional core progress callbacks
report coarse phases and the project being read. The GUI forwards these events
through its existing worker channel to a compact, translated status bar.

## Build and run

Install [Rust](https://www.rust-lang.org/tools/install) with Cargo. The workspace
declares Rust 1.94 or newer; use current stable for development. The locked Polars
dependency uses `slice::array_windows`, stabilized in Rust 1.94.
Install the native build tools for your platform:

- Windows
    - MSVC C++ build tools
    - Git Bash for the repository's Bash scripts
- macOS
    - Xcode Command Line Tools
- Debian/Ubuntu
    - Run `bash scripts/install-linux-deps.sh` from the repository root.

Then run the application:

```sh
cargo run --locked -p remain-app
```

For optimized packages use `cargo build --locked --release -p remain-app`.

## Required checks

Run these from the repository root:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --locked
cargo doc --workspace --no-deps --locked
```

The minimum-version CI job reads `rust-version` from the Cargo manifest and checks
both crates, all targets, and all features. When changing dependencies, also run:

```sh
rustup toolchain install 1.94.0 --profile minimal
cargo +1.94.0 check --workspace --all-targets --all-features --locked
```

For script or workflow changes, also run [ShellCheck](https://www.shellcheck.net/)
and [actionlint](https://github.com/rhysd/actionlint):

```sh
shellcheck scripts/*.sh packaging/linux/install.sh
actionlint
```

CI tests Linux, Windows, and macOS Apple silicon. Formatting, shell checks, and
workspace Rustdoc run once on Linux. New commits cancel older checks on the same
branch. Shared actions in `.github/actions/` own cached Rust setup and the strict
documentation build; platform packaging and installer tests live in `scripts/`.

Unit tests cover numeric and locale semantics, editor persistence, and UI state.
Integration tests generate actual XLSX/PDF files to check import, validation,
hierarchy, ordering, inheritance, output round trips, and publication failures.
Submission tests cover CSV/JSON imports, PDF-only workspaces, optional and required
workbooks, partial exports, ambiguous patterns, and multi-page Unicode contents.
Tests use temporary directories and do not require a display or a network after
dependencies are fetched. The core's Rustdoc example is compiled by `cargo test`.

Run the disk-backed scalability test separately (Linux CI also runs it):

```sh
cargo test --locked -p remain-core --test scalability -- --ignored --nocapture
```

It analyzes and exports 1,000 projects with real PDFs/XLSX files, 20 groups,
10 categories, unassigned projects, and KPI classes; then reopens the outputs
and checks counts and aggregates. It reports timings without a hardware-dependent pass
threshold. Set `REMAIN_SCALE_PROJECTS` to run a different size (100–100000).
The regular suite also validates a 10,000-project parent chain without recursion.

For visual checks, compile the desktop preview:

```sh
cargo build --locked -p remain-app --example gui_preview --features preview
EFRAME_SCREENSHOT_TO=/tmp/remain.png target/debug/examples/gui_preview examples/demo-en/workspace.json
```

This runs on a graphical display and exits after capturing a PNG. On Windows,
set the environment variable in PowerShell and use the `.exe` suffix. Add `editor`
or `small` as the final argument to inspect the form or a narrow window.
Use `metrics` to inspect the KPI form, `documents` for input rules, and `roster`
for the project-list editor. The source lists additional category,
chart, and About preview modes. Use `help` to check the User guide window.
Add `dark` for the dark theme or a font ID (`noto-sans`, `lato`,
`atkinson-hyperlegible`) for a bundled font. Use `appearance` to open the
appearance menu or `error` for an error banner.

The preview resets appearance on each run and uses temporary preference storage
so it does not change the desktop application's saved choices.
If a Linux capture is blank with the default graphics driver, retry with
`WINIT_UNIX_BACKEND=x11 LIBGL_ALWAYS_SOFTWARE=1` before the preview command.

Inspect all bundled languages, configuration forms, and narrow windows.
Open an exported workbook and render the merged PDF with an independent reader.
A passing parser test alone does not establish visual fidelity.

## API documentation

Rustdoc is the authoritative reference for public Rust types, methods, errors,
and examples. Browse the [Rust API reference](api/index.html), available both
on the documentation website and in the bundled offline guide.
After `cargo doc`, open `target/doc/remain_core/index.html` locally.
The full documentation build places it under `site/api/`.
The library entry points are:

- `Workspace::load` — read configuration from disk
- `Workspace::new` — read the referenced roster using an in-memory `Config`
- `Workspace::with_projects` — use an in-memory configuration and roster
- `Workspace::submissions` — discover files and check presence without parsing contents
- `Workspace::analyze` — validate inputs and aggregate indicators
- `Workspace::export` — create a report bundle
- `Workspace::analyze_with_progress` / `export_with_progress` — the same operations
  with phase notifications for integrations

## Documentation website

The user guide is built with [Zensical](https://zensical.org/docs/). Python is
needed only by this documentation build tool, never by the application:

```sh
python -m venv .venv-docs
# Linux/macOS:
. .venv-docs/bin/activate
# Windows PowerShell instead: .venv-docs\Scripts\Activate.ps1
python -m pip install -r docs/requirements.txt
zensical serve
```

For a distributable HTML site including Rustdoc:

```sh
bash scripts/build-docs.sh
```

The output is `site/`; do not edit generated HTML. See
[Documentation hosting](deployment.md#documentation-hosting) for publishing.

The native Zensical palette toggle in `zensical.toml` supplies light/dark mode
and browser persistence. After theme changes, check both palettes, persistence
after reload, and mobile navigation in the built site.
See [Zensical's palette documentation](https://zensical.org/docs/setup/colors/).

## Bundled fonts

Font files and their unmodified SIL Open Font License notices live in
`assets/fonts/`. The `appearance::FONTS` registry embeds both
bytes and notices in the executable; **About REMAIN** exposes the notices even
when the executable is distributed alone. The core also embeds Noto Sans for PDF
contents pages, independently of the GUI font selection. Rebuild the application after updating
fonts, and keep source revisions and licenses together with the assets.

| Font | Upstream source revision | Files |
| --- | --- | --- |
| Noto Sans Regular | [notofonts/noto-fonts `ffebf8c1`](https://github.com/notofonts/noto-fonts/tree/ffebf8c1ee449e544955a7e813c54f9b73848eac) | `hinted/ttf/NotoSans/NotoSans-Regular.ttf`, `LICENSE` |
| Lato Regular | [google/fonts `45e3d526`](https://github.com/google/fonts/tree/45e3d5269e4a17d823f6d84470bc084dc5ae46f6/ofl/lato) | `Lato-Regular.ttf`, `OFL.txt` |
| Atkinson Hyperlegible Regular | [google/fonts `45e3d526`](https://github.com/google/fonts/tree/45e3d5269e4a17d823f6d84470bc084dc5ae46f6/ofl/atkinsonhyperlegible) | `AtkinsonHyperlegible-Regular.ttf`, `OFL.txt` |

The tests parse the embedded fonts, check interface glyph coverage, and exercise
font switching, theme switching, invalid custom fonts, and saved preference
fallbacks. Saved font IDs are independent of translated labels and registry order.

## Add a language

- Add a catalog under `locales/` with the same keys as `en.json`.
- Register its tag, native name, and decimal convention in `locale::LANGUAGES`.
    - The GUI language selector and regional-tag resolution use this registry.
    - Keep JSON field names and identifiers independent of display translations.
- Add demo text in `demo/configuration.rs`, `demo/classified.rs`, and
  `demo/document.rs`. The generator selects plain names once when creating a
  workspace; these names do not change with the interface or output language.
- Extend export and visual checks, and add a short translated getting-started page.

For organization-specific terminology, use workspace
[translation overrides](configuration.md#languages) instead of changing catalogs.

## Fixtures and changes

Regenerate examples in a new directory to avoid overwriting source data:

```sh
cargo run --locked -p remain-core --example generate_demo -- /tmp/remain-demo-new en
cargo run --locked -p remain-core --example export_demo -- /tmp/remain-demo-new/workspace.json /tmp/remain-report-new
```

Use `de` or `fr` instead of `en` for the other bundled languages. The examples
are synthetic and covered by the project's MIT license. Keep real submissions
outside the checkout, or in its ignored `workspaces/` and `reports/` folders.

Append `--classes` to the generator command for the classified demo. It covers
multiple properties, an unused class, zero, a skipped value, an empty group,
and an unassigned project. Integration tests additionally construct invalid
headers, classes, and formulas to check diagnostics. Language tests combine
English, German, and French PDF parts, vary output languages independently of
Excel source names, and verify that configured names remain unchanged.

Repository conventions:

- Version control
    - Commit `Cargo.lock` and synthetic PDF/XLSX fixtures.
    - `.gitignore` excludes build output, local reports, and temporary files.
- Formatting
    - `.gitattributes` enforces LF text and preserves binary files.
    - `.editorconfig` supplies editing defaults; `cargo fmt` formats Rust.

Use defaults for compatible configuration additions; increment the schema
version for breaking changes. Keep modules focused and add behavior-focused
tests for changed boundaries. Contributions should describe the change and its
validation results.
