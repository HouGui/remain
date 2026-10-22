//! Real XLSX import tests, including malformed cells and layout changes.
mod common;
use anyhow::Result;
use remain_core::{Workspace, config::Missing};
use rust_xlsxwriter::Formula;

#[test]
fn missing_policies_are_explicit_and_means_use_observations() -> Result<()> {
    let (_tmp, workspace) = common::workspace("en")?;
    let analysis = workspace.analyze()?;
    assert!((analysis.overall[2].expect("mean") - 0.7).abs() < 1e-12);
    assert!((analysis.groups[1].values[2].expect("mean") - 0.6).abs() < 1e-12);
    let mut config = workspace.config().clone();
    config.metrics[2].missing = Missing::Zero;
    assert!(
        (Workspace::new(workspace.root(), config)?.analyze()?.overall[2].expect("mean") - 3.5 / 6.)
            .abs()
            < 1e-12
    );
    let mut config = workspace.config().clone();
    config.metrics[2].missing = Missing::Error;
    let error = Workspace::new(workspace.root(), config)?
        .analyze()
        .expect_err("missing cell");
    assert!(format!("{error:#}").contains("Row 4"));
    Ok(())
}

#[test]
fn formulas_are_rejected_even_with_cached_numbers() -> Result<()> {
    let (_tmp, workspace) = common::workspace("en")?;
    common::replace_annex(workspace.root(), |sheet| {
        sheet.write_formula(1, 0, Formula::new("1+2").set_result("3"))?;
        sheet.write_number(1, 1, 0.)?;
        Ok(())
    })?;
    let error = workspace.analyze().expect_err("formula");
    assert!(format!("{error:#}").contains("formulas are not supported"));
    Ok(())
}

#[test]
fn text_booleans_errors_and_partial_rows_are_not_coerced_to_zero() -> Result<()> {
    let (_tmp, workspace) = common::workspace("en")?;
    for value in ["12", "1,5", "#VALUE!", "true", ""] {
        common::replace_annex(workspace.root(), |sheet| {
            sheet.write_string(1, 0, value)?;
            sheet.write_number(1, 1, 1.)?;
            Ok(())
        })?;
        assert!(workspace.analyze().is_err(), "{value:?}");
    }
    common::replace_annex(workspace.root(), |sheet| {
        sheet.write_boolean(1, 0, true)?;
        sheet.write_number(1, 1, 0.)?;
        Ok(())
    })?;
    assert!(workspace.analyze().is_err());
    Ok(())
}

#[test]
fn headers_are_matched_by_name_and_absolute_row_not_position() -> Result<()> {
    let (_tmp, workspace) = common::workspace("en")?;
    // Replace every annex so the common layout is consistent.
    let mut config = workspace.config().clone();
    config.annex.as_mut().expect("KPI settings").header_row = 4;
    for project in workspace.projects() {
        let mut book = rust_xlsxwriter::Workbook::new();
        let sheet = book.add_worksheet().set_name("KPIs")?;
        sheet.write_string(3, 2, " Quality score ")?;
        sheet.write_string(3, 3, " Patents ")?;
        sheet.write_string(3, 4, " Publications ")?;
        sheet.write_number(6, 2, 0.5)?;
        sheet.write_number(6, 3, 2.)?;
        sheet.write_number(6, 4, 3.)?;
        book.save(workspace.root().join(&project.id).join("annex.xlsx"))?;
    }
    let analysis = Workspace::new(workspace.root(), config)?.analyze()?;
    assert_eq!(analysis.overall, vec![Some(9.), Some(6.), Some(0.5)]);
    assert!(analysis.observations.iter().all(|r| r.row == 7));
    Ok(())
}

#[test]
fn duplicate_and_missing_headers_and_missing_sheets_fail() -> Result<()> {
    let (_tmp, workspace) = common::workspace("en")?;
    for replacement in ["Publications", "Other"] {
        common::replace_annex(workspace.root(), |sheet| {
            sheet.write_string(0, 1, replacement)?;
            Ok(())
        })?;
        assert!(workspace.analyze().is_err());
    }
    let mut config = workspace.config().clone();
    config.annex.as_mut().expect("KPI settings").sheet = "Missing".into();
    assert!(Workspace::new(workspace.root(), config)?.analyze().is_err());
    Ok(())
}

#[test]
fn empty_annex_has_no_observations_and_no_fabricated_totals() -> Result<()> {
    let (_tmp, workspace) = common::workspace("en")?;
    common::replace_annex(workspace.root(), |_| Ok(()))?;
    let analysis = workspace.analyze()?;
    assert_eq!(analysis.projects[0].rows, 0);
    assert_eq!(analysis.projects[0].values, vec![None, None, None]);
    assert_eq!(analysis.overall[0], Some(7.));
    Ok(())
}
