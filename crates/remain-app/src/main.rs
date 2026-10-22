//! Native entry point for REMAIN.
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app;
mod appearance;
mod editor;
mod help;
mod jobs;
mod views;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_app_id("remain")
            .with_icon(appearance::icon())
            .with_inner_size([1150.0, 780.0])
            .with_min_inner_size([760.0, 540.0]),
        ..Default::default()
    };
    eframe::run_native(
        "REMAIN",
        options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
}
