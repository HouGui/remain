use super::cells::{self, blank};
use crate::{
    analysis::Observation,
    config::{Config, Project},
};
use anyhow::{Context, Result, ensure};
use calamine::{Data, Reader, Xlsx, open_workbook};
use std::{collections::HashMap, path::Path};

pub(crate) fn read_annex(
    path: &Path,
    config: &Config,
    project: &Project,
) -> Result<Vec<Observation>> {
    read(path, config, project).with_context(|| {
        format!(
            "Project {} · {} · sheet {:?}",
            project.id,
            path.display(),
            config.annex.as_ref().expect("KPI settings").sheet
        )
    })
}

fn read(path: &Path, config: &Config, project: &Project) -> Result<Vec<Observation>> {
    let mut workbook: Xlsx<_> = open_workbook(path).context("Cannot read XLSX workbook")?;
    let range = workbook
        .worksheet_range(&config.annex.as_ref().expect("KPI settings").sheet)
        .context("Cannot read worksheet")?;
    let formulas = workbook
        .worksheet_formula(&config.annex.as_ref().expect("KPI settings").sheet)
        .context("Cannot inspect formulas")?;
    let header_row = config.annex.as_ref().expect("KPI settings").header_row - 1;
    let (_, last_column) = range.end().context("Worksheet is empty")?;
    let first_column = range.start().context("Worksheet is empty")?.1;
    let mut headers = HashMap::new();
    for column in first_column..=last_column {
        if let Some(Data::String(text)) = range.get_value((header_row, column)) {
            let text = text.trim();
            if !text.is_empty() {
                ensure!(
                    headers.insert(text, column).is_none(),
                    "Duplicate header {text:?} at row {}",
                    header_row + 1
                );
            }
        }
    }
    let column = |name: &str| {
        headers
            .get(name.trim())
            .copied()
            .with_context(|| format!("Missing header {name:?} at row {}", header_row + 1))
    };
    let columns: Vec<_> = config
        .metrics
        .iter()
        .map(|metric| {
            Ok((
                column(&metric.column)?,
                metric
                    .breakdown
                    .as_ref()
                    .map(|b| column(&b.column))
                    .transpose()?,
            ))
        })
        .collect::<Result<_>>()?;
    let end_row = range
        .end()
        .context("Worksheet is empty")?
        .0
        .max(formulas.end().map_or(0, |p| p.0));
    let mut output = Vec::new();
    let value_count = config.indicators().len();
    for row in header_row + 1..=end_row {
        // Ignore completely empty rows, but never hide a partial record or a formula.
        if (first_column..=last_column).all(|column| blank(range.get_value((row, column))))
            && columns
                .iter()
                .flat_map(|&(value, property)| std::iter::once(value).chain(property))
                .all(|column| {
                    formulas
                        .get_value((row, column))
                        .is_none_or(String::is_empty)
                })
        {
            continue;
        }
        let mut values = Vec::with_capacity(value_count);
        for (&(column, property), metric) in columns.iter().zip(&config.metrics) {
            ensure!(
                formulas
                    .get_value((row, column))
                    .is_none_or(String::is_empty),
                "Row {}, column {:?}: formulas are not supported; paste calculated numeric values",
                row + 1,
                metric.column
            );
            let value = cells::numeric(range.get_value((row, column)), metric, row)?;
            values.push(value);
            if let (Some(column), Some(breakdown)) = (property, &metric.breakdown) {
                ensure!(
                    formulas
                        .get_value((row, column))
                        .is_none_or(String::is_empty),
                    "Row {}, column {:?}: formulas are not supported; paste class text values",
                    row + 1,
                    breakdown.column
                );
                let selected = cells::class(range.get_value((row, column)), value, breakdown, row)?;
                values.extend(
                    (0..breakdown.classes.len())
                        .map(|i| if selected == Some(i) { value } else { None }),
                );
            }
        }
        output.push(Observation {
            project: project.id.clone(),
            row: row + 1,
            values,
        });
    }
    Ok(output)
}
