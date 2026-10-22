# Bring project reports together

REMAIN manages and integrates reports from projects and subprojects, with
optional groups and categories. It assembles PDF reports and printable HTML summaries, with optional analysis
of key performance indicators (KPIs) submitted in Excel files.

Get REMAIN from [GitHub Releases](https://github.com/HouGui/remain/releases),
follow the [installation guide](installation.md), then [try the demo](tutorials.md#try-the-demo).

![REMAIN showing the fictional demo and a group comparison in light mode](assets/overview.png#only-light)
![REMAIN showing the fictional demo and a group comparison in dark mode](assets/overview-dark.png#only-dark)

## Terminology

| Term      | Meaning                                                                        | Examples                                                     |
|-----------|--------------------------------------------------------------------------------|--------------------------------------------------------------|
| Workspace | One collection of submissions for a reporting period                           | Annual portfolio, semester review                            |
| Group     | Optional organizational grouping                                               | Department, research field, business unit                    |
| Category  | Optional project family, independent of the group                              | A program containing Pilot1 and Pilot2                       |
| Project   | One reporting unit with one or more PDF documents and an optional KPI workbook | Research or industrial project, teaching initiative          |
| Parent    | An optional relationship between reporting projects                            | A pilot belonging to a larger project                        |
| KPI       | A numeric indicator supplied in Excel, optionally broken down into classes     | Publications, patents, learners, defects, energy consumption |

A project may belong to one group, one category, both, or neither. See
[project organization](configuration.md#project-organization) for classification
and parent relationships.

Copy submissions from shared drives or project repositories into a local workspace.
REMAIN does not synchronize repositories or upload submissions. The [input guide](workspaces.md)
explains file preparation and supported formats.
