use eframe::egui;
use remain_core::{Workspace, locale, submissions::Submission};

pub(super) fn show(
    ui: &mut egui::Ui,
    workspace: &Workspace,
    submissions: &[Submission],
    language: &str,
) {
    let t = |key| locale::text(language, &workspace.config().translations, key);
    if workspace.projects().is_empty() {
        ui.label(t("empty_projects"));
        return;
    }
    if submissions.is_empty() {
        return;
    }
    let missing = submissions.iter().filter(|s| !s.is_complete()).count();
    if missing > 0 {
        ui.colored_label(
            ui.visuals().warn_fg_color,
            format!(
                "{}: {missing} / {}",
                t("missing_submissions"),
                submissions.len()
            ),
        );
        ui.label(t("partial_explanation"));
    }
    let names: std::collections::HashMap<_, _> = workspace
        .projects()
        .iter()
        .map(|p| (p.id.as_str(), p.label.as_str()))
        .collect();
    egui::CollapsingHeader::new(format!(
        "{}: {} / {}",
        t("submissions"),
        submissions.len() - missing,
        submissions.len()
    ))
    .default_open(missing > 0)
    .show(ui, |ui| {
        egui::ScrollArea::vertical()
            .id_salt("submission-checklist")
            .max_height(220.0)
            .show(ui, |ui| {
                // Outstanding required submissions come first, preserving roster order.
                for complete in [false, true] {
                    for submission in submissions.iter().filter(|s| s.is_complete() == complete) {
                        ui.strong(format!(
                            "{} ({})",
                            names[submission.project.as_str()],
                            submission.project
                        ));
                        if complete {
                            ui.small(t("submitted"));
                        }
                        for missing in &submission.missing {
                            ui.colored_label(ui.visuals().warn_fg_color, format!("• {missing}"));
                        }
                        if !submission.optional_missing.is_empty() {
                            ui.small(format!(
                                "{}: {}",
                                t("optional_missing"),
                                submission.optional_missing.join(", ")
                            ));
                        }
                        ui.add_space(5.0);
                    }
                }
            });
    });
    ui.add_space(8.0);
}
