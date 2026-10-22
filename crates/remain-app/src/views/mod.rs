mod charts;
mod controls;
pub(crate) mod data;
mod header;
mod overview;
mod submissions;
mod totals;
use crate::{
    app::App,
    editor::{Editor, ProjectEditor},
    jobs,
};
use eframe::egui;
use overview::show as overview;
use std::path::PathBuf;

pub(crate) fn show(app: &mut App, context: &egui::Context) {
    header::show(app, context);
    egui::TopBottomPanel::bottom("status").show(context, |ui| {
        ui.horizontal(|ui| {
            ui.set_min_height(20.0);
            if app.busy() {
                ui.add(egui::Spinner::new().size(14.0));
            }
            ui.add(egui::Label::new(egui::RichText::new(app.status()).small()).truncate())
                .on_hover_text(format!("{}\n{}", app.status(), app.t("refresh")));
        });
    });
    egui::SidePanel::left("actions")
        .resizable(false)
        .default_width(230.0)
        .show(context, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(16.0);
                ui.heading(app.t("workspace"));
                ui.add_space(10.0);
                ui.add_enabled_ui(
                    !app.busy() && app.editor.is_none() && app.project_editor.is_none(),
                    |ui| {
                        if ui.button(app.t("open")).clicked()
                            && let Some(path) = rfd::FileDialog::new()
                                .set_title(app.t("open"))
                                .pick_folder()
                        {
                            app.open_folder(path);
                        }
                        ui.menu_button(app.t("demo"), |ui| {
                            for (key, classes) in [("demo_basic", false), ("demo_classes", true)] {
                                if ui.button(app.t(key)).clicked() {
                                    ui.close();
                                    app.demo(context, classes);
                                }
                            }
                        });
                        if ui.button(app.t("new")).clicked()
                            && let Some(root) = rfd::FileDialog::new().pick_folder()
                        {
                            let path = root.join("workspace.json");
                            if path.exists() {
                                app.load(path);
                            } else {
                                app.editor = Some(Editor::empty(root, &app.language));
                            }
                        }
                        ui.separator();
                        if ui
                            .add_enabled(app.workspace.is_some(), egui::Button::new(app.t("edit")))
                            .clicked()
                            && let (Some(workspace), Some(path)) =
                                (&app.workspace, &app.loaded_path)
                        {
                            app.editor = Some(Editor::new(workspace, path.clone(), &app.language));
                        }
                        if ui
                            .add_enabled(
                                app.workspace.is_some(),
                                egui::Button::new(app.t("project_list")),
                            )
                            .clicked()
                            && let (Some(workspace), Some(path)) =
                                (&app.workspace, &app.loaded_path)
                        {
                            app.project_editor =
                                Some(ProjectEditor::new(workspace, path.clone(), &app.language));
                        }
                        if ui
                            .add_enabled(
                                app.workspace.is_some(),
                                egui::Button::new(app.t("validate")),
                            )
                            .clicked()
                        {
                            app.analyze(context);
                        }
                        if ui
                            .add_enabled(
                                app.analysis
                                    .as_ref()
                                    .is_some_and(|a| !a.projects.is_empty()),
                                egui::Button::new(app.t("export")),
                            )
                            .clicked()
                        {
                            app.export(context);
                        }
                        ui.separator();
                        ui.collapsing(app.t("path"), |ui| {
                            ui.add(egui::TextEdit::singleline(&mut app.path).desired_width(200.0));
                            if ui.button(app.t("load")).clicked() && !app.path.is_empty() {
                                app.open_folder(PathBuf::from(&app.path));
                            }
                        });
                    },
                );
                if let Some(path) = &app.output {
                    ui.separator();
                    ui.label(app.t("output"));
                    ui.label(path.display().to_string());
                    if ui.button(app.t("open_output")).clicked()
                        && let Err(error) = jobs::open_directory(path)
                    {
                        app.error = Some(error.to_string());
                    }
                }
            });
        });
    egui::CentralPanel::default().show(context, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(20.0);
            if let Some(error) = &app.error {
                egui::Frame::new()
                    .fill(ui.visuals().faint_bg_color)
                    .stroke(egui::Stroke::new(1.0_f32, ui.visuals().error_fg_color))
                    .inner_margin(16.0)
                    .show(ui, |ui| {
                        ui.colored_label(ui.visuals().error_fg_color, app.t("error"));
                        ui.label(error);
                    });
                ui.add_space(16.0);
            }
            if app.workspace.is_none() {
                ui.heading(app.t("welcome"));
                ui.add_space(16.0);
                ui.label(app.t("intro"));
                ui.add_space(24.0);
                ui.strong(app.t("workflow"));
                return;
            }
            overview(app, ui);
        });
    });
}
