use crate::{app::App, appearance};
use eframe::egui::{self, RichText};
use remain_core::locale;

pub(super) fn show(app: &mut App, context: &egui::Context) {
    egui::TopBottomPanel::top("header")
        .exact_height(48.0)
        .show(context, |ui| {
            ui.spacing_mut().interact_size.y = 36.0;
            let show_tagline = ui.available_width() >= 900.0;
            ui.horizontal_centered(|ui| {
                ui.label(
                    RichText::new("REMAIN")
                        .strong()
                        .size(26.0)
                        .color(appearance::accent(ui.visuals())),
                );
                if show_tagline {
                    ui.label(app.t("tagline"));
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.menu_button(app.t("help_menu"), |ui| {
                        if ui.button(app.t("help")).clicked() {
                            app.help = true;
                            ui.close();
                        }
                        if ui.button(app.t("about")).clicked() {
                            app.about = true;
                            ui.close();
                        }
                    });
                    let button = ui.button(app.t("appearance"));
                    #[cfg(feature = "preview")]
                    if std::env::args().any(|arg| arg == "appearance") {
                        egui::Popup::open_id(context, egui::Popup::default_response_id(&button));
                    }
                    egui::Popup::menu(&button)
                        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                        .show(|ui| appearance_menu(app, ui));
                    ui.allocate_ui_with_layout(
                        egui::vec2(100.0, 36.0),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            egui::ComboBox::from_id_salt("language")
                                .selected_text(
                                    locale::LANGUAGES
                                        .iter()
                                        .find(|language| language.code == app.language)
                                        .map_or(app.language.as_str(), |language| language.name),
                                )
                                .show_ui(ui, |ui| {
                                    for language in locale::LANGUAGES.iter() {
                                        ui.selectable_value(
                                            &mut app.language,
                                            language.code.into(),
                                            language.name,
                                        );
                                    }
                                    if let Some(workspace) = &app.workspace {
                                        let language = &workspace.config().locale;
                                        if !locale::LANGUAGES
                                            .iter()
                                            .any(|builtin| builtin.code == language)
                                        {
                                            ui.selectable_value(
                                                &mut app.language,
                                                language.clone(),
                                                language,
                                            );
                                        }
                                    }
                                });
                        },
                    );
                });
            });
        });
}

fn appearance_menu(app: &mut App, ui: &mut egui::Ui) {
    let dark_mode = app.t("dark_mode");
    if ui
        .checkbox(&mut app.appearance.dark_mode, dark_mode)
        .changed()
    {
        app.appearance.apply_theme(ui.ctx());
    }
    ui.separator();
    ui.label(app.t("font"));
    let default = app.t("font_default");
    for (font, name) in std::iter::once((None, default.as_str())).chain(
        appearance::FONTS
            .iter()
            .enumerate()
            .map(|(index, font)| (Some(index), font.name)),
    ) {
        let selected = !app.appearance.has_custom_font() && app.appearance.font == font;
        if ui.radio(selected, name).clicked() {
            app.appearance.select_font(font, ui.ctx());
        }
    }
    if app.appearance.has_custom_font() {
        ui.label(app.appearance.font_name(&default));
    }
    ui.separator();
    if ui.button(app.t("font_custom")).clicked() {
        ui.close();
        if let Some(path) = rfd::FileDialog::new()
            .add_filter(app.t("font"), &["ttf", "otf"])
            .pick_file()
            && let Err(error) = app.appearance.load_custom_font(&path, ui.ctx())
        {
            app.error = Some(error.to_string());
        }
    }
}

#[cfg(test)]
#[path = "header_tests.rs"]
mod tests;
