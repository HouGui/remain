//! Visual smoke-test harness using the real desktop views.
//!
//! Build with the preview feature. Set EFRAME_SCREENSHOT_TO to a PNG path, then
//! run with a workspace path and optional editor, metrics, small, categories, dots,
//! projects, negative, help, about, appearance, error, or dark arguments.
//! Pass a bundled font ID (noto-sans, lato, atkinson-hyperlegible) to select it.
//! eframe captures the second rendered frame and exits.
#[path = "../src/app.rs"]
mod app;
#[path = "../src/appearance.rs"]
mod appearance;
#[path = "../src/editor/mod.rs"]
mod editor;
#[path = "../src/help.rs"]
mod help;
#[path = "../src/jobs.rs"]
mod jobs;
#[path = "../src/views/mod.rs"]
mod views;

fn main() -> eframe::Result {
    let args: Vec<_> = std::env::args().collect();
    let path = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| "examples/demo-en/workspace.json".into());
    let small = args.iter().any(|a| a == "small");
    let edit = args
        .iter()
        .any(|a| a == "editor" || a == "metrics" || a == "documents");
    let storage = tempfile::tempdir().expect("temporary preview storage");
    let options = eframe::NativeOptions {
        persistence_path: Some(storage.path().join("preview.ron")),
        persist_window: false,
        viewport: eframe::egui::ViewportBuilder::default().with_inner_size(if small {
            [780.0, 560.0]
        } else {
            [1150.0, 900.0]
        }),
        ..Default::default()
    };
    eframe::run_native(
        "REMAIN preview",
        options,
        Box::new(move |cc| {
            let mut app = app::App::new(cc);
            app.appearance = appearance::Appearance::default();
            app.appearance.dark_mode = args.iter().any(|a| a == "dark");
            app.appearance.apply_theme(&cc.egui_ctx);
            let font = appearance::FONTS
                .iter()
                .position(|font| args.iter().any(|a| a == font.id));
            app.appearance.select_font(font, &cc.egui_ctx);
            cc.egui_ctx
                .all_styles_mut(|style| style.animation_time = 0.0);
            app.load(path.clone().into());
            if let Some(workspace) = &app.workspace {
                app.analysis = Some(
                    workspace
                        .analyze()
                        .expect("preview workspace must validate"),
                );
                if args.iter().any(|a| a == "roster") {
                    app.project_editor = Some(editor::ProjectEditor::new(
                        workspace,
                        path.clone().into(),
                        &app.language,
                    ));
                }
                if edit {
                    app.editor = Some(editor::Editor::new(workspace, path.into(), &app.language));
                    if args.iter().any(|a| a == "documents") {
                        app.editor.as_mut().expect("editor").section = editor::Section::Documents;
                    }
                    if args.iter().any(|a| a == "metrics") {
                        app.editor.as_mut().expect("editor").section = editor::Section::Metrics;
                    }
                }
            }
            if args.iter().any(|a| a == "categories") {
                app.tab = 1;
            }
            if args.iter().any(|a| a == "projects") {
                app.tab = 2;
            }
            if args.iter().any(|a| a == "dots") {
                app.view.chart = views::data::Chart::Dots;
            }
            if args.iter().any(|a| a == "about") {
                app.about = true;
            }
            if args.iter().any(|a| a == "help") {
                app.help = true;
            }
            if args.iter().any(|a| a == "error") {
                app.error = Some("Preview: an input file could not be read.".into());
            }
            if args.iter().any(|a| a == "negative")
                && let Some(analysis) = &mut app.analysis
            {
                analysis.groups[0].values[0] = Some(-5.0);
            }
            Ok(Box::new(app))
        }),
    )
}
