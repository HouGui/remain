# Understand the output

Each export creates a new folder. Existing folders are never overwritten.
The GUI uses `remain-report`, then `remain-report-2` and so on.

| File | Purpose |
| --- | --- |
| `index.html` | Standalone, printable submission status and results with report links |
| `report.pdf` | Contents pages followed by included project reports in [configured order](#report-order) |
| `projects/ID.pdf` | Each included project's assembled report |
| `groups/ID.pdf` | Contents and reports for each group with roster projects |
| `categories/ID.pdf` | Contents and reports for each category with roster projects |
| `annex.xlsx` | When KPIs are configured: submission status, summaries, and observations |
| `summary.json` | Completeness, submission checklist, results, and source-row provenance |
| `configuration.json` | Configuration snapshot used for this export |
| `project-list.json` | Project metadata snapshot, including projects with missing submissions |

Open `index.html` in a browser. It works offline and has print styling. Keep the
bundle together when moving it so its relative links remain valid.
The configuration snapshot documents the original inputs; it is not a new
workspace, because those inputs are not copied into the bundle.
Discovered source paths and missing-file patterns are relative to the workspace
and use forward slashes (`/`) on every platform, including Windows.

## Report order

The central PDF starts with a contents list of included project names and IDs,
with their starting page numbers. Numbers are absolute PDF page positions,
including all contents pages, rather than numbers printed in source documents.
Long names wrap and large lists continue onto additional contents pages.
Group and category PDFs get their own contents and page numbers. Individual
project PDFs contain only their submitted parts.

After the contents, reports follow group order, roster order within each group,
and document-definition order within each project. Multiple matches for a pattern
sort lexically by filename. Projects without a group follow in roster order.
Category PDFs use that same order. Categories and parent links do not change the
central reading order.

Groups and categories without any roster projects have no PDF. A scope with
projects but no complete submissions gets a contents/status PDF explaining what
is missing, provided another project permits the overall export to proceed.

## Partial reports and coverage

When required submissions are missing, PDF front matter and the HTML summary
show **Incomplete report**, explain the exclusions, and list missing projects
and files. Only complete projects contribute report pages and KPI results.
Optional documents are listed separately in the submission checklist; their
absence does not mark the report incomplete. If KPIs are collected, PDF front
matter, HTML, and the Excel submission sheet show included workbook counts
against expected projects, so optional KPI coverage is visible.

Export requires at least one complete project. See the
[missing-submission rules](workspaces.md#missing-submissions) for details.

## Excel workbook

Worksheet names are stable across languages for integrations. Built-in captions
use the report language; configured names remain as entered. Each KPI total is
followed by its class columns, labeled **KPI · Class**, in configured order.

- **Submissions** is the opening sheet: completeness, included/expected project
  counts, KPI coverage, and required/optional files missing for every project.
- **Summary** contains the overall result and one row for each group.
- **Categories** contains one row per category.
- **Projects** contains project/group/category/parent identifiers, observation and page
  counts, and each project's KPIs.
- **Observations** contains project/group/category identifiers, original annex path,
  original one-based Excel row, and normalized numeric values.

Projects without a group or category are combined in that classification's
**Unassigned** summary row. They still have individual PDFs and appear in the
central PDF; there is no separate Unassigned PDF. Blank classification cells in
Excel mean unassigned.

The workbook contains calculated values, not live formulas. Editing it does not
update other sheets or the PDF. Correct the source annex and export a new bundle.
The [aggregation reference](configuration.md#aggregations-and-missing-values)
defines how totals, means, and missing values are calculated.

## JSON results

`complete` indicates whether every project supplied its required documents.
`submissions` lists discovered workspace-relative paths and missing required or
optional files for every roster project. `projects` contains only included
projects; `has_kpis` distinguishes a submitted workbook from its absence.
Group/category summaries give both included `projects` and `expected_projects`.

`metric_ids` defines the order of every numeric `values` vector and the
`overall` vector. Totals use the KPI ID; classes use `kpi_id.class_id`, for example
`publications.conference`. In observations, a class column is `null` when the row
does not belong to that class or its numeric value was skipped.
`projects`, `groups`, and `categories` identify their scope using stable IDs.
An unassigned classification summary has `id: null`; project membership is also
`null` when absent.
`observations` records the originating project and Excel row. Use `project-list.json` for project names, `configuration.json` for indicator
labels and units, and `submissions` for discovered source filenames.

## Validation and consistency

Export rediscovers files and validates inputs for complete submissions.
The desktop also reloads saved settings and the roster before each run.
The Rust `Workspace` object keeps its settings/roster snapshot; call
`Workspace::load` again to pick up external metadata edits. Avoid editing submissions while
analysis or export is running. Ordinary write failures remove incomplete output;
a power loss or forced termination may leave a partial folder. Wait until the
GUI reports completion before using the bundle. Discard incomplete exports and
export again.
