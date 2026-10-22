//! Shared text catalogs for the desktop interface and generated reports.

use std::{collections::BTreeMap, sync::LazyLock};

/// A bundled language used by the GUI, report text, and number formatting.
pub struct Language {
    /// Base language tag, such as `fr`.
    pub code: &'static str,
    /// Language name in its own language, for selection controls.
    pub name: &'static str,
    decimal_comma: bool,
    messages: BTreeMap<String, String>,
}

/// Built-in languages in display order. English is the fallback language.
pub static LANGUAGES: LazyLock<Vec<Language>> = LazyLock::new(|| {
    [
        (
            "en",
            "English",
            false,
            include_str!("../../../locales/en.json"),
        ),
        (
            "de",
            "Deutsch",
            true,
            include_str!("../../../locales/de.json"),
        ),
        (
            "fr",
            "Français",
            true,
            include_str!("../../../locales/fr.json"),
        ),
    ]
    .into_iter()
    .map(|(code, name, decimal_comma, catalog)| Language {
        code,
        name,
        decimal_comma,
        messages: serde_json::from_str(catalog).expect("valid bundled language catalog"),
    })
    .collect()
});

/// Find a bundled language by its base tag, including regional tags such as
/// `fr-CA`. Returns `None` for languages without a bundled catalog.
pub fn builtin(locale: &str) -> Option<&'static Language> {
    let base = locale.split('-').next().unwrap_or(locale);
    LANGUAGES.iter().find(|language| language.code == base)
}

pub(crate) fn is_key(key: &str) -> bool {
    LANGUAGES[0].messages.contains_key(key)
}

/// Resolve a catalog key, using an optional workspace override before the
/// built-in language and English fallback. Unknown keys are returned unchanged.
pub fn text<'a>(locale: &str, overrides: &'a BTreeMap<String, String>, key: &'a str) -> &'a str {
    overrides
        .get(key)
        .map(String::as_str)
        .or_else(|| {
            builtin(locale)
                .and_then(|language| language.messages.get(key))
                .map(String::as_str)
        })
        .or_else(|| LANGUAGES[0].messages.get(key).map(String::as_str))
        .unwrap_or(key)
}

/// Format a finite metric for display. Missing values remain visibly absent.
/// English uses a decimal point; German and French use a comma. No thousands
/// separators are inserted, and the underlying numeric value is not changed.
pub fn number(value: Option<f64>, decimals: u8, locale: &str) -> String {
    let Some(value) = value else {
        return "—".into();
    };
    let value = if value == 0.0 { 0.0 } else { value };
    let formatted = format!("{value:.precision$}", precision = decimals as usize);
    if builtin(locale).is_some_and(|language| language.decimal_comma) {
        formatted.replace('.', ",")
    } else {
        formatted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalogs_have_identical_keys_and_nonempty_values() {
        let english = &LANGUAGES[0].messages;
        let mut codes = std::collections::HashSet::new();
        for language in LANGUAGES.iter() {
            assert!(codes.insert(language.code), "duplicate language");
            assert!(!language.name.trim().is_empty());
            assert_eq!(
                english.keys().collect::<Vec<_>>(),
                language.messages.keys().collect::<Vec<_>>(),
                "{}",
                language.code
            );
            assert!(language.messages.values().all(|s| !s.trim().is_empty()));
        }
    }

    #[test]
    fn locale_resolution_and_overrides_are_deterministic() {
        let overrides = BTreeMap::from([("open".into(), "Ouvrir le dossier".into())]);
        assert_eq!(text("de-AT", &overrides, "save"), "Konfiguration speichern");
        assert_eq!(text("fr-CA", &overrides, "open"), "Ouvrir le dossier");
        assert_eq!(
            text("fr-CA", &overrides, "save"),
            "Enregistrer la configuration"
        );
        assert_eq!(text("ja", &overrides, "save"), "Save configuration");
        assert_eq!(text("en", &overrides, "unknown"), "unknown");
    }

    #[test]
    fn zero_and_missing_remain_distinct_in_all_languages() {
        for locale in ["en", "de", "fr", "fr-CA"] {
            assert_eq!(number(None, 2, locale), "—");
            let zero = if locale == "en" { "0.00" } else { "0,00" };
            assert_eq!(number(Some(0.), 2, locale), zero);
            assert_eq!(number(Some(-0.), 2, locale), zero);
        }
        assert_eq!(number(Some(-0.), 2, "en"), "0.00");
        assert_eq!(number(Some(12.345), 2, "de-DE"), "12,35");
        assert_eq!(number(Some(-12.345), 2, "fr-FR"), "-12,35");
        assert_eq!(number(Some(12.345), 2, "ja"), "12.35");
    }
}
