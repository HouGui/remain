use super::validate::relative;
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Naming conventions shared by all projects. Filenames may contain `*`, which
/// matches any sequence within one filename. Other wildcard syntax is rejected.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InputRules {
    /// Relative project directory, containing exactly one `{id}` placeholder.
    pub directory: String,
    /// Expected documents in reading order. Matches within a PDF pattern use
    /// lexical filename order; use zero-padded numbers. At most one KPI workbook
    /// definition is allowed, and at least one PDF definition must be required.
    pub documents: Vec<DocumentRule>,
}

/// One expected PDF document or KPI workbook, identified by its filename pattern.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentRule {
    /// Display name used in the configuration editor and submission checklist.
    pub label: String,
    /// Filename pattern ending in `.pdf` or `.xlsx`. `*` matches within a filename.
    pub pattern: String,
    /// Whether absence excludes the project's submission from analysis and export.
    pub required: bool,
}

impl DocumentRule {
    /// Whether this definition identifies the project's numeric KPI workbook.
    pub fn is_kpi(&self) -> bool {
        Path::new(&self.pattern)
            .extension()
            .is_some_and(|s| s.eq_ignore_ascii_case("xlsx"))
    }
}

impl Default for InputRules {
    fn default() -> Self {
        Self {
            directory: "{id}".into(),
            documents: vec![DocumentRule {
                label: "Report".into(),
                pattern: "report*.pdf".into(),
                required: true,
            }],
        }
    }
}

impl InputRules {
    pub(super) fn validate(&self) -> Result<()> {
        ensure!(
            self.directory.matches("{id}").count() == 1,
            "inputs.directory must contain exactly one {{id}} placeholder"
        );
        let directory = self.directory.replace("{id}", "project");
        ensure!(
            !directory.contains(['{', '}']),
            "inputs.directory only supports the {{id}} placeholder"
        );
        relative(Path::new(&directory))?;
        let mut seen = std::collections::HashSet::new();
        let mut kpis = 0;
        let mut required_pdf = false;
        for document in &self.documents {
            ensure!(
                !document.label.trim().is_empty(),
                "Document display name must not be empty"
            );
            pattern(
                &document.pattern,
                if document.is_kpi() { "xlsx" } else { "pdf" },
            )?;
            kpis += usize::from(document.is_kpi());
            required_pdf |= document.required && !document.is_kpi();
            ensure!(
                seen.insert(&document.pattern),
                "Duplicate document pattern {:?}",
                document.pattern
            );
        }
        ensure!(kpis <= 1, "Define at most one KPI workbook");
        ensure!(required_pdf, "Define at least one required PDF document");
        Ok(())
    }
}

fn pattern(value: &str, extension: &str) -> Result<()> {
    relative(Path::new(&value.replace('*', "match")))?;
    ensure!(
        !value.contains(['/', '?', '[', ']', '{', '}']) && !value.contains("**"),
        "Use a filename with optional '*' wildcards: {value:?}"
    );
    ensure!(
        Path::new(value)
            .extension()
            .is_some_and(|s| s.eq_ignore_ascii_case(extension)),
        "Expected a .{extension} filename pattern: {value:?}"
    );
    glob::Pattern::new(value)?;
    Ok(())
}
