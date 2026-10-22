use super::fields;
use eframe::egui;
use remain_core::config::Classification;

pub(super) fn show(
    ui: &mut egui::Ui,
    items: &mut Vec<Classification>,
    singular: &str,
    plural: &str,
    t: &impl Fn(&str) -> String,
) {
    ui.heading(t(plural));
    ui.small(t("optional_classification"));
    let mut remove = None;
    let mut reorder = None;
    let length = items.len();
    for (i, item) in items.iter_mut().enumerate() {
        ui.push_id((singular, i), |ui| {
            fields::text(ui, &t("id"), &mut item.id);
            fields::text(ui, &t("label"), &mut item.label);
            fields::actions(ui, i, length, t, &mut remove, &mut reorder);
        });
    }
    fields::apply(items, remove, reorder);
    if ui.button(format!("{} {}", t("add"), t(singular))).clicked() {
        items.push(Classification {
            id: fields::next_id(singular, items.iter().map(|item| item.id.as_str())),
            label: t(singular),
        });
    }
}
