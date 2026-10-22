# Working on REMAIN

REMAIN is organization-neutral reporting software for research, education, and
industry. Preserve the shared model; never introduce institution-specific paths,
identifiers, languages, or KPI definitions into application logic.

## Sources of truth

- Documentation: `docs/`.
    - User behavior, configuration, and installation: the relevant user-guide page.
    - Architecture, developer commands, and validation: `docs/development.md`.
    - Packaging and release procedure: `docs/deployment.md`.
- Public Rust API contracts: Rustdoc beside the public items.
- Translations: `locales/`; built-in language registry: `remain-core/locale`.
- Package versions: root `Cargo.toml` and committed `Cargo.lock`.

README provides an overview and repository map. Update each subject in its
owning document and link from other pages. Use nested lists for grouped setup
steps and directory maps; use tables for field and platform comparisons.
Indent nested Markdown lists by four spaces for GitHub and Zensical compatibility.

## Change boundaries

- Follow the architecture and required checks in the developer guide.
- Keep processing in `remain-core` and interaction in `remain-app`; do not
  duplicate calculations or validation in the GUI.
- Maintain explicit missing-data semantics, stable ordering, portable paths,
  and protection against overwriting inputs or existing output directories.
- Keep identifiers independent of configured display names. Group and category
  membership are independent and optional; free projects must remain visible.
- Keep fixtures synthetic and reproducible through the Rust demo generator.
- Preserve the MIT license attribution. Do not commit real submissions, branding,
  secrets, generated build trees, or platform-specific absolute paths.
