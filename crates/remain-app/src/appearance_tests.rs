use super::*;
use eframe::Storage as _;
use remain_core::locale;
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn bundled_icon_decodes_with_transparency_and_visible_pixels() {
    let icon = icon();
    assert_eq!((icon.width, icon.height), (256, 256));
    assert_eq!(icon.rgba.len(), 256 * 256 * 4);
    assert_eq!(icon.rgba[3], 0);
    assert!(
        icon.rgba
            .as_chunks::<4>()
            .0
            .iter()
            .any(|pixel| pixel[3] == 255)
    );
}

#[derive(Default)]
struct Storage(BTreeMap<String, String>);

impl eframe::Storage for Storage {
    fn get_string(&self, key: &str) -> Option<String> {
        self.0.get(key).cloned()
    }

    fn set_string(&mut self, key: &str, value: String) {
        self.0.insert(key.into(), value);
    }

    fn flush(&mut self) {}
}

#[test]
fn preferences_round_trip_and_unknown_fonts_fall_back() {
    let mut storage = Storage::default();
    for dark_mode in [false, true] {
        for font in std::iter::once(None).chain((0..FONTS.len()).map(Some)) {
            let appearance = Appearance {
                dark_mode,
                font,
                ..Default::default()
            };
            appearance.save(&mut storage);
            let restored = Appearance::restore(Some(&storage));
            assert_eq!(restored.dark_mode, dark_mode);
            assert_eq!(restored.font, font);
            assert!(!restored.has_custom_font());
        }
    }
    storage.set_string("display-font", "removed-font".into());
    assert_eq!(Appearance::restore(Some(&storage)).font, None);
    assert!(Appearance::restore(Some(&storage)).dark_mode);
    storage.set_string("dark-mode", "invalid".into());
    assert!(!Appearance::restore(Some(&storage)).dark_mode);
    assert!(!Appearance::restore(None).dark_mode);
}

#[test]
fn bundled_fonts_support_interface_text_and_switch_without_losing_fallbacks() {
    let messages: BTreeMap<String, String> =
        serde_json::from_str(include_str!("../../../locales/en.json")).expect("catalog");
    let overrides = BTreeMap::new();
    let mut characters = BTreeSet::new();
    for language in locale::LANGUAGES.iter() {
        for key in messages.keys() {
            // Symbols such as arrows are supplied by egui's fallback fonts.
            characters.extend(
                locale::text(language.code, &overrides, key)
                    .chars()
                    .filter(|c| c.is_alphanumeric()),
            );
        }
    }
    let context = egui::Context::default();
    let mut appearance = Appearance::default();
    let defaults = egui::FontDefinitions::default();
    for (index, font) in FONTS.iter().enumerate() {
        let face = ttf_parser::Face::parse(font.data, 0).expect("valid bundled font");
        for &character in &characters {
            assert!(
                face.glyph_index(character).is_some(),
                "{} lacks {character}",
                font.name
            );
        }
        appearance.select_font(Some(index), &context);
        let _ = context.run(Default::default(), |ctx| {
            ctx.fonts(|fonts| {
                let definitions = fonts.definitions();
                let proportional = &definitions.families[&egui::FontFamily::Proportional];
                assert_eq!(proportional[0], "display-font");
                assert_eq!(
                    proportional[1..],
                    defaults.families[&egui::FontFamily::Proportional]
                );
                assert_eq!(
                    definitions.families[&egui::FontFamily::Monospace],
                    defaults.families[&egui::FontFamily::Monospace]
                );
            });
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.label("Français · Größe → — 0123");
            });
        });
    }
    appearance.select_font(None, &context);
    let _ = context.run(Default::default(), |ctx| {
        ctx.fonts(|fonts| assert_eq!(fonts.definitions(), &defaults));
    });
}

#[test]
fn invalid_custom_font_keeps_selection_and_custom_fonts_are_session_only() -> anyhow::Result<()> {
    let temporary = tempfile::tempdir()?;
    let path = temporary.path().join("font.ttf");
    let context = egui::Context::default();
    let mut appearance = Appearance::default();
    appearance.select_font(Some(1), &context);
    std::fs::write(&path, b"not a font")?;
    assert!(appearance.load_custom_font(&path, &context).is_err());
    assert_eq!(appearance.font_name("Default"), "Lato");
    std::fs::write(&path, FONTS[0].data)?;
    appearance.load_custom_font(&path, &context)?;
    assert_eq!(appearance.font_name("Default"), "font.ttf");
    std::fs::write(&path, b"invalid replacement")?;
    assert!(appearance.load_custom_font(&path, &context).is_err());
    assert_eq!(appearance.font_name("Default"), "font.ttf");
    let mut storage = Storage::default();
    appearance.save(&mut storage);
    assert_eq!(
        Appearance::restore(Some(&storage)).font_name("Default"),
        "Lato"
    );
    appearance.select_font(None, &context);
    assert!(!appearance.has_custom_font());
    assert_eq!(appearance.font_name("Default"), "Default");
    Ok(())
}

#[test]
fn theme_switches_keep_layout_and_font_selection() {
    let context = egui::Context::default();
    configure(&context);
    let mut appearance = Appearance::default();
    appearance.select_font(Some(0), &context);
    for dark_mode in [false, true, false] {
        appearance.dark_mode = dark_mode;
        appearance.apply_theme(&context);
        let style = context.style();
        assert_eq!(style.visuals.dark_mode, dark_mode);
        assert_eq!(style.spacing.button_padding, egui::vec2(14.0, 9.0));
        assert_eq!(style.spacing.item_spacing, egui::vec2(12.0, 10.0));
        assert_eq!(appearance.font_name("Default"), "Noto Sans");
        assert_eq!(style.visuals.hyperlink_color, accent(&style.visuals));
    }
}

#[test]
fn default_caption_names_the_primary_interface_font() {
    let definitions = egui::FontDefinitions::default();
    assert_eq!(
        definitions.families[&egui::FontFamily::Proportional][0],
        "Ubuntu-Light"
    );
    for language in remain_core::locale::LANGUAGES.iter() {
        assert!(
            remain_core::locale::text(language.code, &Default::default(), "font_default")
                .contains("Ubuntu Light")
        );
    }
}
