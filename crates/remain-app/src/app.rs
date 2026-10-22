use crate::{
    appearance::{self, Appearance},
    editor::{Editor, ProjectEditor},
    jobs::{self, Event, Outcome},
    views,
};
use eframe::egui;
use remain_core::{
    Workspace, analysis::Analysis, locale, progress::Progress, submissions::Submission,
};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::mpsc::{Receiver, TryRecvError},
};

type Job = Receiver<Event>;

pub(crate) struct App {
    pub workspace: Option<Workspace>,
    pub analysis: Option<Analysis>,
    pub language: String,
    pub path: String,
    pub loaded_path: Option<PathBuf>,
    pub output: Option<PathBuf>,
    pub error: Option<String>,
    pub editor: Option<Editor>,
    pub project_editor: Option<ProjectEditor>,
    pub submissions: Vec<Submission>,
    pub help: bool,
    pub tab: usize,
    pub chart_metric: usize,
    pub view: views::data::Options,
    pub about: bool,
    pub appearance: Appearance,
    pub logo: Option<egui::TextureHandle>,
    job: Option<Job>,
    operation: &'static str,
    progress: Option<Progress>,
    empty_catalog: BTreeMap<String, String>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let app = Self {
            appearance: Appearance::restore(cc.storage),
            ..Self::default()
        };
        appearance::configure(&cc.egui_ctx);
        app.appearance.apply_theme(&cc.egui_ctx);
        app.appearance.apply_font(&cc.egui_ctx);
        app
    }

    pub fn t(&self, key: &str) -> String {
        let catalog = self
            .workspace
            .as_ref()
            .map(|w| &w.config().translations)
            .unwrap_or(&self.empty_catalog);
        locale::text(&self.language, catalog, key).into()
    }

    pub fn busy(&self) -> bool {
        self.job.is_some()
    }

    pub fn status(&self) -> String {
        if self.busy() {
            return match self.progress {
                Some(Progress::Discovering) => self.t("discovering"),
                Some(Progress::ReadingProject { current, total }) => {
                    format!("{} {current} / {total}", self.t("reading_project"))
                }
                Some(Progress::Summarizing) => self.t("summarizing"),
                Some(Progress::AssemblingPdfs) => self.t("assembling_pdfs"),
                Some(Progress::WritingWorkbook) => self.t("writing_workbook"),
                Some(Progress::WritingSummary) => self.t("writing_summary"),
                Some(Progress::Publishing) => self.t("publishing"),
                None => self.t(self.operation),
            };
        }
        self.t(if self.error.is_some() {
            "operation_failed"
        } else if self.output.is_some() {
            "complete"
        } else if self.analysis.as_ref().is_some_and(|a| !a.is_complete()) {
            "analysis_incomplete"
        } else if self.analysis.is_some() {
            "analysis_ready"
        } else {
            "ready"
        })
    }

    pub fn open_folder(&mut self, folder: PathBuf) {
        self.load(folder.join("workspace.json"));
    }

    pub fn load(&mut self, path: PathBuf) {
        match Workspace::load(&path) {
            Ok(workspace) => self.replace_workspace(path, Some(workspace)),
            Err(error) => {
                self.replace_workspace(path, None);
                self.error = Some(format!("{error:#}"));
            }
        }
    }

    fn reload(&mut self, path: PathBuf) {
        let language = self.language.clone();
        self.load(path);
        self.language = language;
    }

    fn replace_workspace(&mut self, path: PathBuf, workspace: Option<Workspace>) {
        self.analysis = None;
        self.chart_metric = 0;
        self.view = views::data::Options::default();
        self.output = None;
        self.editor = None;
        self.project_editor = None;
        self.submissions.clear();
        self.error = None;
        self.path = path.parent().unwrap_or(&path).to_string_lossy().into();
        self.loaded_path = workspace.as_ref().map(|_| path);
        if let Some(workspace) = &workspace {
            self.language = workspace.config().locale.clone();
            match workspace.submissions() {
                Ok(submissions) => self.submissions = submissions,
                Err(error) => self.error = Some(format!("{error:#}")),
            }
        }
        self.workspace = workspace;
    }

    pub fn analyze(&mut self, context: &egui::Context) {
        let Some(path) = self.loaded_path.clone() else {
            return;
        };
        self.analysis = None;
        self.error = None;
        self.output = None;
        self.operation = "loading_workspace";
        self.progress = None;
        self.job = Some(jobs::start(context.clone(), move |progress| {
            let workspace = Workspace::load(path)?;
            let analysis = workspace.analyze_with_progress(progress)?;
            Ok(Outcome::Analyzed(workspace, analysis))
        }));
    }

    pub fn export(&mut self, context: &egui::Context) {
        let Some(path) = self.loaded_path.clone() else {
            return;
        };
        let Some(parent) = rfd::FileDialog::new()
            .set_title(self.t("destination"))
            .pick_folder()
        else {
            return;
        };
        let destination = jobs::unused_directory(&parent, "remain-report");
        self.analysis = None;
        self.error = None;
        self.output = None;
        self.operation = "loading_workspace";
        self.progress = None;
        self.job = Some(jobs::start(context.clone(), move |progress| {
            let workspace = Workspace::load(path)?;
            let analysis = workspace.export_with_progress(&destination, progress)?;
            Ok(Outcome::Exported(destination, workspace, analysis))
        }));
    }

    pub fn demo(&mut self, context: &egui::Context, with_classes: bool) {
        let Some(parent) = rfd::FileDialog::new()
            .set_title(self.t("demo_destination"))
            .pick_folder()
        else {
            return;
        };
        let destination = jobs::unused_directory(&parent, "remain-demo");
        let language = locale::builtin(&self.language).map_or("en", |language| language.code);
        self.error = None;
        self.operation = "creating_demo";
        self.progress = None;
        self.job = Some(jobs::start(context.clone(), move |_| {
            let path = if with_classes {
                remain_core::demo::create_classified(destination, language)?
            } else {
                remain_core::demo::create(destination, language)?
            };
            let workspace = Workspace::load(&path)?;
            Ok(Outcome::Demo(path, workspace))
        }));
    }

    fn accept_analysis(&mut self, workspace: Workspace, analysis: Analysis) {
        self.chart_metric = self
            .chart_metric
            .min(analysis.metric_ids.len().saturating_sub(1));
        self.submissions = analysis.submissions.clone();
        self.workspace = Some(workspace);
        self.analysis = Some(analysis);
    }

    fn poll(&mut self) {
        let Some(job) = &self.job else {
            return;
        };
        let result = loop {
            match job.try_recv() {
                Ok(Event::Progress(progress)) => self.progress = Some(progress),
                Ok(Event::Finished(result)) => break *result,
                Err(TryRecvError::Empty) => return,
                Err(TryRecvError::Disconnected) => {
                    break Err(
                        "The worker stopped unexpectedly. Reopen the workspace and try again."
                            .into(),
                    );
                }
            }
        };
        self.progress = None;
        self.job = None;
        match result {
            Ok(Outcome::Demo(path, workspace)) => {
                self.replace_workspace(path, Some(workspace));
            }
            Ok(Outcome::Analyzed(workspace, analysis)) => self.accept_analysis(workspace, analysis),
            Ok(Outcome::Exported(path, workspace, analysis)) => {
                self.output = Some(path);
                self.accept_analysis(workspace, analysis);
            }
            Err(error) => self.error = Some(error),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            workspace: None,
            analysis: None,
            language: "en".into(),
            path: String::new(),
            loaded_path: None,
            output: None,
            error: None,
            editor: None,
            project_editor: None,
            submissions: Vec::new(),
            help: false,
            tab: 0,
            chart_metric: 0,
            view: views::data::Options::default(),
            about: false,
            appearance: Appearance::default(),
            logo: None,
            job: None,
            operation: "ready",
            progress: None,
            empty_catalog: BTreeMap::new(),
        }
    }
}

impl eframe::App for App {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.appearance.save(storage);
    }

    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll();
        views::show(self, context);
        if let Some(editor) = &mut self.editor {
            let result = editor.show(context);
            match result {
                Some(Ok(path)) => self.reload(path),
                Some(Err(error)) => self.error = Some(error),
                None => {}
            }
            if self.editor.as_ref().is_some_and(|editor| !editor.open) {
                self.editor = None;
            }
        }
        if let Some(editor) = &mut self.project_editor {
            let saved = editor.show(context);
            if !editor.open {
                self.project_editor = None;
            }
            if saved && let Some(path) = self.loaded_path.clone() {
                self.reload(path);
            }
        }
        crate::help::show(self, context);
        if self.busy() {
            context.request_repaint_after(std::time::Duration::from_millis(100));
        }
    }
}

#[cfg(test)]
#[path = "app_tests.rs"]
mod tests;
