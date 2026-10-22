//! Deterministic discovery and completeness checks for project submissions.

use crate::{
    config::{Config, Project},
    workspace::input_path,
};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

/// The files found for one project and the expected patterns still missing.
/// Completeness describes file presence; analysis separately validates content.
/// Discovered paths are workspace-relative and use forward slashes on every platform.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Submission {
    /// Stable project identifier from the roster.
    pub project: String,
    /// Discovered annex path relative to the workspace, if present.
    pub annex: Option<PathBuf>,
    /// Discovered PDF paths in configured pattern order, then lexical filename order.
    pub reports: Vec<PathBuf>,
    /// Document names and workspace-relative patterns with no required match.
    pub missing: Vec<String>,
    /// Names and patterns of optional documents that have not been submitted; these do not
    /// exclude the project or make the assembled report incomplete.
    pub optional_missing: Vec<String>,
}

impl Submission {
    /// Whether every required input pattern has a match. This does not certify
    /// that the file contents are valid; use [`crate::Workspace::analyze`].
    pub fn is_complete(&self) -> bool {
        self.missing.is_empty()
    }
}

pub(crate) fn discover(
    root: &Path,
    config: &Config,
    projects: &[Project],
) -> Result<Vec<Submission>> {
    let patterns = config
        .inputs
        .documents
        .iter()
        .map(|d| glob::Pattern::new(&d.pattern))
        .collect::<Result<Vec<_>, _>>()?;
    let mut seen = HashSet::new();
    projects
        .iter()
        .map(|project| {
            let directory = config.inputs.directory.replace("{id}", &project.id);
            let absolute = root.join(&directory);
            let mut files = match fs::canonicalize(&absolute) {
                Ok(path) => {
                    ensure!(
                        path.starts_with(root) && path.is_dir(),
                        "Project directory must stay inside the workspace: {}",
                        directory
                    );
                    fs::read_dir(path)?
                        .map(|entry| entry.map(|e| e.file_name()))
                        .collect::<std::io::Result<Vec<_>>>()?
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
                Err(error) => {
                    return Err(error).with_context(|| format!("Cannot scan {directory}"));
                }
            };
            files.sort();
            let mut submission = Submission {
                project: project.id.clone(),
                annex: None,
                reports: Vec::new(),
                missing: Vec::new(),
                optional_missing: Vec::new(),
            };
            for (definition, pattern) in config.inputs.documents.iter().zip(&patterns) {
                let matches = files
                    .iter()
                    .filter_map(|name| name.to_str())
                    .filter(|name| pattern.matches(name))
                    .collect::<Vec<_>>();
                if matches.is_empty() {
                    let missing = if definition.required {
                        &mut submission.missing
                    } else {
                        &mut submission.optional_missing
                    };
                    missing.push(format!(
                        "{}: {}/{}",
                        definition.label,
                        directory,
                        pattern.as_str()
                    ));
                }
                ensure!(
                    !definition.is_kpi() || matches.len() <= 1,
                    "{}: annex pattern {:?} matches multiple files; keep exactly one submission",
                    project.id,
                    definition.pattern
                );
                for name in matches {
                    // Keep serialized and displayed source paths portable; Path::join
                    // would insert a platform-specific separator on Windows.
                    let path = PathBuf::from(format!("{directory}/{name}"));
                    let canonical = input_path(root, &path)?;
                    ensure!(
                        seen.insert(canonical),
                        "Input matched more than once (overlapping patterns or symlinks): {}",
                        path.display()
                    );
                    if definition.is_kpi() {
                        submission.annex = Some(path);
                    } else {
                        submission.reports.push(path);
                    }
                }
            }
            Ok(submission)
        })
        .collect()
}
