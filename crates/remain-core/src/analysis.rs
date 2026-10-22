//! Numeric observations and summaries, calculated with Polars.

use crate::{
    config::{Aggregation, Classification, Config, Project},
    submissions::Submission,
};
use anyhow::{Result, ensure};
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// One nonempty annex row with numeric cells in [`Config::indicators`] order.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Observation {
    /// Project whose annex supplied the row.
    pub project: String,
    /// Original one-based Excel row, including the header offset.
    pub row: u32,
    /// Numeric values; `None` represents a skipped missing cell or a class that
    /// does not match this observation. Class columns never change KPI totals.
    pub values: Vec<Option<f64>>,
}

/// Validated project results in [`Config::indicators`] order.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectSummary {
    /// Stable project identifier.
    pub id: String,
    /// Optional group identifier.
    pub group: Option<String>,
    /// Optional category identifier.
    pub category: Option<String>,
    /// Count of nonempty observation rows.
    pub rows: usize,
    /// Whether this project supplied a validated KPI workbook.
    pub has_kpis: bool,
    /// Number of pages across all submitted report parts.
    pub pages: usize,
    /// Aggregated numeric KPIs.
    pub values: Vec<Option<f64>>,
}

/// Results for a group or category, including empty configured classifications.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScopeSummary {
    /// Stable identifier; `None` identifies projects without this classification.
    pub id: Option<String>,
    /// Number of complete, analyzed projects in this scope.
    pub projects: usize,
    /// Number of projects expected according to the roster, including missing submissions.
    pub expected_projects: usize,
    /// Aggregated numeric KPIs.
    pub values: Vec<Option<f64>>,
}

/// Snapshot of analyzed submissions, with explicit coverage and missing inputs.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Analysis {
    /// Result identifiers from [`Config::indicators`], defining the order of
    /// every `values` vector and `overall`.
    pub metric_ids: Vec<String>,
    /// Project results in group order, then project order.
    pub projects: Vec<ProjectSummary>,
    /// Group results in configuration order, followed by unassigned projects.
    pub groups: Vec<ScopeSummary>,
    /// Category results in configuration order, followed by unassigned projects.
    pub categories: Vec<ScopeSummary>,
    /// Overall aggregates, calculated from observations exactly once.
    pub overall: Vec<Option<f64>>,
    /// Source observations for traceability and consolidated Excel output.
    pub observations: Vec<Observation>,
    /// All roster projects and their discovered files or missing filename patterns.
    pub submissions: Vec<Submission>,
}

impl Analysis {
    /// Whether every roster project was included. Empty rosters cannot be exported.
    pub fn is_complete(&self) -> bool {
        self.submissions.iter().all(Submission::is_complete)
    }
}

pub(crate) fn scopes(
    config: &Config,
    roster: &[Project],
    included: &[ProjectSummary],
    definitions: &[Classification],
    observations: &[Observation],
    classify: impl Fn(&Project) -> Option<&str>,
) -> Result<Vec<ScopeSummary>> {
    let projects: HashMap<_, _> = roster.iter().map(|p| (p.id.as_str(), p)).collect();
    let complete: std::collections::HashSet<_> = included.iter().map(|p| p.id.as_str()).collect();
    let kpis: std::collections::HashSet<_> = included
        .iter()
        .filter(|p| p.has_kpis)
        .map(|p| classify(projects[p.id.as_str()]))
        .collect();
    let mut expected = HashMap::<Option<&str>, usize>::new();
    let mut counts = HashMap::<Option<&str>, usize>::new();
    let mut buckets = HashMap::<Option<&str>, Vec<&Observation>>::new();
    for project in roster {
        *expected.entry(classify(project)).or_default() += 1;
        if complete.contains(project.id.as_str()) {
            *counts.entry(classify(project)).or_default() += 1;
        }
    }
    for row in observations {
        buckets
            .entry(classify(projects[row.project.as_str()]))
            .or_default()
            .push(row);
    }
    definitions
        .iter()
        .map(|c| Some(c.id.as_str()))
        .chain(expected.contains_key(&None).then_some(None))
        .map(|id| {
            Ok(ScopeSummary {
                id: id.map(str::to_owned),
                projects: counts.get(&id).copied().unwrap_or(0),
                expected_projects: expected.get(&id).copied().unwrap_or(0),
                values: if kpis.contains(&id) {
                    summarize(config, buckets.get(&id).map(Vec::as_slice).unwrap_or(&[]))?
                } else {
                    vec![None; config.indicators().len()]
                },
            })
        })
        .collect()
}

pub(crate) fn summarize(config: &Config, rows: &[&Observation]) -> Result<Vec<Option<f64>>> {
    config
        .indicators()
        .iter()
        .enumerate()
        .map(|(index, indicator)| {
            let metric = indicator.metric;
            let values: Vec<Option<f64>> = rows.iter().map(|r| r.values[index]).collect();
            let series = Series::new(indicator.id.clone().into(), values);
            let numbers = series.f64()?;
            let present = numbers.len() - numbers.null_count();
            let result = if metric.aggregation == Aggregation::Count {
                Some(present as f64)
            } else if present == 0 {
                None
            } else {
                match metric.aggregation {
                    Aggregation::Sum => numbers.sum(),
                    Aggregation::Mean => numbers.mean(),
                    Aggregation::Min => numbers.min(),
                    Aggregation::Max => numbers.max(),
                    Aggregation::Count => unreachable!(),
                }
            };
            ensure!(
                result.is_none_or(f64::is_finite),
                "{}: aggregation overflow; reduce the scale of input values",
                indicator.id
            );
            Ok(result)
        })
        .collect()
}

#[cfg(test)]
#[path = "analysis_tests.rs"]
mod tests;
