# Tutorials

These tutorials use the desktop GUI. [Install REMAIN](installation.md) first.

## Try the demo

1. Start REMAIN and select **English**, **Deutsch**, or **Français** in the top bar.
2. Choose **Create demo workspace → Basic reporting**, then a writable folder.
   REMAIN creates a `remain-demo` subfolder, adding a numeric suffix if needed.
3. Click **Analyze inputs** and review the overall figures and project tables.
4. Select a KPI and try the bar and dot charts.
5. Click **Export report bundle** and select a parent folder.
6. Click **Open output folder**. Open `index.html` for the printable summary and
   `report.pdf` for the assembled reports.

The bottom status bar names the current operation, shows which project is being
read, and reports completion or an issue that needs attention.

The fictional demo has three projects, two groups, one category, and a subproject.
One project supplies two PDF parts. Expect **12 publications**, **4 patents**,
and a **0.70 quality mean** (**0,70** in German and French).
Source checkouts include the same workspaces under `examples/`:

- `demo-en/` — English
- `demo-de/` — German
- `demo-fr/` — French

Use **Open workspace** to select the demo folder containing `workspace.json`.

**Help → User guide** shows a short workflow and a summary of the exported files,
with a button to open the full bundled guide. **Help → About REMAIN** shows the
version, license, and online documentation link when configured by the distributor.

## Split a KPI into classes

1. Choose **Create demo workspace → KPI classes**, then analyze the workspace.
2. The Publications card shows a total of **12**, split into **6 Conference** and
   **6 Journal**. Book is unused (`—`). Quality is also split by assessment phase.
3. Open **Edit configuration → KPIs** and expand Publications. Review its
   **Break down by property** setting and the matching Excel text for each class.
4. In your own annexes, add the property column and put one class value beside
   each numeric value. Follow the [class rules](configuration.md#kpi-classes).
5. Save, analyze, and select a class in the KPI selector to compare groups,
   categories, or projects. Export to include the total and every class.

The same example is in `examples/demo-classes/`. It also includes an unassigned
project, an empty group, an explicit zero, and a skipped value. A journal value of
zero is different from an unused Book class.

## Create a portfolio without groups

**Goal:** combine independent projects without an organizational hierarchy.

1. Create a workspace folder and put each project's PDF in its own subfolder,
   such as `pilot/report.pdf`. Follow the [input rules](workspaces.md#input-requirements).
2. Select **New workspace**, choose the folder, and enter its title and period.
3. Leave **Groups** and **Categories** empty. In **Expected documents**, keep
   folder `{id}` and set the required PDF pattern to `report.pdf`. Save.
4. Open **Project list**. Add each project ID and display name, or import a
   [CSV/JSON list](workspaces.md#project-list). Leave classifications Unassigned.
5. Save the list, review the submission checklist, analyze, and export.

To collect KPIs too, add a workbook under **Expected documents**, enter its
worksheet settings, and define columns in **KPIs**. The workbook can be optional.

Projects with all required documents appear in the central PDF and individual project PDFs. The group
and category summaries show an Unassigned row; no artificial group is required.

## Organize a project family

**Goal:** compare related projects across departments or research fields.

1. Open **Edit configuration**. Add any groups you need, such as Research and
   Education.
2. Under **Categories**, add ID `pilots` with display name `Pilot program`.
3. Save the settings, then open **Project list**. Add projects `pilot` and `pilot2`
   and assign both to category Pilot program.
4. Choose a group independently for each project, or leave it Unassigned.
5. Save the list, submit the expected files, and analyze. Select **Categories**
   to review the family’s results.

Export includes `categories/pilots.pdf` and, when KPIs are configured, the
category's Excel summary.
See [project organization](configuration.md#project-organization) for the
difference between categories and parent projects.

## Follow up on missing submissions

1. Open the workspace folder and expand **Submissions**. Outstanding required
   files appear first, with the affected project names and expected paths.
2. Collect the missing files and place them in the matching project folders.
3. Click **Analyze inputs** to refresh the checklist and validate complete projects.
4. Export when needed. If some required submissions are still missing, the report
   is marked incomplete and lists them. No export is possible until at least one
   project has supplied all required documents.

Optional workbooks can arrive later without holding up PDF assembly. The report
shows KPI coverage, and absent workbooks do not contribute zero values.

## Compare KPIs and repeat a reporting cycle

**Goal:** inspect differences without changing the underlying report.

1. After analysis, choose **Groups**, **Categories**, or **Projects**.
2. Select a KPI. Switch between **Bar chart** and **Dot chart**.
3. Filter by name or ID, sort by the selected KPI, and adjust **Display limit**.
   The displayed count shows how many matching rows are visible.
4. Read values beside the marks. Negative values appear left of zero; missing
   values have no mark. The table preserves the full numeric context.
5. Export when ready. Filters, sorting, and display limits affect only the GUI;
   exports include all projects with complete required submissions.

For the next period, copy the workspace folder, replace the submissions, update
the period, and analyze again. The period is a label, not an automatic date filter.
Keep the original submissions and previous exports for comparison.
