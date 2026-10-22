use super::fields;
use eframe::egui;
use remain_core::config::{Breakdown, Metric, MetricClass};

pub(super) fn show(ui: &mut egui::Ui, metric: &mut Metric, t: &impl Fn(&str) -> String) {
    let mut enabled = metric.breakdown.is_some();
    if ui.checkbox(&mut enabled, t("breakdown")).changed() {
        metric.breakdown = enabled.then(|| Breakdown {
            column: String::new(),
            classes: Vec::new(),
        });
    }
    let Some(breakdown) = &mut metric.breakdown else {
        return;
    };
    ui.indent("breakdown", |ui| {
        ui.label(t("breakdown_hint"));
        fields::text(ui, &t("property_column"), &mut breakdown.column);
        let mut remove = None;
        let mut reorder = None;
        let length = breakdown.classes.len();
        for (index, class) in breakdown.classes.iter_mut().enumerate() {
            ui.push_id(index, |ui| {
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    fields::text(ui, &t("id"), &mut class.id);
                    fields::text(ui, &t("class_name"), &mut class.label);
                    fields::text(ui, &t("class_value"), &mut class.value);
                    fields::actions(ui, index, length, t, &mut remove, &mut reorder);
                });
            });
        }
        fields::apply(&mut breakdown.classes, remove, reorder);
        if ui.button(t("add_class")).clicked() {
            breakdown.classes.push(MetricClass {
                id: fields::next_id("class", breakdown.classes.iter().map(|c| c.id.as_str())),
                label: String::new(),
                value: String::new(),
            });
        }
    });
}
