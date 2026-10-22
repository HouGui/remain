use super::fields;
use eframe::egui;
use remain_core::config::{Classification, Config, Project};
use remain_core::{Workspace, locale};
use std::path::PathBuf;

pub(super) fn show(
    ui: &mut egui::Ui,
    config: &Config,
    projects: &mut Vec<Project>,
    t: &impl Fn(&str) -> String,
) {
    ui.heading(t("projects"));
    let mut remove = None;
    let mut reorder = None;
    let length = projects.len();
    for (i, project) in projects.iter_mut().enumerate() {
        egui::CollapsingHeader::new(format!("{} · {}", project.id, project.label))
            .id_salt(("project", i))
            .default_open(i == 0)
            .show(ui, |ui| {
                fields::text(ui, &t("id"), &mut project.id);
                fields::text(ui, &t("label"), &mut project.label);
                assignment(ui, "group", &mut project.group, &config.groups, t);
                assignment(ui, "category", &mut project.category, &config.categories, t);
                let mut parent = project.parent.clone().unwrap_or_default();
                fields::text(ui, &t("parent"), &mut parent);
                project.parent = if parent.trim().is_empty() {
                    None
                } else {
                    Some(parent)
                };
                fields::actions(ui, i, length, t, &mut remove, &mut reorder);
            });
    }
    fields::apply(projects, remove, reorder);
    if ui.button(t("add")).clicked() {
        projects.push(Project {
            id: fields::next_id("project", projects.iter().map(|p| p.id.as_str())),
            label: t("project"),
            group: None,
            category: None,
            parent: None,
        });
    }
}

fn assignment(
    ui: &mut egui::Ui,
    key: &str,
    value: &mut Option<String>,
    items: &[Classification],
    t: &impl Fn(&str) -> String,
) {
    let caption = value
        .as_deref()
        .and_then(|id| items.iter().find(|item| item.id == id))
        .map(|item| item.label.clone())
        .unwrap_or_else(|| value.clone().unwrap_or_else(|| t("unassigned")));
    fields::row(ui, &t(key), |ui| {
        egui::ComboBox::from_id_salt(key)
            .selected_text(caption)
            .show_ui(ui, |ui| {
                ui.selectable_value(value, None, t("unassigned"));
                for item in items {
                    ui.selectable_value(value, Some(item.id.clone()), item.label.as_str());
                }
            });
    });
}

pub(crate) struct ProjectEditor {
    pub open: bool,
    workspace: Workspace,
    config_path: PathBuf,
    projects: Vec<Project>,
    original: Option<Vec<u8>>,
    language: String,
    error: Option<String>,
}

impl ProjectEditor {
    pub fn new(workspace: &Workspace, config_path: PathBuf, language: &str) -> Self {
        let path = workspace.root().join(&workspace.config().projects);
        let original = std::fs::read(&path).ok().filter(|_| {
            remain_core::projects::load(&path)
                .ok()
                .and_then(|p| serde_json::to_value(p).ok())
                == serde_json::to_value(workspace.projects()).ok()
        });
        Self {
            open: true,
            workspace: workspace.clone(),
            config_path,
            projects: workspace.projects().to_vec(),
            original,
            language: language.into(),
            error: None,
        }
    }

    pub fn show(&mut self, context: &egui::Context) -> bool {
        let mut open = self.open;
        let mut saved = false;
        let language = self.language.clone();
        let catalog = self.workspace.config().translations.clone();
        let t = |key: &str| locale::text(&language, &catalog, key).to_owned();
        egui::Window::new(t("project_list"))
            .open(&mut open)
            .default_width((context.content_rect().width() - 48.0).min(740.0))
            .max_width((context.content_rect().width() - 48.0).max(280.0))
            .default_height(600.0)
            .show(context, |ui| {
                ui.set_max_width((context.content_rect().width() - 64.0).max(260.0));
                ui.add(egui::Label::new(t("project_list_hint")).wrap());
                if ui.button(t("import_projects")).clicked()
                    && let Some(path) = rfd::FileDialog::new()
                        .add_filter("CSV / JSON", &["csv", "json"])
                        .pick_file()
                {
                    match remain_core::projects::load(path) {
                        Ok(projects) => {
                            self.projects = projects;
                            self.error = None;
                        }
                        Err(error) => self.error = Some(format!("{error:#}")),
                    }
                }
                if let Some(error) = &self.error {
                    ui.colored_label(ui.visuals().error_fg_color, error);
                }
                egui::ScrollArea::vertical()
                    .max_height((ui.available_height() - 60.0).max(120.0))
                    .show(ui, |ui| {
                        show(ui, self.workspace.config(), &mut self.projects, &t);
                    });
                ui.horizontal(|ui| {
                    if ui.button(t("save_projects")).clicked() {
                        match self.save() {
                            Ok(()) => {
                                self.open = false;
                                saved = true;
                            }
                            Err(error) => self.error = Some(format!("{error:#}")),
                        }
                    }
                    if ui.button(t("cancel")).clicked() {
                        self.open = false;
                    }
                });
            });
        self.open &= open;
        saved
    }

    fn save(&self) -> anyhow::Result<()> {
        let current = Workspace::load(&self.config_path)?;
        anyhow::ensure!(
            serde_json::to_value(current.config())?
                == serde_json::to_value(self.workspace.config())?,
            "Workspace settings changed on disk. Reopen the workspace before saving the project list."
        );
        current.config().validate_projects(&self.projects)?;
        let path = current.root().join(&current.config().projects);
        super::storage::save(
            &path,
            self.original.as_deref(),
            &remain_core::projects::encode(&path, &self.projects)?,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_edits_save_separately_and_detect_stale_rosters() -> anyhow::Result<()> {
        let tmp = tempfile::tempdir()?;
        let path = remain_core::demo::create(tmp.path().join("demo"), "en")?;
        let workspace = Workspace::load(&path)?;
        let config_bytes = std::fs::read(&path)?;
        let mut editor = ProjectEditor::new(&workspace, path.clone(), "fr");
        let context = egui::Context::default();
        let output = context.run(Default::default(), |ctx| {
            editor.show(ctx);
        });
        assert!(!output.shapes.is_empty());
        editor.projects[0].label = "Énergie, recherche".into();
        editor.save()?;
        assert_eq!(std::fs::read(&path)?, config_bytes);
        assert_eq!(
            Workspace::load(&path)?.projects()[0].label,
            "Énergie, recherche"
        );
        assert!(editor.save().is_err());
        let stale = ProjectEditor::new(&workspace, path.clone(), "en");
        assert!(stale.save().is_err());
        let current = Workspace::load(&path)?;
        let mut editor = ProjectEditor::new(&current, path.clone(), "en");
        editor.projects[0].group = Some("unknown".into());
        assert!(editor.save().is_err());
        assert_eq!(
            Workspace::load(path)?.projects()[0].group,
            workspace.projects()[0].group
        );
        Ok(())
    }
}
