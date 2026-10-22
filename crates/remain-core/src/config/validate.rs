use super::{Config, Project, SCHEMA_VERSION};
use anyhow::{Result, bail, ensure};
use std::{
    collections::{HashMap, HashSet},
    path::{Component, Path},
};

impl Config {
    /// Validate workspace settings, definitions, and portable input rules.
    /// Use [`Self::validate_projects`] separately for roster references.
    ///
    /// # Errors
    /// Returns a contextual error for unsupported versions, empty or ambiguous
    /// definitions, invalid filenames, or undefined/cyclic references.
    /// Roster files are checked when loading a workspace; submitted document
    /// contents are checked during analysis or export.
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.schema_version == SCHEMA_VERSION,
            "Unsupported schema_version {}; expected {SCHEMA_VERSION}",
            self.schema_version
        );
        label(&self.title, "title")?;
        ensure!(!self.locale.trim().is_empty(), "locale must not be empty");
        ensure!(!self.period.trim().is_empty(), "period must not be empty");
        let has_kpis = self
            .inputs
            .documents
            .iter()
            .any(|document| document.is_kpi());
        ensure!(
            has_kpis == self.annex.is_some() && has_kpis == !self.metrics.is_empty(),
            "Define worksheet settings and at least one metric exactly when a KPI workbook is defined"
        );
        if let Some(annex) = &self.annex {
            ensure!(
                !annex.sheet.trim().is_empty(),
                "annex.sheet must not be empty"
            );
            ensure!(
                (1..=1_048_576).contains(&annex.header_row),
                "annex.header_row must be an Excel row (1–1048576)"
            );
        }
        relative(&self.projects)?;
        ensure!(
            self.projects
                .extension()
                .is_some_and(|s| s.eq_ignore_ascii_case("csv") || s.eq_ignore_ascii_case("json")),
            "Project list must be a .csv or .json file"
        );
        self.inputs.validate()?;
        if let Some(font) = &self.contents_font {
            relative(font)?;
            ensure!(
                font.extension()
                    .is_some_and(|s| s.eq_ignore_ascii_case("ttf")),
                "Contents font must be a TrueType .ttf file"
            );
        }
        ensure!(
            self.metrics
                .iter()
                .map(|metric| {
                    1 + metric
                        .breakdown
                        .as_ref()
                        .map_or(0, |breakdown| breakdown.classes.len())
                })
                .sum::<usize>()
                <= 16_377,
            "Expected at most 16377 result columns, including KPI classes"
        );
        unique(self.groups.iter().map(|x| x.id.as_str()), "group")?;
        unique(self.categories.iter().map(|x| x.id.as_str()), "category")?;
        unique(self.metrics.iter().map(|x| x.id.as_str()), "metric")?;
        let mut columns = HashSet::new();
        for metric in &self.metrics {
            label(&metric.label, &metric.id)?;
            ensure!(
                !metric.column.trim().is_empty(),
                "{}: column must not be empty",
                metric.id
            );
            ensure!(
                columns.insert(metric.column.trim()),
                "Duplicate KPI column {:?}",
                metric.column
            );
            ensure!(metric.decimals <= 6, "{}: decimals must be 0–6", metric.id);
        }
        for metric in &self.metrics {
            if let Some(breakdown) = &metric.breakdown {
                ensure!(
                    !breakdown.column.trim().is_empty(),
                    "{}: property column must not be empty",
                    metric.id
                );
                ensure!(
                    !columns.contains(breakdown.column.trim()),
                    "{}: property column must differ from numeric KPI columns",
                    metric.id
                );
                ensure!(
                    !breakdown.classes.is_empty(),
                    "{}: at least one KPI class is required",
                    metric.id
                );
                unique(
                    breakdown.classes.iter().map(|class| class.id.as_str()),
                    "KPI class",
                )?;
                let mut values = HashSet::new();
                for class in &breakdown.classes {
                    label(&class.label, &class.id)?;
                    ensure!(
                        !class.value.trim().is_empty(),
                        "{}: class Excel value must not be empty",
                        class.id
                    );
                    ensure!(
                        values.insert(class.value.trim()),
                        "{}: duplicate class Excel value {:?}",
                        metric.id,
                        class.value
                    );
                }
            }
        }
        for group in self.groups.iter().chain(&self.categories) {
            label(&group.label, &group.id)?;
        }
        for (key, value) in &self.translations {
            ensure!(
                crate::locale::is_key(key),
                "Unknown translation key {key:?}"
            );
            ensure!(
                !value.trim().is_empty(),
                "Translation {key:?} must not be empty"
            );
        }
        Ok(())
    }

    /// Validate roster names, identifiers, memberships, and parent relationships.
    /// An empty roster is allowed while setting up a workspace.
    ///
    /// # Errors
    /// Rejects invalid or duplicate IDs, empty names, unknown references, and cycles.
    pub fn validate_projects(&self, roster: &[Project]) -> Result<()> {
        unique(roster.iter().map(|p| p.id.as_str()), "project")?;
        let groups: HashSet<_> = self.groups.iter().map(|g| g.id.as_str()).collect();
        let categories: HashSet<_> = self.categories.iter().map(|c| c.id.as_str()).collect();
        let projects: HashMap<_, _> = roster.iter().map(|p| (p.id.as_str(), p)).collect();
        let mut checked = HashSet::new();
        for project in roster {
            label(&project.label, &project.id)?;
            ensure!(
                project
                    .group
                    .as_deref()
                    .is_none_or(|id| groups.contains(id)),
                "{}: unknown group {:?}",
                project.id,
                project.group
            );
            ensure!(
                project
                    .category
                    .as_deref()
                    .is_none_or(|id| categories.contains(id)),
                "{}: unknown category {:?}",
                project.id,
                project.category
            );
            let mut chain = HashSet::new();
            let mut parent = Some(project.id.as_str());
            while let Some(id) = parent {
                if checked.contains(id) {
                    break;
                }
                ensure!(chain.insert(id), "{}: cycle in parent projects", project.id);
                let node = projects
                    .get(id)
                    .ok_or_else(|| anyhow::anyhow!("{}: unknown parent {id:?}", project.id))?;
                parent = node.parent.as_deref();
            }
            checked.extend(chain);
        }
        Ok(())
    }
}

fn label(value: &str, name: &str) -> Result<()> {
    ensure!(
        !value.trim().is_empty(),
        "{name}: display name must not be empty"
    );
    Ok(())
}

fn unique<'a>(ids: impl Iterator<Item = &'a str>, kind: &str) -> Result<()> {
    let mut seen = HashSet::new();
    for id in ids {
        ensure!(
            !id.is_empty()
                && id.len() <= 80
                && id
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_'),
            "{kind} ID {id:?} must use 1–80 ASCII letters, digits, hyphens, or underscores"
        );
        let lower = id.to_ascii_lowercase();
        if reserved(&lower) {
            bail!("{kind} ID {id:?} is a reserved filename");
        }
        ensure!(
            seen.insert(lower),
            "Duplicate {kind} ID {id:?} (case-insensitive)"
        );
    }
    Ok(())
}

fn reserved(name: &str) -> bool {
    ["con", "prn", "aux", "nul"].contains(&name)
        || (name.len() == 4
            && (name.starts_with("com") || name.starts_with("lpt"))
            && matches!(name.as_bytes()[3], b'1'..=b'9'))
}

pub(super) fn relative(path: &Path) -> Result<()> {
    let text = path.to_string_lossy();
    ensure!(
        !path.as_os_str().is_empty()
            && text.split('/').all(|part| !matches!(part, "." | ".." | ""))
            && path.components().all(|c| matches!(c, Component::Normal(_))),
        "Input path must stay inside the workspace, without '.' or '..': {}",
        path.display()
    );
    ensure!(
        !text.contains(['\\', ':', '*', '?', '<', '>', '"', '|'])
            && !text.chars().any(char::is_control)
            && text.split('/').all(|part| !part.ends_with([' ', '.'])
                && !reserved(&part.split('.').next().unwrap_or("").to_ascii_lowercase())),
        "Use portable forward-slash paths: {text}"
    );
    Ok(())
}
