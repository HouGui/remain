#![allow(dead_code)]
use anyhow::Result;
use remain_core::Workspace;
use rust_xlsxwriter::{Workbook, Worksheet};
use std::path::Path;

pub fn workspace(locale: &str) -> Result<(tempfile::TempDir, Workspace)> {
    let directory = tempfile::tempdir()?;
    let config = remain_core::demo::create(directory.path().join("workspace"), locale)?;
    Ok((directory, Workspace::load(config)?))
}

pub fn classified_workspace(locale: &str) -> Result<(tempfile::TempDir, Workspace)> {
    let directory = tempfile::tempdir()?;
    let config = remain_core::demo::create_classified(directory.path().join("workspace"), locale)?;
    Ok((directory, Workspace::load(config)?))
}

pub fn replace_annex(root: &Path, write: impl FnOnce(&mut Worksheet) -> Result<()>) -> Result<()> {
    let mut book = Workbook::new();
    let sheet = book.add_worksheet().set_name("KPIs")?;
    for (i, label) in ["Publications", "Patents", "Quality score"]
        .iter()
        .enumerate()
    {
        sheet.write_string(0, i as u16, *label)?;
    }
    write(sheet)?;
    book.save(root.join("atlas/annex.xlsx"))?;
    Ok(())
}
