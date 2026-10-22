//! Synthetic workspaces for learning, tests, and reproducible bug reports.
//!
//! All organizations, projects, narrative text, and values are fictional. The
//! generator uses the same Rust dependencies as production exports.

mod classified;
mod configuration;
mod document;

use configuration::configuration;

use crate::locale::LANGUAGES;
use anyhow::{Result, ensure};
use rust_xlsxwriter::{Format, Workbook};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Generate a new demo directory in a bundled language and return its workspace
/// file. The three projects illustrate two groups, multiple report parts, a
/// subproject, a missing optional value, and unequal observation counts.
///
/// # Errors
/// Fails if the destination exists, the parent is unavailable, the locale is
/// not a base tag in [`LANGUAGES`], or writing any fixture fails. Existing data
/// is never changed.
pub fn create(destination: impl AsRef<Path>, locale: &str) -> Result<PathBuf> {
    generate(destination.as_ref(), locale, false)
}

/// Generate a demo with publication classes and assessment phases. It includes
/// an unused class, an unassigned project, zeroes, and a skipped quality value.
///
/// # Errors
/// Uses the same validation and overwrite protection as [`create`].
pub fn create_classified(destination: impl AsRef<Path>, locale: &str) -> Result<PathBuf> {
    generate(destination.as_ref(), locale, true)
}

fn generate(destination: &Path, locale: &str, with_classes: bool) -> Result<PathBuf> {
    ensure!(
        LANGUAGES.iter().any(|language| language.code == locale),
        "Unsupported demo language: {locale}"
    );
    ensure!(
        !destination.exists(),
        "Demo destination already exists: {}",
        destination.display()
    );
    let parent = destination
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let stage = tempfile::Builder::new()
        .prefix(".remain-demo-")
        .tempdir_in(parent)?;
    let (mut config, mut projects) = configuration(locale);
    if with_classes {
        classified::configure(&mut config, &mut projects);
    }
    config.validate()?;
    config.validate_projects(&projects)?;
    let data = [
        vec![
            [Some(3.), Some(1.), Some(0.8)],
            [Some(2.), Some(0.), Some(0.9)],
        ],
        vec![[Some(4.), Some(2.), Some(0.6)]],
        vec![
            [Some(1.), Some(0.), Some(0.7)],
            [Some(2.), Some(1.), Some(0.5)],
            [Some(0.), Some(0.), None],
        ],
    ];
    for (index, (project, rows)) in projects.iter().zip(data).enumerate() {
        fs::create_dir_all(stage.path().join(&project.id))?;
        let mut book = Workbook::new();
        let sheet = book
            .add_worksheet()
            .set_name(&config.annex.as_ref().expect("demo KPI settings").sheet)?;
        let header = Format::new().set_bold().set_background_color(0xDFEEEB);
        for (i, metric) in config.metrics.iter().enumerate() {
            sheet.write_string_with_format(0, i as u16, &metric.column, &header)?;
            sheet.set_column_width(i as u16, 25)?;
        }
        sheet.set_freeze_panes(1, 0)?;
        for (r, row) in rows.iter().enumerate() {
            for (c, value) in row.iter().enumerate() {
                if let Some(value) = value {
                    sheet.write_number(r as u32 + 1, c as u16, *value)?;
                }
            }
        }
        if with_classes {
            classified::properties(sheet, &config, index, &header)?;
        }
        book.save(stage.path().join(&project.id).join("annex.xlsx"))?;
        for part in 1..=if project.id == "nova" { 2 } else { 1 } {
            document::create(
                &stage
                    .path()
                    .join(&project.id)
                    .join(format!("part-{part}.pdf")),
                &project.id,
                part,
                locale,
            )?;
        }
    }
    fs::write(
        stage.path().join(&config.projects),
        crate::projects::encode(&config.projects, &projects)?,
    )?;
    fs::write(
        stage.path().join("workspace.json"),
        serde_json::to_vec_pretty(&config)?,
    )?;
    crate::export::publish(stage, destination)?;
    Ok(destination.join("workspace.json"))
}
