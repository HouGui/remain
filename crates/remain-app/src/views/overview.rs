use super::{
    charts, controls,
    data::{self, Chart},
};
use crate::app::App;
use eframe::egui::{self, RichText};
use remain_core::locale;

pub(super) fn show(app: &mut App, ui: &mut egui::Ui) {
    let content_width = ui.available_width().min(ui.clip_rect().width());
    ui.set_max_width(content_width);
    let workspace = app.workspace.as_ref().expect("workspace view");
    let config = workspace.config();
    let t = |key| locale::text(&app.language, &config.translations, key).to_owned();
    ui.heading(config.title.as_str());
    ui.label(format!("{}: {}", t("period"), config.period));
    ui.add_space(12.0);
    ui.horizontal_wrapped(|ui| {
        for (count, key) in [
            (workspace.projects().len(), "projects"),
            (config.groups.len(), "groups"),
            (config.categories.len(), "categories"),
            (config.metrics.len(), "metrics"),
        ] {
            ui.label(
                RichText::new(format!("{}: {count}", t(key)))
                    .strong()
                    .size(18.0),
            );
        }
    });
    ui.separator();
    super::submissions::show(ui, workspace, &app.submissions, &app.language);
    let Some(analysis) = &app.analysis else {
        ui.label(t("no_results"));
        return;
    };
    let indicators = config.indicators();
    if !indicators.is_empty() {
        ui.small(format!(
            "{}: {} / {}",
            t("kpi_submissions"),
            analysis.projects.iter().filter(|p| p.has_kpis).count(),
            workspace.projects().len()
        ));
        ui.add_space(8.0);
    }
    super::totals::show(
        ui,
        &indicators,
        &analysis.overall,
        &app.language,
        content_width,
    );
    ui.add_space(16.0);
    ui.horizontal_wrapped(|ui| {
        for (tab, key) in [(0, "groups"), (1, "categories"), (2, "projects")] {
            ui.selectable_value(&mut app.tab, tab, t(key));
        }
    });
    controls::show(
        ui,
        &mut app.view,
        &mut app.chart_metric,
        config,
        &indicators,
        &app.language,
        content_width,
    );
    let mut rows = data::rows(workspace, analysis, app.tab, &app.language);
    data::select(&mut rows, &app.view, app.chart_metric);
    ui.small(format!(
        "{} {} / {}",
        t("showing"),
        rows.len().min(app.view.limit),
        rows.len()
    ));
    rows.truncate(app.view.limit);
    egui::ScrollArea::horizontal()
        .id_salt("result-table")
        .max_width(content_width)
        .show(ui, |ui| {
            egui::Grid::new("results")
                .striped(true)
                .min_col_width(110.0)
                .show(ui, |ui| {
                    ui.strong(t(match app.tab {
                        0 => "group",
                        1 => "category",
                        _ => "project",
                    }));
                    ui.strong(t(if app.tab == 2 {
                        "pages"
                    } else {
                        "included_projects"
                    }));
                    for indicator in &indicators {
                        ui.strong(indicator.label());
                    }
                    ui.end_row();
                    for row in &rows {
                        ui.label(row.label).on_hover_text(row.id);
                        ui.label(&row.count);
                        for (indicator, value) in indicators.iter().zip(row.values) {
                            ui.label(locale::number(
                                *value,
                                indicator.metric.decimals,
                                &app.language,
                            ));
                        }
                        ui.end_row();
                    }
                });
        });
    if indicators.is_empty() {
        return;
    }
    ui.small(t("missing"));
    ui.add_space(16.0);
    ui.horizontal(|ui| {
        ui.selectable_value(&mut app.view.chart, Chart::Bars, t("bars"));
        ui.selectable_value(&mut app.view.chart, Chart::Dots, t("dots"));
    });
    let indicator = &indicators[app.chart_metric];
    let metric = indicator.metric;
    ui.strong(format!("{} {}", indicator.label(), metric.unit));
    charts::show(
        ui,
        &rows,
        metric,
        app.chart_metric,
        app.view.chart,
        &app.language,
    );
    ui.small(t("chart_hint"));
}
