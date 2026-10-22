# Troubleshooting

| Message or symptom | What to do |
| --- | --- |
| Cannot open workspace / Invalid workspace JSON | Select the folder containing `workspace.json`; correct its JSON or schema errors |
| Missing input | Restore the referenced roster/font or correct its relative path in workspace settings |
| Missing required submissions | Put the listed files in the expected project folders and analyze again; a partial export excludes those projects |
| No complete submissions to export | Add/import projects and supply all required files for at least one project |
| Invalid project list | Check CSV headings and quoting or the JSON array; see [project lists](workspaces.md#project-list) |
| Pattern matches multiple files / Input matched more than once | Keep one matching KPI workbook; make PDF patterns distinct and remove duplicate symlinks |
| Missing header / worksheet | Match the configured name and header row to the actual annex |
| Duplicate header | Rename or remove duplicate worksheet headings |
| Missing numeric value | Supply a number or deliberately choose a different missing policy |
| Expected a finite numeric cell | Convert numeric text to numbers; resolve dates, booleans, or spreadsheet errors |
| Class is required / unknown class value | Fill the property column with a configured class value; check spelling and letter case against the [class rules](configuration.md#kpi-classes) |
| Expected a text class value | Use text in the property column, including for class codes such as `01` |
| Formulas are not supported | Calculate in Excel/LibreOffice and paste values into the submitted numeric or property columns |
| Unknown group / category / parent | Correct the referenced ID, including letter case |
| Cycle in parent projects | Remove the circular parent relationship |
| Input assigned more than once | Each project must use its own input files, including through symlinks |
| Interactive forms or widgets | Export a static, unsigned PDF copy |
| Cannot read/decode PDF | Re-export the original document to a valid PDF |
| Destination already exists | Choose a new output folder; the GUI normally chooses an unused suffix |
| File changed on disk | Close the form, reopen the workspace, and repeat your edit |
| File dialog does not open on Linux | Install an XDG portal backend or zenity; expand Workspace folder and enter the folder path directly |
| Missing characters | Choose a bundled font or load a suitable TTF/OTF under Appearance; see [display settings](configuration.md#appearance) |
| Contents font has no glyph | Set `contents_font` directly in `workspace.json` to a suitable workspace-relative TrueType font; the GUI display font does not change PDF contents pages |
| App fails to start | Check graphics drivers/OpenGL and use a package for your OS and CPU architecture |

Errors identify the affected input where possible. They stop the current job;
REMAIN does not silently skip invalid projects. Fix the first issue and analyze
again. The UI remains responsive while a job runs, and the status bar identifies the current operation. There is no cancellation
or percentage-complete estimate.

For a bug report, include the application version, operating system, exact error,
and a small synthetic workspace that reproduces the issue. The demo generator is
useful for making one. Do not include confidential submissions or personal data.
