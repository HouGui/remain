# Configure REMAIN

Use **Edit configuration** for workspace settings, expected documents, KPIs,
groups, and categories. **Project list** opens the separate roster editor and
CSV/JSON importer. For integrations, edit the UTF-8 files directly. Unknown
configuration fields and unsupported schema versions are rejected to catch typos.

## Minimal configuration

A workspace needs only a project list and one required PDF definition. Save this
as `workspace.json`:

```json
{
  "schema_version": 2,
  "title": "Annual review",
  "locale": "en",
  "period": "2026",
  "projects": "projects.csv",
  "inputs": {
    "directory": "{id}",
    "documents": [
      {"label": "Report", "pattern": "report.pdf", "required": true}
    ]
  }
}
```

With `id,label` and `pilot,Learning pilot` as the two lines of `projects.csv`,
REMAIN expects `pilot/report.pdf`. No Excel file, KPI, group, or category is
required. See [project lists](workspaces.md#project-list) for import formats.

## Workspace fields

| Field | Required | Meaning |
| --- | --- | --- |
| `schema_version` | Yes | Currently `2` |
| `title` | Yes | Nonempty title, used as entered |
| `locale` | Yes | Language for generated captions and number formatting, such as `en`, `de`, or `fr-CA` |
| `period` | Yes | Nonempty descriptive period, not a date filter |
| `projects` | Yes | Relative path to a CSV or JSON project list |
| `inputs.directory` | Yes | Relative project folder with exactly one `{id}` placeholder |
| `inputs.documents` | Yes | Ordered document definitions; at least one required PDF |
| `groups` / `categories` | No | Ordered lists of `id` and `label`; default empty |
| `annex` | With KPI workbook | Worksheet settings: `sheet` and one-based `header_row` (1–1048576) |
| `metrics` | With KPI workbook | Nonempty list of KPI definitions; omit for PDF-only workspaces |
| `contents_font` | No | Relative path to a TrueType `.ttf` for generated contents pages; defaults to bundled Noto Sans |
| `translations` | No | Interface and output caption overrides |

Identifiers use 1–80 ASCII letters, digits, hyphens, or underscores. IDs are
unique within their kind, including case-insensitive uniqueness for portable
output filenames. Windows reserved filenames are rejected. References match
case exactly. The title and every `label` are nonempty plain strings, including
document, project, KPI, and class names. Language-keyed objects are not accepted.

## Expected documents

Every document definition has the same three fields:

| Field | Meaning |
| --- | --- |
| `label` | Display name for the submission checklist |
| `pattern` | Filename ending in `.pdf` or `.xlsx`; `*` matches any sequence of characters |
| `required` | If `true`, a missing match excludes the whole project from analysis and export |

Define as many PDF documents as needed, and at most one KPI workbook. A literal
filename expects that file; a wildcard PDF pattern includes all matching files.
If individual parts are mandatory, give each its own required definition. A
pattern such as `part-*.pdf` only requires **one or more** matching parts.

For example, this `inputs` value expects two required PDFs and an optional workbook:

```json
{
  "directory": "submissions/{id}/reports",
  "documents": [
    {"label": "Narrative", "pattern": "report.pdf", "required": true},
    {"label": "Declaration", "pattern": "declaration.pdf", "required": true},
    {"label": "Appendices", "pattern": "appendix-*.pdf", "required": false},
    {"label": "KPIs", "pattern": "annex.xlsx", "required": false}
  ]
}
```

Patterns match filenames case-sensitively. They cannot include folders or other
wildcard syntax (`?`, `[]`, `**`). The folder template supports only `{id}`;
use a different workspace folder for each period. PDF definitions determine
reading order, then matching filenames sort lexically. Use `part-01.pdf`,
`part-02.pdf`, and so on when numeric order matters. Unmatched files are ignored;
subfolders are not scanned recursively. Overlapping patterns, reused files, and
multiple matches for the KPI workbook are errors, so REMAIN never guesses which
submission to use.

## Add KPIs

In **Expected documents**, add a KPI workbook and choose whether it is required.
Enter its worksheet name and header row, then define indicators in **KPIs**.
Removing the workbook definition in the editor also removes its KPI settings.
For JSON editing, define `annex` and at least one `metrics` entry together with
the workbook definition; omit all three when KPIs are not collected.

Example additions to the root configuration:

```json
"annex": {"sheet": "Indicators", "header_row": 1},
"metrics": [
  {
    "id": "learners",
    "label": "Learners",
    "column": "Learners",
    "aggregation": "sum",
    "missing": "error",
    "unit": "people",
    "decimals": 0
  }
]
```

Each KPI requires `id`, `label`, `column`, and `aggregation`. Optional fields are
`missing` (default `error`), `unit` (default empty), `decimals` (default `0`,
range 0–6), and `breakdown` (default absent). Each numeric source heading can be
mapped once. Up to 16377 result columns, including classes, fit in the exported
Excel layout; practical GUI tables should be much smaller.

**Display name** (`label`) and **Excel column header** (`column`) may be identical.
The column identifies the spreadsheet heading; the display name appears in the
GUI and reports exactly as entered. A column named `pub_count` can be displayed
as “Publications” without changing the spreadsheet.

## KPI classes

A KPI can be divided by **one text property column** in the same worksheet.
For example, sum the numeric `Publications` column and classify each row using
`Publication type`:

| Publications | Publication type |
| --- | --- |
| 3 | Conference |
| 2 | Journal |
| 1 | Conference |

The result is **6 publications**, including **4 Conference** and **2 Journal**.
Use any property and class names, such as course format, patent status, or product
type. Each KPI can use a different property; multiple KPIs may share a property.

In **KPIs**, enable **Break down by property**, enter the property column, and
add classes. Each class has a stable ID, a display name, and matching Excel text.
The equivalent fragment inside a KPI definition is:

```json
"breakdown": {
  "column": "Publication type",
  "classes": [
    {"id": "conference", "label": "Conference papers", "value": "Conference"},
    {"id": "journal", "label": "Journal articles", "value": "Journal"}
  ]
}
```

- Class IDs must be unique within their KPI and follow the normal ID rules.
- Display names are plain strings. Excel values are separate literal strings,
  matched case-sensitively after trimming surrounding whitespace, and must be unique.
- Every present numeric value, including zero, requires a known class.
  Unknown values, numeric properties, and formulas are rejected.
- With `missing: "skip"`, a missing numeric value may have a blank property.
  With `missing: "zero"`, the substituted zero requires a class.
- Classes use the KPI's aggregation, unit, and precision. An unused class follows
  the [empty-value rules](#aggregations-and-missing-values); a mean remains weighted
  by observations. Each observation belongs to one class and contributes only
  once to the KPI total.

For one publication per row, enter `1` in the numeric column and use `sum`.
For counts already grouped by type, enter the counts and still use `sum`.
`count` counts present numeric cells, not the quantities written in those cells.

## Project organization

Groups and categories classify projects independently. Unassigned projects
remain in the central report and appear under **Unassigned** in the relevant
summary. A category describes a project family and needs no submission of its own.

Parent links describe relationships between reporting projects. Parents can have
parents, but cycles are rejected. Links do not roll up child figures: every
observation contributes once. A parent's annex must exclude figures separately
reported by children. REMAIN cannot detect overlapping real-world observations.

## Aggregations and missing values

| Aggregation | Result |
| --- | --- |
| `sum` | Sum of present numeric values |
| `mean` | Mean of present observations |
| `min` / `max` | Lowest / highest present observation |
| `count` | Count of present numeric cells, including explicit zeros |

For example, means from projects with 2 and 20 observations are recalculated
from all 22 observations. REMAIN does not average the two project means.
If each project supplies one precomputed mean, each row has equal weight;
provide raw observations if an observation-weighted mean is required.

| Missing policy | Empty cell in a nonempty row |
| --- | --- |
| `error` | Stop and report the project, workbook, row, and column |
| `zero` | Insert an observed zero; contributes to mean and count |
| `skip` | Exclude this cell from this KPI's aggregation |

For a supplied workbook with no observed values, sum/mean/min/max are missing
and count is zero. An absent workbook has no values at all, including count.
Scopes without any included workbook also have missing values. The submission
checklist and exported coverage distinguish absence from a measured zero.
Missing output appears as `—` in the GUI/HTML, a blank Excel cell, and `null` in
JSON. Calculations use 64-bit floating-point numbers without intermediate rounding.
Display precision does not change the underlying values. This is not an exact
decimal accounting engine. Non-finite aggregate results are rejected.

## Languages

The top-bar language selector changes interface captions, help text, and number
formatting. **Report language** (`locale`) controls generated captions and number
formatting in exports. Opening a workspace initially selects its report language
for the GUI; you can then change the GUI language independently.

Configured names can use any language, or combine languages in one string, such
as `Research / Forschung`. Changing either language setting leaves them unchanged.

Language settings do not translate source documents or change Excel matching.
See the input rules for [multilingual PDFs](workspaces.md#written-reports) and
[shared Excel headings and class values](workspaces.md#excel-annex).

Configuration field names and aggregation names remain stable English API keys.

Built-in interface and report catalogs live in `locales/`:

- `en.json` — English
- `de.json` — German
- `fr.json` — French, also used for regional tags such as `fr-FR` and `fr-CA`

A workspace may override any catalog key:

```json
"translations": {
  "group": "Department",
  "groups": "Departments",
  "project": "Initiative",
  "projects": "Initiatives"
}
```

This is a fragment to place inside the root configuration object. Overrides
replace built-in captions in both the GUI and exports, regardless of the selected
language; they do not provide alternatives for configured names or Excel matching.
Unspecified keys use the selected built-in base-language catalog, then English.
For an output language without a bundled catalog, set `locale` and supply the
needed caption overrides. Detailed technical diagnostics remain English.

English uses a decimal point; German and French use a comma in the GUI and HTML.
Other languages currently use a decimal point. Excel receives numeric values
and applies its own regional display settings.

The native GUI does not offer full right-to-left layout or complex-script
shaping; use the browser-rendered HTML summary for those presentation needs.
Generated PDF contents pages use the bundled Noto Sans font. For names with
other glyphs, set `contents_font` directly in `workspace.json` to the relative path
of a suitable TrueType font inside the workspace. Missing glyphs produce an error
instead of unreadable text. Contents pages do not perform complex-script shaping
or right-to-left layout. PDF source text retains its embedded fonts and original
layout.

## Upgrade a version 1 workspace

1. Keep a backup of the workspace.
2. Replace language-keyed `title` and `label` objects with the text you want
   displayed, including KPI classes. Existing plain strings need no changes.
3. Move the project metadata (`id`, `label`, optional `group`, `category`, `parent`)
   into a [project list](workspaces.md#project-list). Replace the `projects` array
   with its relative filename.
4. Replace per-project `annex` and `reports` paths with shared `inputs` rules.
   Arrange or rename files to match those rules, retaining the intended order.
   Mark the workbook required if every project must submit KPIs; otherwise make
   it optional or remove it together with `annex` and `metrics`.
5. Replace any `translations.help_text` override with the relevant `help_*`
   entries from `locales/en.json`; the Help window now uses headings and bullets.
6. Set `schema_version` to `2`, open the workspace folder, and analyze it.

Keep stable identifiers, membership references, and Excel source mappings.
Version 1 files must be migrated explicitly; REMAIN does not choose name
translations or infer filename conventions automatically.

## Appearance

Manage GUI fonts and the theme through **Appearance** in the desktop toolbar:

- Toggle **Dark mode** to switch between dark and light colors.
- Choose **Display font**: Ubuntu Light (default), Noto Sans, Lato, or Atkinson Hyperlegible.
    - All three additional fonts are bundled in the application and work offline.
    - Choose **Ubuntu Light (default)** to restore the built-in interface font.
    - Monospaced text uses the built-in Hack font; symbol/emoji fallbacks remain available.
    - **Load custom font…** accepts a local TTF/OTF for additional glyphs.

The theme and bundled font choice are saved locally for your next launch and
apply across workspaces. Custom fonts last for the current session; the next
launch restores your last bundled or default font. Appearance settings do not
change workspace configuration or exported reports. The bundled fonts cover the
English, German, and French interfaces; they do not cover every writing system.
Their licenses are available under **Help → About REMAIN → Bundled font licenses**.

The documentation website has its own sun/moon switch beside the search button.
It initially follows your system's light/dark preference and remembers your
selection in that browser. Its setting is independent of the desktop theme.
