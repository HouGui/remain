use super::{breakdown, fields};
use eframe::egui;
use remain_core::config::{Aggregation, Config, Metric, Missing};

pub(super) fn show(ui: &mut egui::Ui, config: &mut Config, t: &impl Fn(&str) -> String) {
    ui.heading(t("metrics"));
    let mut remove = None;
    let mut reorder = None;
    let length = config.metrics.len();
    for (i, metric) in config.metrics.iter_mut().enumerate() {
        egui::CollapsingHeader::new(format!("{} · {}", metric.id, metric.label))
            .id_salt(("metric", i))
            .default_open(i == 0)
            .show(ui, |ui| {
                fields::text(ui, &t("id"), &mut metric.id);
                fields::text(ui, &t("label"), &mut metric.label);
                fields::text(ui, &t("column"), &mut metric.column);
                fields::choice(
                    ui,
                    t,
                    "aggregation",
                    &mut metric.aggregation,
                    &[
                        (Aggregation::Sum, "sum"),
                        (Aggregation::Mean, "mean"),
                        (Aggregation::Min, "min"),
                        (Aggregation::Max, "max"),
                        (Aggregation::Count, "count"),
                    ],
                );
                fields::choice(
                    ui,
                    t,
                    "missing_policy",
                    &mut metric.missing,
                    &[
                        (Missing::Error, "error_policy"),
                        (Missing::Skip, "skip"),
                        (Missing::Zero, "zero"),
                    ],
                );
                breakdown::show(ui, metric, t);
                ui.collapsing(t("display_options"), |ui| {
                    fields::text(ui, &t("unit"), &mut metric.unit);
                    fields::row(ui, &t("decimals"), |ui| {
                        ui.add(egui::DragValue::new(&mut metric.decimals).range(0..=6));
                    });
                });
                fields::actions(ui, i, length, t, &mut remove, &mut reorder);
            });
    }
    fields::apply(&mut config.metrics, remove, reorder);
    if ui.button(t("add")).clicked() {
        config.metrics.push(Metric {
            id: fields::next_id("metric", config.metrics.iter().map(|m| m.id.as_str())),
            label: t("metrics"),
            column: String::new(),
            aggregation: Aggregation::Sum,
            missing: Missing::Error,
            unit: String::new(),
            decimals: 0,
            breakdown: None,
        });
    }
}
