mod html;

use crate::{Workspace, analysis::Analysis, excel, pdf, progress::Progress, workspace::input_path};
use anyhow::{Context, Result, ensure};
use std::{fs, path::Path};
use tempfile::TempDir;

pub(crate) fn bundle(
    workspace: &Workspace,
    destination: &Path,
    progress: &mut dyn FnMut(Progress),
) -> Result<Analysis> {
    ensure!(
        !destination.exists(),
        "Destination already exists: {}. Choose a new folder.",
        destination.display()
    );
    let parent = destination
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    ensure!(
        parent.is_dir(),
        "Destination parent must be an existing directory"
    );
    let prepared = workspace.prepare(progress)?;
    ensure!(
        !prepared.analysis.projects.is_empty(),
        "No complete submissions to export. Add the required documents for at least one project."
    );
    let font = workspace
        .config
        .contents_font
        .as_ref()
        .map(|path| fs::read(input_path(workspace.root(), path)?).map_err(anyhow::Error::from))
        .transpose()?;
    let font = font.as_deref().unwrap_or(pdf::DEFAULT_FONT);
    let names: std::collections::HashMap<_, _> = workspace
        .projects()
        .iter()
        .map(|p| (p.id.as_str(), p.label.as_str()))
        .collect();
    let assemble = |scope: &str, selected: &dyn Fn(&crate::config::Project) -> bool| {
        let ids: std::collections::HashSet<_> = workspace
            .projects()
            .iter()
            .filter(|p| selected(p))
            .map(|p| p.id.as_str())
            .collect();
        let missing = prepared
            .analysis
            .submissions
            .iter()
            .filter(|s| ids.contains(s.project.as_str()) && !s.is_complete())
            .map(|s| {
                format!(
                    "{} ({}): {}",
                    names[s.project.as_str()],
                    s.project,
                    s.missing.join(", ")
                )
            })
            .collect::<Vec<_>>();
        let included = prepared
            .analysis
            .projects
            .iter()
            .zip(&prepared.documents)
            .filter(|(p, _)| ids.contains(p.id.as_str()))
            .collect::<Vec<_>>();
        let coverage = (
            included.iter().filter(|(p, _)| p.has_kpis).count(),
            ids.len(),
        );
        let entries = included
            .into_iter()
            .map(|(p, d)| (format!("{} ({})", names[p.id.as_str()], p.id), d.clone()))
            .collect();
        pdf::assemble(&workspace.config, scope, entries, &missing, coverage, font)
    };
    let stage = tempfile::Builder::new()
        .prefix(".remain-")
        .tempdir_in(parent)?;
    let root = stage.path();
    fs::create_dir(root.join("projects"))?;
    fs::create_dir(root.join("groups"))?;
    fs::create_dir(root.join("categories"))?;
    progress(Progress::AssemblingPdfs);
    for (project, document) in prepared.analysis.projects.iter().zip(&prepared.documents) {
        document
            .clone()
            .save(root.join("projects").join(format!("{}.pdf", project.id)))?;
    }
    for (directory, definitions) in [
        ("groups", &workspace.config.groups),
        ("categories", &workspace.config.categories),
    ] {
        for group in definitions {
            let selected = |p: &crate::config::Project| {
                let id = if directory == "groups" {
                    &p.group
                } else {
                    &p.category
                };
                id.as_deref() == Some(group.id.as_str())
            };
            if workspace.projects().iter().any(selected) {
                assemble(&group.label, &selected)?
                    .save(root.join(directory).join(format!("{}.pdf", group.id)))?;
            }
        }
    }
    assemble(
        crate::locale::text(
            &workspace.config.locale,
            &workspace.config.translations,
            "total",
        ),
        &|_| true,
    )?
    .save(root.join("report.pdf"))?;
    if !workspace.config.metrics.is_empty() {
        progress(Progress::WritingWorkbook);
        excel::write_annex(&root.join("annex.xlsx"), workspace, &prepared.analysis)?;
    }
    progress(Progress::WritingSummary);
    fs::write(
        root.join("index.html"),
        html::render(workspace, &prepared.analysis),
    )?;
    #[derive(serde::Serialize)]
    struct Summary<'a> {
        complete: bool,
        #[serde(flatten)]
        analysis: &'a Analysis,
    }
    fs::write(
        root.join("summary.json"),
        serde_json::to_vec_pretty(&Summary {
            complete: prepared.analysis.is_complete(),
            analysis: &prepared.analysis,
        })?,
    )?;
    fs::write(
        root.join("project-list.json"),
        serde_json::to_vec_pretty(workspace.projects())?,
    )?;
    fs::write(
        root.join("configuration.json"),
        serde_json::to_vec_pretty(&workspace.config)?,
    )?;
    progress(Progress::Publishing);
    publish(stage, destination)?;
    Ok(prepared.analysis)
}

/// Reserve a new directory exclusively, then move completed files into it.
/// Keeping staging beside the output makes these renames filesystem-local.
pub(crate) fn publish(stage: TempDir, destination: &Path) -> Result<()> {
    fs::create_dir(destination).with_context(|| {
        format!(
            "Cannot create a new output folder: {}",
            destination.display()
        )
    })?;
    let result = (|| -> Result<()> {
        for entry in fs::read_dir(stage.path())? {
            let entry = entry?;
            fs::rename(entry.path(), destination.join(entry.file_name()))?;
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(destination);
    }
    result.context("Cannot publish report files")
}
