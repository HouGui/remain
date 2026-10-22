mod breakdown;
mod classifications;
mod documents;
mod fields;
mod metrics;
mod projects;
mod storage;
pub(crate) use projects::ProjectEditor;

use eframe::egui;
use remain_core::{
    Workspace,
    config::{Config, InputRules},
    locale,
};
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Clone, Copy, Default, PartialEq)]
pub(crate) enum Section {
    #[default]
    Workspace,
    Metrics,
    Documents,
    Groups,
    Categories,
}

pub(crate) struct Editor {
    pub open: bool,
    pub section: Section,
    config: Config,
    root: PathBuf,
    path: PathBuf,
    language: String,
    error: Option<String>,
    original: Option<Vec<u8>>,
}

impl Editor {
    pub fn new(workspace: &Workspace, path: PathBuf, language: &str) -> Self {
        // A workspace may have changed since loading, even before opening this form.
        let original = std::fs::read(&path).ok().filter(|bytes| {
            serde_json::from_slice::<Config>(bytes)
                .ok()
                .and_then(|config| serde_json::to_value(config).ok())
                == serde_json::to_value(workspace.config()).ok()
        });
        Self {
            open: true,
            section: Section::Workspace,
            config: workspace.config().clone(),
            root: workspace.root().into(),
            original,
            path,
            language: language.into(),
            error: None,
        }
    }

    pub fn empty(root: PathBuf, language: &str) -> Self {
        Self {
            open: true,
            section: Section::Workspace,
            path: root.join("workspace.json"),
            root,
            language: language.into(),
            error: None,
            original: None,
            config: Config {
                schema_version: remain_core::config::SCHEMA_VERSION,
                title: "Portfolio".into(),
                locale: language.into(),
                period: String::new(),
                annex: None,
                contents_font: None,
                inputs: InputRules::default(),
                groups: Vec::new(),
                categories: Vec::new(),
                projects: "projects.csv".into(),
                metrics: Vec::new(),
                translations: BTreeMap::new(),
            },
        }
    }

    pub fn show(&mut self, context: &egui::Context) -> Option<Result<PathBuf, String>> {
        let catalog = self.config.translations.clone();
        let language = self.language.clone();
        let t = |key: &str| locale::text(&language, &catalog, key).to_owned();
        let mut open = self.open;
        let mut result = None;
        egui::Window::new(t("edit"))
            .open(&mut open)
            .default_width((context.content_rect().width() - 48.0).min(740.0))
            .max_width((context.content_rect().width() - 48.0).max(280.0))
            .default_height(600.0)
            .show(context, |ui| {
                ui.set_max_width((context.content_rect().width() - 64.0).max(260.0));
                ui.spacing_mut().interact_size.y = 24.0;
                ui.spacing_mut().button_padding = egui::vec2(10.0, 5.0);
                ui.spacing_mut().item_spacing.y = 7.0;
                ui.label(self.root.display().to_string());
                ui.horizontal_wrapped(|ui| {
                    for (section, key) in [
                        (Section::Workspace, "workspace"),
                        (Section::Metrics, "metrics"),
                        (Section::Groups, "groups"),
                        (Section::Categories, "categories"),
                        (Section::Documents, "documents"),
                    ] {
                        ui.selectable_value(&mut self.section, section, t(key));
                    }
                });
                ui.separator();
                if let Some(error) = &self.error {
                    ui.colored_label(ui.visuals().error_fg_color, error);
                }
                egui::ScrollArea::vertical()
                    .id_salt(self.section as usize)
                    .max_height((ui.available_height() - 58.0).max(120.0))
                    .show(ui, |ui| match self.section {
                        Section::Workspace => {
                            ui.small(t("workspace_hint"));
                            fields::text(ui, &t("language"), &mut self.config.locale);
                            fields::text(ui, &t("title"), &mut self.config.title);
                            fields::text(ui, &t("period"), &mut self.config.period);
                            let mut projects = self.config.projects.to_string_lossy().into_owned();
                            fields::text(ui, &t("project_list"), &mut projects);
                            self.config.projects = projects.into();
                        }
                        Section::Groups => classifications::show(
                            ui,
                            &mut self.config.groups,
                            "group",
                            "groups",
                            &t,
                        ),
                        Section::Categories => classifications::show(
                            ui,
                            &mut self.config.categories,
                            "category",
                            "categories",
                            &t,
                        ),
                        Section::Metrics => {
                            if self.config.annex.is_some() {
                                metrics::show(ui, &mut self.config, &t);
                            } else {
                                ui.label(t("no_kpis"));
                            }
                        }
                        Section::Documents => documents::show(ui, &mut self.config, &t),
                    });
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button(t("save")).clicked() {
                        match self.save() {
                            Ok(()) => {
                                self.open = false;
                                result = Some(Ok(self.path.clone()));
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
        result
    }

    fn save(&self) -> anyhow::Result<()> {
        self.config.validate()?;
        let roster = self.root.join(&self.config.projects);
        if !roster.exists() && self.original.is_none() {
            storage::verify(&self.path, None)?;
            let parent = std::fs::canonicalize(roster.parent().expect("relative roster path"))?;
            anyhow::ensure!(
                parent.starts_with(std::fs::canonicalize(&self.root)?),
                "Project list must stay inside the workspace"
            );
            storage::save(&roster, None, &remain_core::projects::encode(&roster, &[])?)?;
        }
        Workspace::new(&self.root, self.config.clone())?;
        storage::save(
            &self.path,
            self.original.as_deref(),
            &serde_json::to_vec_pretty(&self.config)?,
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
