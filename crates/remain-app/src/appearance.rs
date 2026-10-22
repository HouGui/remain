use eframe::egui::{self, Color32};
use std::path::Path;

const TEAL: Color32 = Color32::from_rgb(35, 112, 108);

pub(crate) fn icon() -> egui::IconData {
    eframe::icon_data::from_png_bytes(include_bytes!("../../../packaging/remain.png"))
        .expect("bundled application icon is valid PNG")
}

pub(crate) struct Font {
    pub id: &'static str,
    pub name: &'static str,
    pub data: &'static [u8],
    pub license: &'static str,
}

pub(crate) const FONTS: &[Font] = &[
    Font {
        id: "noto-sans",
        name: "Noto Sans",
        data: include_bytes!("../../../assets/fonts/NotoSans-Regular.ttf"),
        license: include_str!("../../../assets/fonts/NotoSans-OFL.txt"),
    },
    Font {
        id: "lato",
        name: "Lato",
        data: include_bytes!("../../../assets/fonts/Lato-Regular.ttf"),
        license: include_str!("../../../assets/fonts/Lato-OFL.txt"),
    },
    Font {
        id: "atkinson-hyperlegible",
        name: "Atkinson Hyperlegible",
        data: include_bytes!("../../../assets/fonts/AtkinsonHyperlegible-Regular.ttf"),
        license: include_str!("../../../assets/fonts/AtkinsonHyperlegible-OFL.txt"),
    },
];

#[derive(Default)]
pub(crate) struct Appearance {
    pub dark_mode: bool,
    // None selects egui's default font. Persist the font ID, never this index.
    pub font: Option<usize>,
    custom: Option<(String, Vec<u8>)>,
}

impl Appearance {
    pub fn restore(storage: Option<&dyn eframe::Storage>) -> Self {
        let mut appearance = Self::default();
        if let Some(storage) = storage {
            appearance.dark_mode = storage.get_string("dark-mode").as_deref() == Some("true");
            appearance.font = storage
                .get_string("display-font")
                .and_then(|id| FONTS.iter().position(|font| font.id == id));
        }
        appearance
    }

    pub fn save(&self, storage: &mut dyn eframe::Storage) {
        storage.set_string("dark-mode", self.dark_mode.to_string());
        storage.set_string(
            "display-font",
            self.font.map_or("default", |index| FONTS[index].id).into(),
        );
    }

    pub fn apply_theme(&self, context: &egui::Context) {
        context.set_theme(if self.dark_mode {
            egui::Theme::Dark
        } else {
            egui::Theme::Light
        });
    }

    pub fn font_name<'a>(&'a self, default: &'a str) -> &'a str {
        self.custom.as_ref().map_or_else(
            || self.font.map_or(default, |index| FONTS[index].name),
            |(name, _)| name.as_str(),
        )
    }

    pub fn has_custom_font(&self) -> bool {
        self.custom.is_some()
    }

    pub fn select_font(&mut self, font: Option<usize>, context: &egui::Context) {
        self.font = font;
        self.custom = None;
        self.apply_font(context);
    }

    pub fn load_custom_font(&mut self, path: &Path, context: &egui::Context) -> anyhow::Result<()> {
        let bytes = std::fs::read(path)?;
        anyhow::ensure!(
            ttf_parser::Face::parse(&bytes, 0).is_ok(),
            "This file is not a supported TTF/OTF font."
        );
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into();
        self.custom = Some((name, bytes));
        self.apply_font(context);
        Ok(())
    }

    pub fn apply_font(&self, context: &egui::Context) {
        let mut definitions = egui::FontDefinitions::default();
        let data = self.custom.as_ref().map_or_else(
            || {
                self.font
                    .map(|index| egui::FontData::from_static(FONTS[index].data))
            },
            |(_, bytes)| Some(egui::FontData::from_owned(bytes.clone())),
        );
        if let Some(data) = data {
            definitions
                .font_data
                .insert("display-font".into(), data.into());
            definitions
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "display-font".into());
        }
        // Preserve egui's symbol/emoji fallbacks and monospace family.
        context.set_fonts(definitions);
    }
}

pub(crate) fn configure(context: &egui::Context) {
    context.all_styles_mut(|style| {
        style.spacing.item_spacing = egui::vec2(12.0, 10.0);
        style.spacing.button_padding = egui::vec2(14.0, 9.0);
        style.visuals.selection.bg_fill = TEAL;
        style.visuals.selection.stroke.color = Color32::WHITE;
        style.visuals.hyperlink_color = accent(&style.visuals);
        style.visuals.warn_fg_color = if style.visuals.dark_mode {
            Color32::from_rgb(255, 194, 125)
        } else {
            Color32::from_rgb(156, 82, 0)
        };
        style.visuals.error_fg_color = if style.visuals.dark_mode {
            Color32::from_rgb(255, 137, 127)
        } else {
            Color32::from_rgb(166, 39, 39)
        };
    });
}

pub(crate) fn accent(visuals: &egui::Visuals) -> Color32 {
    if visuals.dark_mode {
        Color32::from_rgb(105, 200, 190)
    } else {
        TEAL
    }
}

pub(crate) fn negative(visuals: &egui::Visuals) -> Color32 {
    if visuals.dark_mode {
        Color32::from_rgb(240, 156, 110)
    } else {
        Color32::from_rgb(176, 82, 39)
    }
}

#[cfg(test)]
#[path = "appearance_tests.rs"]
mod tests;
