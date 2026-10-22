use eframe::egui;

pub(super) fn next_id<'a>(prefix: &str, existing: impl Iterator<Item = &'a str>) -> String {
    let used: std::collections::HashSet<_> = existing.map(str::to_ascii_lowercase).collect();
    (1..)
        .map(|i| format!("{prefix}-{i}"))
        .find(|id| !used.contains(id))
        .expect("finite IDs")
}

pub(super) fn text(ui: &mut egui::Ui, caption: &str, value: &mut String) {
    row(ui, caption, |ui| {
        let input_width = ui.available_width().min(340.0);
        ui.add(egui::TextEdit::singleline(value).desired_width(input_width));
    });
}

pub(super) fn row<R>(
    ui: &mut egui::Ui,
    caption: &str,
    content: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    ui.horizontal(|ui| {
        let caption_width = (ui.available_width() * 0.35).min(170.0);
        ui.add_sized(
            [caption_width, 24.0],
            egui::Label::new(caption).wrap().halign(egui::Align::Min),
        );
        content(ui)
    })
    .inner
}

pub(super) fn choice<T: Copy + PartialEq>(
    ui: &mut egui::Ui,
    t: &impl Fn(&str) -> String,
    key: &str,
    value: &mut T,
    options: &[(T, &str)],
) {
    row(ui, &t(key), |ui| {
        let selected = options
            .iter()
            .find(|(option, _)| option == value)
            .expect("known option")
            .1;
        egui::ComboBox::from_id_salt(key)
            .selected_text(t(selected))
            .show_ui(ui, |ui| {
                for &(option, name) in options {
                    ui.selectable_value(value, option, t(name));
                }
            });
    });
}

pub(super) fn actions(
    ui: &mut egui::Ui,
    index: usize,
    length: usize,
    t: &impl Fn(&str) -> String,
    remove: &mut Option<usize>,
    reorder: &mut Option<(usize, usize)>,
) {
    ui.horizontal(|ui| {
        if ui.small_button(t("remove")).clicked() {
            *remove = Some(index);
        }
        if ui
            .add_enabled(index > 0, egui::Button::new(t("up")).small())
            .clicked()
        {
            *reorder = Some((index, index - 1));
        }
        if ui
            .add_enabled(index + 1 < length, egui::Button::new(t("down")).small())
            .clicked()
        {
            *reorder = Some((index, index + 1));
        }
    });
}

pub(super) fn apply<T>(items: &mut Vec<T>, remove: Option<usize>, reorder: Option<(usize, usize)>) {
    if let Some(index) = remove {
        items.remove(index);
    } else if let Some((a, b)) = reorder {
        items.swap(a, b);
    }
}
