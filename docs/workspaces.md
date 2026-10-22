# Prepare a workspace

Use one folder for each reporting collection. **Open workspace** selects this
folder; it must contain a valid `workspace.json` and the referenced project list.
All inputs stay inside the folder. Files are discovered using the
[document definitions](configuration.md#expected-documents), for example:

- `annual-review/`
    - `workspace.json` — settings and expected document patterns
    - `projects.csv` — project IDs, names, and optional memberships
    - `learning-project/`
        - `report.pdf`
        - `annex.xlsx` — only when KPIs are collected
    - `materials-pilot/`
        - `report.pdf`
        - `appendix-01.pdf`
        - `appendix-02.pdf`

Agree the expected documents and naming rules with contributors. If collecting
KPIs, also agree a common worksheet layout, units, and observation definitions.
Contributors submit their files; the coordinator reviews the checklist and
analysis before exporting.

## Project list

The roster is separate from `workspace.json`, so it can come from a project
management system. Supported formats are UTF-8 comma-separated CSV and a JSON
array. Use stable IDs rather than translated names for references.

| Field | Required | Meaning |
| --- | --- | --- |
| `id` | Yes | Unique project ID; substituted into the project folder template |
| `label` | Yes | Nonempty display name, in any language |
| `group` | No | Exact ID of a configured group |
| `category` | No | Exact ID of a configured category |
| `parent` | No | Exact ID of another project |

CSV example:

```csv
id,label,group,category,parent
pilot,Learning pilot,education,pilots,
pilot2,Second pilot,,pilots,pilot
```

CSV requires the `id` and `label` headings; other standard columns may be omitted.
Extra columns from an external system are ignored. Blank membership cells mean
unassigned. IDs, references, and headings are trimmed; display names are preserved.
Use CSV quoting for names containing commas, quotes, or line breaks. Duplicate
or empty headings and rows with the wrong number of columns are rejected.

Configure groups and categories referenced by the list before saving it.
The equivalent JSON is:

```json
[
  {"id": "pilot", "label": "Learning pilot", "group": "education", "category": "pilots"},
  {"id": "pilot2", "label": "Second pilot", "category": "pilots", "parent": "pilot"}
]
```

In JSON, omit optional fields or use `null`. Unknown fields are rejected. Project
entries contain metadata only; filenames belong in the shared document rules.
[Identifier and relationship rules](configuration.md#project-organization) apply
to both formats. Roster order determines project order within each group.

Use **Project list → Import project list…** to load an external CSV/JSON into the
editor, review it, and save it to the workspace's roster. Import replaces the
list, rather than appending or merging. Saving writes only the five supported
fields; keep the original system export if you need its extra columns. You can
also edit the roster directly in the GUI or update its file externally.
Analysis and export in the GUI reload saved settings and the roster each time.
The editor refuses to overwrite changes made externally while it is open.

An empty roster is allowed during setup. Analysis can show an empty workspace,
but export requires at least one project with all required documents.

## Missing submissions

Opening the workspace checks file presence. The checklist shows each project
and the required or optional files still missing. **Analyze inputs** refreshes
the checklist and validates the contents of complete submissions.

- Missing required documents exclude the **whole project**, including any of its
  available PDFs or KPI rows. Other complete projects can still be exported.
- Partial reports are clearly marked incomplete and list the omitted projects
  and required files in PDF, HTML, Excel (when KPIs are configured), and JSON.
- Missing optional documents do not exclude a project. An optional workbook that
  is absent contributes no KPI values; coverage counts show how many workbooks
  were included. A missing workbook is never treated as a zero measurement.
- A supplied file that is invalid, ambiguous, or outside the workspace causes an
  error. Fix the issue before exporting; invalid content is not silently skipped.

A presence check does not certify document contents. Content validation runs for
projects with all required files; incomplete projects remain excluded until their
required submissions arrive. Run analysis again after collecting more files.

## Input requirements

### Written reports

- Submit one or more readable, unencrypted PDFs per project.
- Export Word, OpenDocument, or other written documents to PDF first.
- Include appendices and declarations as additional PDF parts when needed.
- Parts can use different languages, including within one project. Their language
  does not affect assembly; no language metadata is required.
- Flatten interactive forms to static pages and use unsigned copies for assembly.
  REMAIN rejects form fields and signature-bearing form documents.
- Original page size, orientation, embedded fonts, images, and page content are
  retained. Pages are not rasterized or reflowed.
- Source-level bookmarks, page labels, embedded attachments, and named-destination
  navigation are not carried into the new document catalog. Use static report
  pages; verify important internal links in the final PDF.

Overall, group, and category PDFs begin with generated contents pages; the
[output guide](outputs.md#report-order) explains their page numbers. KPI results
are supplied separately in Excel and printable HTML when configured.
REMAIN does not claim to produce tagged PDF/PDF-A or
preserve the validity of digital signatures.

### Excel annex

When a KPI workbook is configured, each project can supply one matching `.xlsx`
file. Its document definition determines whether it is required or optional.
The same configured worksheet and header row are read from each file.
Use the same configured source names in every submission: translated worksheet
names, headings, or class values are not interchangeable. These source names
are independent of the [GUI and report languages](configuration.md#languages).

- Put one heading per KPI column on the configured header row.
- For a [class breakdown](configuration.md#kpi-classes), add the configured text
  property column beside the numeric data. A row belongs to one class per KPI.
- Use genuine numeric cells. Local Excel number formatting is fine, including
  German decimal display; text such as `"1,5"` is not a numeric cell.
- Each subsequent nonempty row is an observation. A project can provide a single
  row of totals or several distinct observations.
- Do not put subtotal or total rows below the observations: those would be
  counted as observations too.
- Empty rows are ignored. Missing cells in a nonempty row follow the configured
  policy for each KPI.
- Formulas in KPI or property columns are rejected even if cached values exist.
  Calculate them in your spreadsheet editor and **paste values** into the submitted copy.
- Dates, booleans, Excel errors, infinity, and numeric text are rejected.
- Column order can vary. Headings are matched exactly after trimming surrounding
  whitespace. Duplicate headings are rejected.
- Unconfigured columns and other worksheets are not exported. Preserve the source
  files if you need their narrative fields, formulas, formatting, or charts.

For example, a school might use columns `Learners`, `Courses`, and `Satisfaction`.
An industrial team might use `Tests`, `Defects`, and `Energy (kWh)`. Define units
and observation granularity consistently; REMAIN does not convert units or
currencies.

## Paths

Paths in JSON use forward slashes, are relative to the workspace, and must not
contain `.` or `..` components. Absolute paths and symlinks escaping the workspace
are rejected. An input file cannot be assigned twice, including through a symlink.
The same checks apply to the roster, contents font, project folders, and discovered
documents. Spaces and Unicode are allowed in filenames; patterns match their exact case.

Keep your submitted originals. An export is a new bundle; it does not modify or
replace the source documents.
