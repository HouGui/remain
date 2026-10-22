use crate::{Workspace, analysis::Analysis, config::Config, locale};
use anyhow::Result;
use rust_xlsxwriter::{Color, Format, Workbook, Worksheet};
use std::{collections::HashMap, path::Path};

pub(crate) fn write_annex(path: &Path, workspace: &Workspace, analysis: &Analysis) -> Result<()> {
    let config = workspace.config();
    let mut book = Workbook::new();
    let projects: HashMap<_, _> = workspace
        .projects()
        .iter()
        .map(|p| (p.id.as_str(), p))
        .collect();
    let sources: HashMap<_, _> = analysis
        .submissions
        .iter()
        .filter_map(|s| s.annex.as_ref().map(|path| (s.project.as_str(), path)))
        .collect();
    let text = |key| locale::text(&config.locale, &config.translations, key);
    let header = Format::new()
        .set_bold()
        .set_font_color(Color::White)
        .set_background_color(Color::RGB(0x174C52))
        .set_text_wrap();
    {
        let sheet = book.add_worksheet().set_name("Submissions")?;
        sheet.write_string_with_format(
            0,
            0,
            if analysis.is_complete() {
                text("submitted")
            } else {
                text("incomplete_report")
            },
            &header,
        )?;
        sheet.write_string(1, 0, text("included_projects"))?;
        sheet.write_number(1, 1, analysis.projects.len() as f64)?;
        sheet.write_string(2, 0, text("expected_projects"))?;
        sheet.write_number(2, 1, workspace.projects().len() as f64)?;
        sheet.write_string(3, 0, text("kpi_submissions"))?;
        sheet.write_number(
            3,
            1,
            analysis.projects.iter().filter(|p| p.has_kpis).count() as f64,
        )?;
        sheet.merge_range(
            4,
            0,
            4,
            4,
            text("partial_explanation"),
            &Format::new().set_text_wrap(),
        )?;
        sheet.set_row_height(4, 42)?;
        for (column, label) in [
            "project_id",
            text("project"),
            text("submissions"),
            text("missing_submissions"),
            text("optional_missing"),
        ]
        .iter()
        .enumerate()
        {
            sheet.write_string_with_format(6, column as u16, *label, &header)?;
            sheet.set_column_width(column as u16, if column < 2 { 28 } else { 48 })?;
        }
        let wrap = Format::new().set_text_wrap();
        for (index, submission) in analysis.submissions.iter().enumerate() {
            let row = index as u32 + 7;
            sheet.write_string(row, 0, &submission.project)?;
            sheet.write_string(row, 1, &projects[submission.project.as_str()].label)?;
            sheet.write_string(
                row,
                2,
                text(if submission.is_complete() {
                    "submitted"
                } else {
                    "missing_submissions"
                }),
            )?;
            sheet.write_string_with_format(row, 3, submission.missing.join("\n"), &wrap)?;
            sheet.write_string_with_format(
                row,
                4,
                submission.optional_missing.join("\n"),
                &wrap,
            )?;
        }
        sheet.set_freeze_panes(7, 0)?;
        sheet.autofilter(6, 0, analysis.submissions.len() as u32 + 6, 4)?;
    }
    let indicators = config.indicators();
    let formats: Vec<_> = indicators
        .iter()
        .map(|indicator| {
            let m = indicator.metric;
            Format::new().set_num_format(if m.decimals == 0 {
                "0".into()
            } else {
                format!("0.{}", "0".repeat(m.decimals as usize))
            })
        })
        .collect();
    {
        let sheet = book.add_worksheet().set_name("Summary")?;
        setup(
            sheet,
            &[text("group").into(), text("projects").into()],
            config,
            &header,
        )?;
        sheet.write_string(1, 0, text("total"))?;
        sheet.write_number(1, 1, analysis.projects.len() as f64)?;
        values(sheet, 1, 2, &analysis.overall, &formats)?;
        for (i, result) in analysis.groups.iter().enumerate() {
            let row = i as u32 + 2;
            let label = config
                .groups
                .iter()
                .find(|g| Some(&g.id) == result.id.as_ref())
                .map(|g| g.label.as_str())
                .unwrap_or_else(|| text("unassigned"));
            sheet.write_string(row, 0, label)?;
            sheet.write_number(row, 1, result.projects as f64)?;
            values(sheet, row, 2, &result.values, &formats)?;
        }
    }
    {
        let sheet = book.add_worksheet().set_name("Categories")?;
        setup(
            sheet,
            &[text("category").into(), text("projects").into()],
            config,
            &header,
        )?;
        for (i, result) in analysis.categories.iter().enumerate() {
            let row = i as u32 + 1;
            let label = config
                .categories
                .iter()
                .find(|c| Some(&c.id) == result.id.as_ref())
                .map(|c| c.label.as_str())
                .unwrap_or_else(|| text("unassigned"));
            sheet.write_string(row, 0, label)?;
            sheet.write_number(row, 1, result.projects as f64)?;
            values(sheet, row, 2, &result.values, &formats)?;
        }
    }
    {
        let sheet = book.add_worksheet().set_name("Projects")?;
        setup(
            sheet,
            &[
                "project_id".into(),
                text("project").into(),
                "group_id".into(),
                "category_id".into(),
                "parent_id".into(),
                text("rows").into(),
                text("pages").into(),
            ],
            config,
            &header,
        )?;
        for (i, result) in analysis.projects.iter().enumerate() {
            let project = projects.get(result.id.as_str()).expect("analyzed project");
            let row = i as u32 + 1;
            sheet.write_string(row, 0, &project.id)?;
            sheet.write_string(row, 1, project.label.as_str())?;
            sheet.write_string(row, 2, project.group.as_deref().unwrap_or(""))?;
            sheet.write_string(row, 3, project.category.as_deref().unwrap_or(""))?;
            sheet.write_string(row, 4, project.parent.as_deref().unwrap_or(""))?;
            sheet.write_number(row, 5, result.rows as f64)?;
            sheet.write_number(row, 6, result.pages as f64)?;
            values(sheet, row, 7, &result.values, &formats)?;
        }
        sheet.autofilter(
            0,
            0,
            analysis.projects.len() as u32,
            indicators.len() as u16 + 6,
        )?;
    }
    {
        let sheet = book.add_worksheet().set_name("Observations")?;
        setup(
            sheet,
            &[
                "project_id".into(),
                "group_id".into(),
                "category_id".into(),
                text("source").into(),
                text("row").into(),
            ],
            config,
            &header,
        )?;
        for (i, record) in analysis.observations.iter().enumerate() {
            let project = projects
                .get(record.project.as_str())
                .expect("analyzed project");
            let row = i as u32 + 1;
            sheet.write_string(row, 0, &record.project)?;
            sheet.write_string(row, 1, project.group.as_deref().unwrap_or(""))?;
            sheet.write_string(row, 2, project.category.as_deref().unwrap_or(""))?;
            sheet.write_string(row, 3, sources[project.id.as_str()].to_string_lossy())?;
            sheet.write_number(row, 4, record.row)?;
            values(sheet, row, 5, &record.values, &formats)?;
        }
        if !analysis.observations.is_empty() {
            sheet.autofilter(
                0,
                0,
                analysis.observations.len() as u32,
                indicators.len() as u16 + 4,
            )?;
        }
    }
    book.save(path)?;
    Ok(())
}

fn setup(sheet: &mut Worksheet, prefix: &[String], config: &Config, header: &Format) -> Result<()> {
    let indicators = config.indicators();
    let labels = prefix
        .iter()
        .cloned()
        .chain(indicators.iter().map(|indicator| {
            let m = indicator.metric;
            let label = indicator.label();
            if m.unit.is_empty() {
                label
            } else {
                format!("{label} ({})", m.unit)
            }
        }));
    for (column, label) in labels.enumerate() {
        sheet.write_string_with_format(0, column as u16, label, header)?;
        sheet.set_column_width(column as u16, if column < prefix.len() { 28 } else { 22 })?;
    }
    sheet.set_row_height(0, 42)?;
    sheet.set_freeze_panes(1, 0)?;
    Ok(())
}

fn values(
    sheet: &mut Worksheet,
    row: u32,
    offset: u16,
    numbers: &[Option<f64>],
    formats: &[Format],
) -> Result<()> {
    for (i, (number, format)) in numbers.iter().zip(formats).enumerate() {
        if let Some(value) = number {
            sheet.write_number_with_format(row, offset + i as u16, *value, format)?;
        }
    }
    Ok(())
}
