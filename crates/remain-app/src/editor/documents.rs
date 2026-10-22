use super::fields;
use eframe::egui;
use remain_core::config::{Annex, Config, DocumentRule};

pub(super) fn show(ui: &mut egui::Ui, config: &mut Config, t: &impl Fn(&str) -> String) {
    ui.add(egui::Label::new(t("documents_hint")).wrap());
    fields::text(ui, &t("directory_pattern"), &mut config.inputs.directory);
    let length = config.inputs.documents.len();
    let mut remove = None;
    let mut reorder = None;
    for (i, document) in config.inputs.documents.iter_mut().enumerate() {
        ui.push_id(i, |ui| {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                fields::text(ui, &t("label"), &mut document.label);
                fields::text(ui, &t("filename_pattern"), &mut document.pattern);
                ui.checkbox(&mut document.required, t("required"));
                fields::actions(ui, i, length, t, &mut remove, &mut reorder);
            });
        });
    }
    if remove.is_some_and(|i| config.inputs.documents[i].is_kpi()) {
        config.annex = None;
        config.metrics.clear();
    }
    fields::apply(&mut config.inputs.documents, remove, reorder);
    ui.horizontal_wrapped(|ui| {
        if ui.button(t("add_report")).clicked() {
            config.inputs.documents.push(DocumentRule {
                label: t("pdf"),
                pattern: String::new(),
                required: true,
            });
        }
        if ui
            .add_enabled(
                !config.inputs.documents.iter().any(DocumentRule::is_kpi),
                egui::Button::new(format!("{} {}", t("add"), t("excel"))),
            )
            .clicked()
        {
            config.inputs.documents.push(DocumentRule {
                label: t("excel"),
                pattern: "annex.xlsx".into(),
                required: false,
            });
        }
    });
    let has_kpis = config.inputs.documents.iter().any(DocumentRule::is_kpi);
    if has_kpis {
        let annex = config.annex.get_or_insert_with(|| Annex {
            sheet: "KPIs".into(),
            header_row: 1,
        });
        ui.separator();
        fields::text(ui, &t("sheet"), &mut annex.sheet);
        fields::row(ui, &t("header"), |ui| {
            ui.add(egui::DragValue::new(&mut annex.header_row).range(1..=1_048_576));
        });
    }
}
