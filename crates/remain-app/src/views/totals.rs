use eframe::egui::{self, RichText};
use remain_core::{config::Indicator, locale};

pub(super) fn show(
    ui: &mut egui::Ui,
    indicators: &[Indicator<'_>],
    values: &[Option<f64>],
    language: &str,
    width: f32,
) {
    let totals: Vec<_> = indicators
        .iter()
        .enumerate()
        .filter(|(_, indicator)| indicator.class.is_none())
        .collect();
    let columns = ((width + 12.0) / 195.0).floor().max(1.0) as usize;
    for row in totals.chunks(columns) {
        ui.horizontal_top(|ui| {
            for &(index, indicator) in row {
                let metric = indicator.metric;
                egui::Frame::group(ui.style())
                    .inner_margin(18.0)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.set_width(145.0);
                            ui.strong(metric.label.as_str());
                            ui.label(
                                RichText::new(locale::number(
                                    values[index],
                                    metric.decimals,
                                    language,
                                ))
                                .size(30.0)
                                .strong(),
                            );
                            if !metric.unit.is_empty() {
                                ui.small(&metric.unit);
                            }
                            if let Some(breakdown) = &metric.breakdown {
                                ui.separator();
                                for (offset, class) in breakdown.classes.iter().enumerate() {
                                    ui.small(format!(
                                        "{}: {}",
                                        class.label.as_str(),
                                        locale::number(
                                            values[index + offset + 1],
                                            metric.decimals,
                                            language
                                        )
                                    ));
                                }
                            }
                        });
                    });
            }
        });
    }
}
