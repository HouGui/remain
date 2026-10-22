use crate::{app::App, appearance, jobs};
use eframe::egui;

pub(crate) fn show(app: &mut App, context: &egui::Context) {
    let mut help = app.help;
    let mut about = app.about;
    if help {
        egui::Window::new(app.t("help"))
            .open(&mut help)
            .default_width(520.0)
            .show(context, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(380.0)
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = 10.0;
                        for (heading, items) in [
                            (
                                "help_workflow",
                                ["help_open", "help_analyze", "help_export"],
                            ),
                            ("help_bundle", ["help_pdfs", "help_excel", "help_html"]),
                        ] {
                            ui.strong(app.t(heading));
                            for key in items {
                                ui.horizontal_top(|ui| {
                                    ui.label("•");
                                    ui.add(egui::Label::new(app.t(key)).wrap());
                                });
                            }
                            ui.add_space(4.0);
                        }
                    });
                ui.separator();
                guide_button(app, ui);
            });
    }
    if about {
        egui::Window::new(app.t("about"))
            .open(&mut about)
            .resizable(false)
            .show(context, |ui| {
                ui.horizontal(|ui| {
                    let logo = app.logo.get_or_insert_with(|| {
                        let icon = appearance::icon();
                        context.load_texture(
                            "remain-logo",
                            egui::ColorImage::from_rgba_unmultiplied(
                                [icon.width as usize, icon.height as usize],
                                &icon.rgba,
                            ),
                            egui::TextureOptions::LINEAR,
                        )
                    });
                    ui.image((logo.id(), egui::vec2(48.0, 48.0)));
                    ui.heading("REMAIN");
                });
                ui.label(app.t("about_text"));
                ui.label(format!(
                    "{} {}",
                    app.t("version"),
                    env!("CARGO_PKG_VERSION")
                ));
                ui.label(format!(
                    "{}: {} · {}",
                    app.t("license"),
                    env!("CARGO_PKG_LICENSE"),
                    env!("CARGO_PKG_AUTHORS")
                ));
                if let Some(url) =
                    option_env!("REMAIN_DOCS_URL").filter(|url| url.starts_with("https://"))
                {
                    ui.hyperlink_to(app.t("docs_website"), url);
                } else {
                    ui.label(app.t("docs_pending"));
                }
                guide_button(app, ui);
                ui.collapsing(app.t("font_licenses"), |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(240.0)
                        .show(ui, |ui| {
                            ui.set_max_width(500.0);
                            for font in appearance::FONTS {
                                ui.collapsing(font.name, |ui| {
                                    ui.label(font.license);
                                });
                            }
                        });
                });
            });
    }
    app.help = help;
    app.about = about;
}

fn guide_button(app: &mut App, ui: &mut egui::Ui) {
    if ui.button(app.t("guide_open")).clicked()
        && let Err(error) = jobs::guide().and_then(|path| jobs::open_directory(&path))
    {
        app.error = Some(error.to_string());
    }
}
