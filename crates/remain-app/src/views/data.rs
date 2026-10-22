use remain_core::{Workspace, analysis::Analysis, locale};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) enum Sort {
    #[default]
    Configured,
    Name,
    Value,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) enum Chart {
    #[default]
    Bars,
    Dots,
}

pub(crate) struct Options {
    pub filter: String,
    pub sort: Sort,
    pub limit: usize,
    pub chart: Chart,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            filter: String::new(),
            sort: Sort::Configured,
            limit: 25,
            chart: Chart::Bars,
        }
    }
}

pub(super) struct Row<'a> {
    pub id: &'a str,
    pub label: &'a str,
    pub values: &'a [Option<f64>],
    pub count: String,
}

pub(super) fn rows<'a>(
    workspace: &'a Workspace,
    analysis: &'a Analysis,
    tab: usize,
    language: &str,
) -> Vec<Row<'a>> {
    let config = workspace.config();
    if tab == 2 {
        let projects: HashMap<_, _> = workspace
            .projects()
            .iter()
            .map(|p| (p.id.as_str(), p))
            .collect();
        analysis
            .projects
            .iter()
            .map(|result| Row {
                id: &result.id,
                label: projects[result.id.as_str()].label.as_str(),
                values: &result.values,
                count: result.pages.to_string(),
            })
            .collect()
    } else {
        let (definitions, results) = if tab == 0 {
            (&config.groups, &analysis.groups)
        } else {
            (&config.categories, &analysis.categories)
        };
        let names: HashMap<_, _> = definitions
            .iter()
            .map(|c| (c.id.as_str(), &c.label))
            .collect();
        results
            .iter()
            .map(|result| Row {
                id: result.id.as_deref().unwrap_or(""),
                label: result
                    .id
                    .as_deref()
                    .and_then(|id| names.get(id))
                    .map(|label| label.as_str())
                    .unwrap_or_else(|| locale::text(language, &config.translations, "unassigned")),
                values: &result.values,
                count: format!("{} / {}", result.projects, result.expected_projects),
            })
            .collect()
    }
}

pub(super) fn select(rows: &mut Vec<Row<'_>>, options: &Options, metric: usize) {
    if !options.filter.is_empty() {
        let query = options.filter.to_lowercase();
        rows.retain(|row| {
            row.label.to_lowercase().contains(&query) || row.id.to_lowercase().contains(&query)
        });
    }
    match options.sort {
        Sort::Configured => {}
        Sort::Name => rows.sort_by_cached_key(|row| row.label.to_lowercase()),
        Sort::Value => rows.sort_by(|a, b| {
            match (
                a.values.get(metric).copied().flatten(),
                b.values.get(metric).copied().flatten(),
            ) {
                (Some(a), Some(b)) => b.total_cmp(&a),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                _ => std::cmp::Ordering::Equal,
            }
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn numeric_sort_keeps_negative_zero_and_missing_distinct() {
        let a = [Some(-2.)];
        let b = [Some(0.)];
        let c = [None];
        let d = [Some(5.)];
        let mut rows = vec![
            Row {
                id: "a",
                label: "Alpha",
                values: &a,
                count: "1".into(),
            },
            Row {
                id: "b",
                label: "Beta",
                values: &b,
                count: "1".into(),
            },
            Row {
                id: "c",
                label: "Gamma",
                values: &c,
                count: "1".into(),
            },
            Row {
                id: "d",
                label: "Delta",
                values: &d,
                count: "1".into(),
            },
        ];
        let mut options = Options {
            sort: Sort::Value,
            ..Default::default()
        };
        select(&mut rows, &options, 0);
        assert_eq!(
            rows.iter().map(|r| r.id).collect::<Vec<_>>(),
            ["d", "b", "a", "c"]
        );
        options.filter = "ALP".into();
        select(&mut rows, &options, 0);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, "a");
    }
}
