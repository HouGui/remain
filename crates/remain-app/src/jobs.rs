use eframe::egui;
use remain_core::{Workspace, analysis::Analysis, progress::Progress};
use std::{
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver},
};

pub(crate) enum Outcome {
    Demo(PathBuf, Workspace),
    Analyzed(Workspace, Analysis),
    Exported(PathBuf, Workspace, Analysis),
}

pub(crate) enum Event {
    Progress(Progress),
    Finished(Box<Result<Outcome, String>>),
}

pub(crate) fn start(
    context: egui::Context,
    work: impl FnOnce(&mut dyn FnMut(Progress)) -> anyhow::Result<Outcome> + Send + 'static,
) -> Receiver<Event> {
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let mut progress = |progress| {
            let _ = sender.send(Event::Progress(progress));
            context.request_repaint();
        };
        let result = work(&mut progress).map_err(|e| format!("{e:#}"));
        let _ = sender.send(Event::Finished(Box::new(result)));
        context.request_repaint();
    });
    receiver
}

pub(crate) fn unused_directory(parent: &Path, name: &str) -> PathBuf {
    let mut path = parent.join(name);
    let mut suffix = 2;
    while path.exists() {
        path = parent.join(format!("{name}-{suffix}"));
        suffix += 1;
    }
    path
}

pub(crate) fn open_directory(path: &Path) -> anyhow::Result<()> {
    #[cfg(target_os = "windows")]
    let mut command = std::process::Command::new("explorer");
    #[cfg(target_os = "macos")]
    let mut command = std::process::Command::new("open");
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let mut command = std::process::Command::new("xdg-open");
    command.arg(path).spawn()?;
    Ok(())
}

pub(crate) fn guide() -> anyhow::Result<PathBuf> {
    let executable = std::env::current_exe()?;
    let directory = executable
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Cannot locate the application"))?;
    let mut candidates = vec![
        directory.join("guide/index.html"),
        directory.join("../share/doc/remain/guide/index.html"),
        directory.join("../Resources/guide/index.html"),
    ];
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("site/index.html"));
    }
    candidates.into_iter().find(|path| path.is_file()).ok_or_else(|| anyhow::anyhow!(
        "The HTML user guide is not installed. In a source checkout, build it using scripts/build-docs.sh."))
}
