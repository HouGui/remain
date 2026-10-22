use super::*;

fn render(
    app: &mut App,
    context: &egui::Context,
    width: f32,
    events: Vec<egui::Event>,
) -> egui::FullOutput {
    context.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width, 560.0),
            )),
            events,
            ..Default::default()
        },
        |ctx| show(app, ctx),
    )
}

fn text_rect(output: &egui::FullOutput, label: &str) -> egui::Rect {
    output
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::Shape::Text(text) if text.galley.job.text == label => {
                Some(egui::Rect::from_min_size(text.pos, text.galley.size()))
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("visible label: {label}"))
}

fn click_label(app: &mut App, context: &egui::Context, label: &str) {
    let output = render(app, context, 780.0, Vec::new());
    let pos = text_rect(&output, label).center();
    let _ = render(
        app,
        context,
        780.0,
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Default::default(),
            },
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            },
        ],
    );
    // Settle font/layout changes before the next interaction.
    let _ = render(app, context, 780.0, Vec::new());
}

#[test]
fn toolbar_labels_fit_and_align_in_both_themes_with_every_bundled_font() {
    for language in locale::LANGUAGES.iter() {
        for width in [760.0, 1150.0] {
            let mut app = App::default();
            app.language = language.code.into();
            let context = egui::Context::default();
            appearance::configure(&context);
            for font in std::iter::once(None).chain((0..appearance::FONTS.len()).map(Some)) {
                app.appearance.select_font(font, &context);
                for dark_mode in [false, true] {
                    app.appearance.dark_mode = dark_mode;
                    app.appearance.apply_theme(&context);
                    let _ = render(&mut app, &context, width, Vec::new());
                    let output = render(&mut app, &context, width, Vec::new());
                    let bounds: Vec<_> = [
                        app.t("help_menu"),
                        app.t("appearance"),
                        language.name.into(),
                    ]
                    .iter()
                    .map(|label| text_rect(&output, label))
                    .collect();
                    for rect in &bounds {
                        assert!((rect.center().y - bounds[0].center().y).abs() <= 1.0);
                        assert!(rect.min.x >= 0.0 && rect.max.x <= width);
                        assert!(rect.min.y >= 0.0 && rect.max.y <= 48.0);
                    }
                    assert!(bounds[2].max.x < bounds[1].min.x && bounds[1].max.x < bounds[0].min.x);
                }
            }
        }
    }
}

#[test]
fn appearance_menu_changes_theme_and_fonts_without_closing_or_losing_selection() {
    let mut app = App::default();
    let context = egui::Context::default();
    appearance::configure(&context);
    context.all_styles_mut(|style| style.animation_time = 0.0);
    app.appearance.apply_theme(&context);
    let _ = render(&mut app, &context, 780.0, Vec::new());
    click_label(&mut app, &context, "Appearance");
    click_label(&mut app, &context, "Dark mode");
    assert!(app.appearance.dark_mode && context.style().visuals.dark_mode);
    for (index, font) in appearance::FONTS.iter().enumerate() {
        click_label(&mut app, &context, font.name);
        assert_eq!(app.appearance.font, Some(index));
        assert_eq!(app.appearance.font_name("Default"), font.name);
        assert!(egui::Popup::is_any_open(&context));
    }
    click_label(&mut app, &context, "Dark mode");
    assert!(!app.appearance.dark_mode && !context.style().visuals.dark_mode);
    assert_eq!(app.appearance.font, Some(appearance::FONTS.len() - 1));
    click_label(&mut app, &context, "Ubuntu Light (default)");
    assert_eq!(app.appearance.font, None);
    click_label(&mut app, &context, "Appearance");
    assert!(!egui::Popup::is_any_open(&context));
}

#[test]
fn language_selector_translates_controls_without_changing_the_workspace() -> anyhow::Result<()> {
    let tmp = tempfile::tempdir()?;
    let path = remain_core::demo::create_classified(tmp.path().join("demo"), "en")?;
    let mut app = App::default();
    app.load(path);
    let original =
        serde_json::to_value(app.workspace.as_ref().expect("loaded workspace").config())?;
    let context = egui::Context::default();
    appearance::configure(&context);
    context.all_styles_mut(|style| style.animation_time = 0.0);
    let mut current = "English";
    for (name, code, caption) in [
        ("Deutsch", "de", "Konfiguration speichern"),
        ("Français", "fr", "Enregistrer la configuration"),
        ("English", "en", "Save configuration"),
    ] {
        click_label(&mut app, &context, current);
        click_label(&mut app, &context, name);
        assert_eq!(app.language, code);
        assert_eq!(app.t("save"), caption);
        assert_eq!(
            serde_json::to_value(app.workspace.as_ref().expect("loaded workspace").config())?,
            original
        );
        current = name;
    }
    Ok(())
}
