//! Property columns reject ambiguous or invalid classes without losing totals.
mod common;
use anyhow::Result;
use remain_core::{Workspace, config::Missing};
use rust_xlsxwriter::{Formula, Workbook, Worksheet};

fn replace(workspace: &Workspace, write: impl FnOnce(&mut Worksheet) -> Result<()>) -> Result<()> {
    let mut book = Workbook::new();
    let sheet = book.add_worksheet().set_name("KPIs")?;
    for (i, label) in [
        "Publications",
        "Patents",
        "Quality score",
        "Publication type",
        "Assessment phase",
    ]
    .iter()
    .enumerate()
    {
        sheet.write_string(0, i as u16, *label)?;
    }
    sheet.write_number(1, 0, 0.)?;
    sheet.write_number(1, 1, 1.)?;
    sheet.write_string(1, 3, "Conference")?;
    write(sheet)?;
    book.save(workspace.root().join("atlas/annex.xlsx"))?;
    Ok(())
}

#[test]
fn properties_match_trimmed_text_and_do_not_require_a_class_for_skipped_values() -> Result<()> {
    let (_tmp, workspace) = common::classified_workspace("en")?;
    replace(&workspace, |sheet| {
        sheet.write_string(1, 3, " Conference ")?;
        Ok(())
    })?;
    let analysis = workspace.analyze()?;
    let atlas = analysis
        .projects
        .iter()
        .find(|p| p.id == "atlas")
        .expect("project");
    assert_eq!(&atlas.values[..4], &[Some(0.), Some(0.), None, None]);
    assert!(atlas.values[5..].iter().all(Option::is_none));
    let mut config = workspace.config().clone();
    let projects = vec![workspace.projects()[0].clone()];
    config.metrics[2].missing = Missing::Zero;
    assert!(
        format!(
            "{:#}",
            Workspace::with_projects(workspace.root(), config.clone(), projects.clone())?
                .analyze()
                .expect_err("zero requires a class")
        )
        .contains("Assessment phase")
    );
    replace(&workspace, |sheet| {
        sheet.write_string(1, 4, "Baseline")?;
        Ok(())
    })?;
    let analysis = Workspace::with_projects(workspace.root(), config, projects)?.analyze()?;
    let atlas = analysis
        .projects
        .iter()
        .find(|p| p.id == "atlas")
        .expect("project");
    assert_eq!(&atlas.values[5..], &[Some(0.), Some(0.), None]);
    Ok(())
}

#[test]
fn kpis_share_a_property_with_independent_class_order_and_offset_headers() -> Result<()> {
    let (_tmp, workspace) = common::classified_workspace("en")?;
    let mut config = workspace.config().clone();
    let projects = vec![workspace.projects()[0].clone()];
    config.metrics.truncate(2);
    config.annex.as_mut().expect("KPI settings").header_row = 3;
    let mut shared = config.metrics[0].breakdown.clone().expect("classes");
    shared.classes.rotate_left(1);
    config.metrics[1].breakdown = Some(shared);
    let mut book = Workbook::new();
    let sheet = book.add_worksheet().set_name("KPIs")?;
    for (column, label) in [(1, "Publication type"), (3, "Patents"), (5, "Publications")] {
        sheet.write_string(2, column, label)?;
    }
    for (row, class, patents, publications) in [(3, "Journal", 2., 8.), (4, "Conference", 0., -3.)]
    {
        sheet.write_string(row, 1, class)?;
        sheet.write_number(row, 3, patents)?;
        sheet.write_number(row, 5, publications)?;
    }
    book.save(workspace.root().join("atlas/annex.xlsx"))?;
    let analysis = Workspace::with_projects(workspace.root(), config, projects)?.analyze()?;
    assert_eq!(
        analysis.overall,
        [
            Some(5.),
            Some(-3.),
            Some(8.),
            None,
            Some(2.),
            Some(2.),
            None,
            Some(0.)
        ]
    );
    assert_eq!(analysis.observations.len(), 2);
    assert_eq!(analysis.observations[0].row, 4);
    assert_eq!(analysis.metric_ids[5], "patents.journal");
    assert_eq!(analysis.metric_ids[7], "patents.conference");
    Ok(())
}

#[test]
fn invalid_class_cells_report_project_row_and_property() -> Result<()> {
    let (_tmp, workspace) = common::classified_workspace("en")?;
    for case in 0..5 {
        replace(&workspace, |sheet| {
            match case {
                0 => {
                    sheet.write_string(1, 3, "Unknown")?;
                }
                1 => {
                    sheet.write_string(1, 3, "conference")?;
                }
                2 => {
                    sheet.write_string(1, 3, " ")?;
                }
                3 => {
                    sheet.write_number(1, 3, 1.)?;
                }
                _ => {
                    sheet.write_formula(
                        1,
                        3,
                        Formula::new("\"Conference\"").set_result("Conference"),
                    )?;
                }
            }
            Ok(())
        })?;
        let error = format!("{:#}", workspace.analyze().expect_err("invalid class"));
        for context in ["atlas", "Row 2", "Publication type"] {
            assert!(error.contains(context), "{case}: {error}");
        }
    }
    replace(&workspace, |sheet| {
        sheet.write_string(0, 3, "Wrong header")?;
        Ok(())
    })?;
    let error = format!("{:#}", workspace.analyze().expect_err("missing header"));
    assert!(error.contains("Missing header \"Publication type\""));
    Ok(())
}

#[test]
fn invalid_class_configuration_fails_before_reading_inputs() -> Result<()> {
    let (_tmp, workspace) = common::classified_workspace("en")?;
    for case in 0..8 {
        let mut config = workspace.config().clone();
        let breakdown = config.metrics[0].breakdown.as_mut().expect("classes");
        match case {
            0 => breakdown.column.clear(),
            1 => breakdown.column = "Publications".into(),
            2 => breakdown.classes.clear(),
            3 => breakdown.classes[1].id = "CONFERENCE".into(),
            4 => breakdown.classes[1].value = " Conference ".into(),
            5 => breakdown.classes[0].value.clear(),
            6 => breakdown.classes[0].label = "".into(),
            _ => breakdown.classes[0].id = "bad.id".into(),
        }
        assert!(config.validate().is_err(), "case {case}");
    }
    let mut config = workspace.config().clone();
    let breakdown = config.metrics[0].breakdown.as_mut().expect("classes");
    breakdown.classes = vec![breakdown.classes[0].clone(); 16_377];
    assert!(
        format!("{:#}", config.validate().expect_err("Excel column limit"))
            .contains("result columns")
    );
    Ok(())
}
