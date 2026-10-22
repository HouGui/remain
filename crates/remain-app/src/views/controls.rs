use super::data::{Options, Sort};
use eframe::egui;
use remain_core::{
    config::{Config, Indicator},
    locale,
};

pub(super) fn show(
    ui: &mut egui::Ui,
    options: &mut Options,
    metric: &mut usize,
    config: &Config,
    indicators: &[Indicator<'_>],
    language: &str,
    width: f32,
) {
    ui.scope(|ui| {
        ui.spacing_mut().interact_size.y = 36.0;
        let t = |key| locale::text(language, &config.translations, key);
        let filter = |ui: &mut egui::Ui, options: &mut Options| {
            ui.label(t("filter"));
            ui.add(egui::TextEdit::singleline(&mut options.filter).desired_width(180.0));
        };
        let metric_picker = |ui: &mut egui::Ui, metric: &mut usize| {
            if indicators.is_empty() {
                return;
            }
            ui.label(t("metrics"));
            egui::ComboBox::from_id_salt("metric")
                .width(150.0)
                .selected_text(indicators[*metric].label())
                .show_ui(ui, |ui| {
                    for (i, definition) in indicators.iter().enumerate() {
                        ui.selectable_value(metric, i, definition.label());
                    }
                });
        };
        if width >= 700.0 {
            ui.horizontal(|ui| {
                filter(ui, options);
                metric_picker(ui, metric);
            });
        } else {
            ui.horizontal(|ui| filter(ui, options));
            ui.horizontal(|ui| metric_picker(ui, metric));
        }
        ui.horizontal(|ui| {
            ui.label(t("sort"));
            egui::ComboBox::from_id_salt("sort")
                .width(180.0)
                .selected_text(t(match options.sort {
                    Sort::Configured => "configured_order",
                    Sort::Name => "by_name",
                    Sort::Value => "by_value",
                }))
                .show_ui(ui, |ui| {
                    for (value, key) in [
                        (Sort::Configured, "configured_order"),
                        (Sort::Name, "by_name"),
                        (Sort::Value, "by_value"),
                    ] {
                        if value != Sort::Value || !indicators.is_empty() {
                            ui.selectable_value(&mut options.sort, value, t(key));
                        }
                    }
                });
            ui.label(t("limit"));
            egui::ComboBox::from_id_salt("limit")
                .width(70.0)
                .selected_text(options.limit.to_string())
                .show_ui(ui, |ui| {
                    for limit in [10, 25, 50, 100] {
                        ui.selectable_value(&mut options.limit, limit, limit.to_string());
                    }
                });
        });
    });
}
