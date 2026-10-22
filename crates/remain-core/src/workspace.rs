use crate::{
    analysis::{Analysis, ProjectSummary, scopes, summarize},
    config::{Config, Project},
    excel, pdf,
    progress::Progress,
    projects,
    submissions::{self, Submission},
};
use anyhow::{Context, Result, ensure};
use lopdf::Document;
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

/// An immutable configuration rooted at its workspace directory.
///
/// Paths are resolved relative to the configuration file, independent of the
/// process's current directory. Input symlinks must resolve inside this root.
#[derive(Clone, Debug)]
pub struct Workspace {
    pub(crate) root: PathBuf,
    pub(crate) config: Config,
    pub(crate) projects: Vec<Project>,
}

pub(crate) struct Prepared {
    pub analysis: Analysis,
    pub documents: Vec<Document>,
}

impl Workspace {
    /// Read and validate a UTF-8 JSON workspace configuration.
    ///
    /// # Errors
    /// Fails for unreadable settings or roster files, unknown fields, an invalid
    /// schema, or malformed metadata. Input documents are checked when analyzing or exporting.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = fs::canonicalize(path.as_ref())
            .with_context(|| format!("Cannot open workspace {}", path.as_ref().display()))?;
        let config: Config = serde_json::from_slice(&fs::read(&path)?)
            .with_context(|| format!("Invalid workspace JSON: {}", path.display()))?;
        let root = path.parent().context("Workspace has no parent directory")?;
        Self::new(root, config)
    }

    /// Construct a workspace from configuration, for embedding or integrations.
    ///
    /// # Errors
    /// Returns an error if the root is not an existing directory or the
    /// settings or referenced roster do not satisfy configuration validation.
    pub fn new(root: impl AsRef<Path>, config: Config) -> Result<Self> {
        config.validate()?;
        let root = fs::canonicalize(root).context("Cannot resolve workspace directory")?;
        ensure!(root.is_dir(), "Workspace root must be a directory");
        let projects = projects::load(input_path(&root, &config.projects)?)?;
        config.validate_projects(&projects)?;
        Ok(Self {
            root,
            config,
            projects,
        })
    }

    /// Construct a workspace using an in-memory roster, for integrations.
    ///
    /// # Errors
    /// Rejects invalid settings, roster references, or a nonexistent root directory.
    pub fn with_projects(
        root: impl AsRef<Path>,
        config: Config,
        projects: Vec<Project>,
    ) -> Result<Self> {
        config.validate()?;
        config.validate_projects(&projects)?;
        let root = fs::canonicalize(root).context("Cannot resolve workspace directory")?;
        ensure!(root.is_dir(), "Workspace root must be a directory");
        Ok(Self {
            root,
            config,
            projects,
        })
    }

    /// Return the roster in its source order.
    pub fn projects(&self) -> &[Project] {
        &self.projects
    }

    /// Discover expected files and identify every project with missing submissions.
    /// File contents are not parsed by this inexpensive presence check.
    ///
    /// # Errors
    /// Rejects ambiguous annexes, overlapping report patterns, inaccessible folders,
    /// and paths or symlinks that escape the workspace or reuse an input.
    pub fn submissions(&self) -> Result<Vec<Submission>> {
        submissions::discover(&self.root, &self.config, &self.projects)
    }

    /// Return the validated configuration.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Return the absolute directory used to resolve input files.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Discover submissions, validate complete projects, and aggregate their KPIs.
    /// This method never writes files or reuses stale input results.
    ///
    /// # Errors
    /// Stops on invalid file content or unsafe paths with project/file context.
    /// Missing submissions are recorded in the analysis and excluded in full;
    /// their absence is never interpreted as zero.
    pub fn analyze(&self) -> Result<Analysis> {
        self.analyze_with_progress(|_| {})
    }

    /// Analyze inputs while reporting operations through a lightweight callback.
    ///
    /// # Errors
    /// Returns the same validation and input errors as [`Self::analyze`].
    pub fn analyze_with_progress(&self, mut progress: impl FnMut(Progress)) -> Result<Analysis> {
        Ok(self.prepare(&mut progress)?.analysis)
    }

    /// Create an entire report bundle in a new directory.
    ///
    /// Inputs are read again and output is staged on the destination filesystem.
    /// An existing destination is never overwritten. On failure the temporary
    /// staging directory is removed. See the user guide for bundle contents.
    ///
    /// # Errors
    /// Fails for invalid inputs, an existing destination, an unwritable parent,
    /// no complete submissions, or a serialization, workbook, or PDF error.
    pub fn export(&self, destination: impl AsRef<Path>) -> Result<Analysis> {
        self.export_with_progress(destination, |_| {})
    }

    /// Export a bundle while reporting the current operation.
    ///
    /// # Errors
    /// Returns the same validation, filesystem, and rendering errors as [`Self::export`].
    pub fn export_with_progress(
        &self,
        destination: impl AsRef<Path>,
        mut progress: impl FnMut(Progress),
    ) -> Result<Analysis> {
        crate::export::bundle(self, destination.as_ref(), &mut progress)
    }

    fn input(&self, relative: &Path, seen: &mut HashSet<PathBuf>) -> Result<PathBuf> {
        let path = input_path(&self.root, relative)?;
        ensure!(
            seen.insert(path.clone()),
            "Input assigned more than once (including symlinks): {}",
            relative.display()
        );
        Ok(path)
    }

    pub(crate) fn prepare(&self, progress: &mut dyn FnMut(Progress)) -> Result<Prepared> {
        progress(Progress::Discovering);
        let submissions = self.submissions()?;
        let files: HashMap<_, _> = submissions
            .iter()
            .map(|s| (s.project.as_str(), s))
            .collect();
        let mut observations = Vec::new();
        let mut projects = Vec::new();
        let mut documents = Vec::new();
        let mut seen = HashSet::new();
        let order: HashMap<_, _> = self
            .config
            .groups
            .iter()
            .enumerate()
            .map(|(i, g)| (g.id.as_str(), i))
            .collect();
        let mut ordered: Vec<_> = self
            .projects
            .iter()
            .filter(|p| files[p.id.as_str()].is_complete())
            .collect();
        ordered.sort_by_key(|p| {
            p.group
                .as_deref()
                .and_then(|id| order.get(id))
                .copied()
                .unwrap_or(order.len())
        });
        let total = ordered.len();
        for (index, project) in ordered.into_iter().enumerate() {
            progress(Progress::ReadingProject {
                current: index + 1,
                total,
            });
            let submission = files[project.id.as_str()];
            let rows = if let Some(path) = &submission.annex {
                excel::read_annex(&self.input(path, &mut seen)?, &self.config, project)?
            } else {
                Vec::new()
            };
            let mut parts = Vec::new();
            for path in &submission.reports {
                parts.push(pdf::read(&self.input(path, &mut seen)?)?);
            }
            let document = pdf::merge(parts)?;
            projects.push(ProjectSummary {
                id: project.id.clone(),
                group: project.group.clone(),
                category: project.category.clone(),
                rows: rows.len(),
                has_kpis: submission.annex.is_some(),
                pages: document.get_pages().len(),
                values: if submission.annex.is_some() {
                    summarize(&self.config, &rows.iter().collect::<Vec<_>>())?
                } else {
                    vec![None; self.config.indicators().len()]
                },
            });
            documents.push(document);
            observations.extend(rows);
        }
        progress(Progress::Summarizing);
        let groups = scopes(
            &self.config,
            &self.projects,
            &projects,
            &self.config.groups,
            &observations,
            |p| p.group.as_deref(),
        )?;
        let categories = scopes(
            &self.config,
            &self.projects,
            &projects,
            &self.config.categories,
            &observations,
            |p| p.category.as_deref(),
        )?;
        let overall = if projects.iter().any(|p| p.has_kpis) {
            summarize(&self.config, &observations.iter().collect::<Vec<_>>())?
        } else {
            vec![None; self.config.indicators().len()]
        };
        let analysis = Analysis {
            metric_ids: self
                .config
                .indicators()
                .into_iter()
                .map(|indicator| indicator.id)
                .collect(),
            projects,
            groups,
            categories,
            overall,
            observations,
            submissions,
        };
        Ok(Prepared {
            analysis,
            documents,
        })
    }
}

/// Resolve an existing file without letting symlinks escape the workspace.
pub(crate) fn input_path(root: &Path, relative: &Path) -> Result<PathBuf> {
    let path = fs::canonicalize(root.join(relative))
        .with_context(|| format!("Missing input {}", relative.display()))?;
    ensure!(
        path.starts_with(root) && path.is_file(),
        "Input must be a file inside the workspace: {}",
        relative.display()
    );
    Ok(path)
}
