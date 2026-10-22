use super::data::{Chart, Row};
use crate::appearance;
use eframe::egui;
use remain_core::{config::Metric, locale};

pub(super) fn show(
    ui: &mut egui::Ui,
    rows: &[Row<'_>],
    metric: &Metric,
    index: usize,
    chart: Chart,
    language: &str,
) {
    let values = rows.iter().filter_map(|r| r.values[index]);
    let scale = values.clone().map(f64::abs).fold(1.0_f64, f64::max);
    // Normalize before subtracting to avoid overflow for large, finite inputs.
    let low = values.clone().map(|v| v / scale).fold(0.0_f64, f64::min);
    let high = values.map(|v| v / scale).fold(0.0_f64, f64::max);
    let span = if high == low { 1.0 } else { high - low };
    let position = |v: f64| ((v / scale - low) / span) as f32;
    let width = ui.available_width().min(ui.clip_rect().width());
    for row in rows {
        ui.horizontal(|ui| {
            ui.add_sized(
                [(width * 0.3).clamp(100.0, 230.0), 28.0],
                egui::Label::new(row.label).wrap(),
            )
            .on_hover_text(row.id);
            let plot_width = (ui.available_width() - 100.0).max(60.0);
            let (rect, response) =
                ui.allocate_exact_size(egui::vec2(plot_width, 28.0), egui::Sense::hover());
            let rect = rect.shrink2(egui::vec2(4.0, 0.0));
            let zero = rect.left() + position(0.0) * rect.width();
            ui.painter().line_segment(
                [
                    egui::pos2(zero, rect.top()),
                    egui::pos2(zero, rect.bottom()),
                ],
                ui.visuals().widgets.noninteractive.fg_stroke,
            );
            if let Some(value) = row.values[index] {
                let x = rect.left() + position(value) * rect.width();
                let color = if value < 0.0 {
                    appearance::negative(ui.visuals())
                } else {
                    appearance::accent(ui.visuals())
                };
                match chart {
                    Chart::Bars => {
                        let bar = egui::Rect::from_min_max(
                            egui::pos2(x.min(zero), rect.top() + 6.0),
                            egui::pos2(x.max(zero), rect.bottom() - 6.0),
                        );
                        ui.painter().rect_filled(bar, 2.0, color);
                    }
                    Chart::Dots => {
                        ui.painter()
                            .circle_filled(egui::pos2(x, rect.center().y), 4.0, color);
                    }
                }
            }
            let number = locale::number(row.values[index], metric.decimals, language);
            response.on_hover_text(format!("{}: {number} {}", row.label, metric.unit));
            ui.label(number);
        });
    }
}
