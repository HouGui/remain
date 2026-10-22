//! Importable project rosters, independent of workspace settings and input files.

use crate::config::Project;
use anyhow::{Context, Result, ensure};
use std::{collections::HashSet, fs, path::Path};

const FIELDS: [&str; 5] = ["id", "label", "group", "category", "parent"];

/// Read a UTF-8 CSV table or JSON array of projects.
/// CSV requires `id` and `label`; membership columns are optional, and unrelated
/// export columns are ignored. JSON uses the strict [`Project`] structure.
///
/// # Errors
/// Rejects unreadable files, unsupported extensions, malformed records, and
/// missing or duplicate CSV headings. Validate references with
/// [`crate::config::Config::validate_projects`] after importing.
pub fn load(path: impl AsRef<Path>) -> Result<Vec<Project>> {
    let path = path.as_ref();
    let bytes =
        fs::read(path).with_context(|| format!("Cannot read project list {}", path.display()))?;
    decode(path, &bytes).with_context(|| format!("Invalid project list {}", path.display()))
}

fn decode(path: &Path, bytes: &[u8]) -> Result<Vec<Project>> {
    if path
        .extension()
        .is_some_and(|s| s.eq_ignore_ascii_case("json"))
    {
        return Ok(serde_json::from_slice(bytes)?);
    }
    ensure!(
        path.extension()
            .is_some_and(|s| s.eq_ignore_ascii_case("csv")),
        "Use a .csv or .json project list"
    );
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::Headers)
        .from_reader(bytes);
    let headers = reader.headers()?.clone();
    let mut unique = HashSet::new();
    for heading in &headers {
        ensure!(
            !heading.is_empty() && unique.insert(heading),
            "CSV headings must be nonempty and unique: {heading:?}"
        );
    }
    let columns = FIELDS.map(|name| headers.iter().position(|heading| heading == name));
    ensure!(
        columns[0].is_some() && columns[1].is_some(),
        "CSV requires id and label headings"
    );
    reader
        .records()
        .enumerate()
        .map(|(row, record)| {
            let record = record.with_context(|| format!("CSV record {}", row + 2))?;
            let value = |index: usize| {
                columns[index]
                    .and_then(|column| record.get(column))
                    .unwrap_or("")
            };
            let optional = |index| {
                let value = value(index).trim();
                (!value.is_empty()).then(|| value.to_owned())
            };
            Ok(Project {
                id: value(0).trim().into(),
                label: value(1).into(),
                group: optional(2),
                category: optional(3),
                parent: optional(4),
            })
        })
        .collect()
}

/// Encode a roster in the format selected by the destination's extension.
/// CSV uses the five standard headings, quoted as necessary, with LF newlines.
///
/// # Errors
/// Returns an error for an unsupported extension or serialization failure.
pub fn encode(path: impl AsRef<Path>, projects: &[Project]) -> Result<Vec<u8>> {
    let path = path.as_ref();
    if path
        .extension()
        .is_some_and(|s| s.eq_ignore_ascii_case("json"))
    {
        return Ok(serde_json::to_vec_pretty(projects)?);
    }
    ensure!(
        path.extension()
            .is_some_and(|s| s.eq_ignore_ascii_case("csv")),
        "Use a .csv or .json project list"
    );
    let mut writer = csv::Writer::from_writer(Vec::new());
    writer.write_record(FIELDS)?;
    for project in projects {
        writer.write_record([
            &project.id,
            &project.label,
            project.group.as_deref().unwrap_or(""),
            project.category.as_deref().unwrap_or(""),
            project.parent.as_deref().unwrap_or(""),
        ])?;
    }
    Ok(writer.into_inner()?)
}
