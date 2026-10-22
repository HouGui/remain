use super::*;

#[test]
fn failed_load_discards_previously_valid_results() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir()?;
    let path = remain_core::demo::create(tmp.path().join("demo"), "en")?;
    let mut app = App::default();
    app.appearance.dark_mode = true;
    app.appearance.font = Some(1);
    app.load(path);
    assert!(app.appearance.dark_mode);
    assert_eq!(app.appearance.font, Some(1));
    app.analysis = Some(app.workspace.as_ref().expect("loaded").analyze()?);
    app.load(tmp.path().join("missing.json"));
    assert!(app.workspace.is_none() && app.analysis.is_none() && app.loaded_path.is_none());
    assert!(app.error.is_some());
    assert!(app.appearance.dark_mode);
    assert_eq!(app.appearance.font, Some(1));
    Ok(())
}

#[test]
fn completed_demo_clears_stale_editor_and_results() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir()?;
    let path = remain_core::demo::create(tmp.path().join("demo"), "fr")?;
    let workspace = Workspace::load(&path)?;
    let mut app = App {
        editor: Some(Editor::empty(tmp.path().into(), "en")),
        analysis: Some(workspace.analyze()?),
        output: Some(tmp.path().join("old-output")),
        error: Some("old error".into()),
        chart_metric: 2,
        ..App::default()
    };
    app.appearance.dark_mode = true;
    app.appearance.font = Some(2);
    let (sender, receiver) = std::sync::mpsc::channel();
    sender.send(Event::Finished(Box::new(Ok(Outcome::Demo(
        path.clone(),
        workspace,
    )))))?;
    app.job = Some(receiver);
    app.poll();
    assert!(!app.busy());
    assert!(app.editor.is_none() && app.analysis.is_none() && app.output.is_none());
    assert!(app.error.is_none());
    assert_eq!(app.loaded_path, Some(path));
    assert_eq!(app.chart_metric, 0);
    assert_eq!(app.language, "fr");
    assert!(app.appearance.dark_mode);
    assert_eq!(app.appearance.font, Some(2));
    Ok(())
}

#[test]
fn background_analysis_completes_and_replaces_stale_state() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir()?;
    let path = remain_core::demo::create(tmp.path().join("demo"), "de")?;
    let mut app = App::default();
    app.load(path);
    app.analyze(&egui::Context::default());
    assert!(app.busy() && app.analysis.is_none());
    let result = finish(app.job.take().expect("worker"))?;
    match result {
        Outcome::Analyzed(_, analysis) => assert_eq!(analysis.overall[0], Some(12.)),
        _ => panic!("unexpected worker outcome"),
    }
    assert_eq!(app.language, "de");
    Ok(())
}

#[test]
fn welcome_and_analyzed_views_render_in_all_languages() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir()?;
    let mut app = App::default();
    let context = egui::Context::default();
    let output = context.run(egui::RawInput::default(), |ctx| views::show(&mut app, ctx));
    assert!(!output.shapes.is_empty());
    let path = remain_core::demo::create(tmp.path().join("demo"), "en")?;
    app.load(path);
    app.analysis = Some(app.workspace.as_ref().expect("loaded").analyze()?);
    for language in locale::LANGUAGES.iter() {
        app.language = language.code.into();
        for tab in [0, 1, 2] {
            app.tab = tab;
            for chart in [views::data::Chart::Bars, views::data::Chart::Dots] {
                app.view.chart = chart;
                let output =
                    context.run(egui::RawInput::default(), |ctx| views::show(&mut app, ctx));
                assert!(!output.shapes.is_empty());
            }
        }
        app.about = true;
        let output = context.run(egui::RawInput::default(), |ctx| {
            crate::help::show(&mut app, ctx)
        });
        assert!(!output.shapes.is_empty());
    }
    Ok(())
}

#[test]
fn french_workspace_loads_with_french_controls() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir()?;
    let path = remain_core::demo::create(tmp.path().join("demo"), "fr")?;
    let mut app = App::default();
    app.load(path);
    assert_eq!(app.language, "fr");
    assert_eq!(app.t("save"), "Enregistrer la configuration");
    assert_eq!(app.t("about"), "À propos de REMAIN");
    assert_eq!(
        app.workspace
            .as_ref()
            .expect("loaded")
            .config()
            .annex
            .as_ref()
            .expect("fixture value")
            .sheet,
        "Indicateurs"
    );
    Ok(())
}

#[test]
fn every_class_can_be_selected_for_tables_and_charts() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir()?;
    let path = remain_core::demo::create_classified(tmp.path().join("classes"), "fr")?;
    let mut app = App::default();
    app.load(path);
    app.analysis = Some(app.workspace.as_ref().expect("loaded").analyze()?);
    let count = app.analysis.as_ref().expect("analysis").metric_ids.len();
    let context = egui::Context::default();
    for metric in 0..count {
        app.chart_metric = metric;
        for tab in [0, 1, 2] {
            app.tab = tab;
            for chart in [views::data::Chart::Bars, views::data::Chart::Dots] {
                app.view.chart = chart;
                let output = context.run(Default::default(), |ctx| views::show(&mut app, ctx));
                assert!(!output.shapes.is_empty());
            }
        }
    }
    Ok(())
}

#[test]
fn disconnected_worker_is_reported_without_staying_busy() {
    let mut app = App::default();
    let (sender, receiver) = std::sync::mpsc::channel();
    drop(sender);
    app.job = Some(receiver);
    app.poll();
    assert!(!app.busy());
    assert!(app.error.is_some());
}

#[test]
fn folder_open_rejects_missing_or_invalid_configuration_without_stale_results() -> anyhow::Result<()>
{
    let tmp = tempfile::tempdir()?;
    let path = remain_core::demo::create(tmp.path().join("folder with spaces"), "en")?;
    let mut app = App::default();
    app.open_folder(path.parent().expect("workspace folder").into());
    assert_eq!(app.submissions.len(), 3);
    assert!(app.error.is_none());
    app.open_folder(path.clone()); // Picking a file is not a folder selection.
    assert!(app.error.is_some() && app.workspace.is_none() && app.submissions.is_empty());
    app.open_folder(tmp.path().into());
    assert!(
        app.error
            .as_ref()
            .expect("load error")
            .contains("workspace.json")
    );
    std::fs::write(&path, "{invalid")?;
    app.open_folder(path.parent().expect("workspace folder").into());
    assert!(
        app.error
            .as_ref()
            .expect("fixture value")
            .contains("Invalid workspace JSON")
    );
    Ok(())
}

#[test]
fn refreshed_analysis_reads_external_roster_edits_and_keeps_interface_language()
-> anyhow::Result<()> {
    let tmp = tempfile::tempdir()?;
    let path = remain_core::demo::create(tmp.path().join("demo"), "en")?;
    let mut app = App::default();
    app.load(path.clone());
    app.language = "de".into();
    app.chart_metric = 5;
    let mut projects = app
        .workspace
        .as_ref()
        .expect("loaded workspace")
        .projects()
        .to_vec();
    projects[2].id = "not-submitted".into();
    std::fs::write(
        path.parent()
            .expect("workspace folder")
            .join("projects.csv"),
        remain_core::projects::encode("projects.csv", &projects)?,
    )?;
    app.analyze(&egui::Context::default());
    let result = finish(app.job.take().expect("worker"))?;
    let (sender, receiver) = std::sync::mpsc::channel();
    sender.send(Event::Finished(Box::new(Ok(result))))?;
    app.job = Some(receiver);
    app.poll();
    assert_eq!(app.language, "de");
    assert_eq!(app.chart_metric, 2);
    assert_eq!(
        app.workspace.as_ref().expect("loaded workspace").projects()[2].id,
        "not-submitted"
    );
    assert_eq!(app.analysis.as_ref().expect("analysis").projects.len(), 2);
    assert!(!app.submissions[2].is_complete());
    Ok(())
}

#[test]
fn pdf_only_empty_and_incomplete_views_render_in_each_gui_language() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir()?;
    let path = remain_core::demo::create(tmp.path().join("demo"), "en")?;
    let workspace = Workspace::load(&path)?;
    let mut config = workspace.config().clone();
    config.metrics.clear();
    config.annex = None;
    config.inputs.documents.retain(|d| !d.is_kpi());
    std::fs::remove_file(workspace.root().join("atlas/part-1.pdf"))?;
    let context = egui::Context::default();
    for roster in [workspace.projects().to_vec(), vec![]] {
        let workspace = Workspace::with_projects(workspace.root(), config.clone(), roster)?;
        for language in ["en", "de", "fr"] {
            let mut app = App::default();
            app.accept_analysis(workspace.clone(), workspace.analyze()?);
            app.language = language.into();
            for tab in [0, 1, 2] {
                app.tab = tab;
                let output = context.run(Default::default(), |ctx| views::show(&mut app, ctx));
                assert!(!output.shapes.is_empty());
            }
        }
    }
    Ok(())
}

fn finish(job: Job) -> anyhow::Result<Outcome> {
    loop {
        if let Event::Finished(result) = job.recv_timeout(std::time::Duration::from_secs(15))? {
            return (*result).map_err(anyhow::Error::msg);
        }
    }
}

#[test]
fn status_bar_tracks_operations_and_clears_progress_on_failure() -> anyhow::Result<()> {
    let mut app = App::default();
    let (sender, receiver) = std::sync::mpsc::channel();
    app.job = Some(receiver);
    sender.send(Event::Progress(Progress::ReadingProject {
        current: 2,
        total: 5,
    }))?;
    app.poll();
    assert!(app.busy());
    assert_eq!(app.status(), "Reading project 2 / 5");
    app.language = "fr".into();
    assert_eq!(app.status(), "Lecture du projet 2 / 5");
    sender.send(Event::Progress(Progress::AssemblingPdfs))?;
    app.poll();
    assert!(app.status().contains("PDF"));
    let context = egui::Context::default();
    let output = context.run(Default::default(), |ctx| views::show(&mut app, ctx));
    assert!(!output.shapes.is_empty());
    sender.send(Event::Finished(Box::new(Err("Invalid PDF".into()))))?;
    app.poll();
    assert!(!app.busy() && app.progress.is_none());
    assert_eq!(app.error.as_deref(), Some("Invalid PDF"));
    assert_eq!(app.status(), app.t("operation_failed"));
    Ok(())
}
