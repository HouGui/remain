use crate::config::{
    Aggregation, Annex, Classification, Config, DocumentRule, InputRules, Metric, Missing, Project,
    SCHEMA_VERSION,
};
use std::collections::BTreeMap;

pub(super) fn localized(locale: &str, en: &str, de: &str, fr: &str) -> String {
    match locale {
        "de" => de,
        "fr" => fr,
        _ => en,
    }
    .into()
}

pub(super) fn configuration(locale: &str) -> (Config, Vec<Project>) {
    let text = |en, de, fr| localized(locale, en, de, fr);
    let metrics = [
        (
            "publications",
            "Publications",
            "Publikationen",
            "Publications",
            Aggregation::Sum,
            Missing::Error,
            0,
        ),
        (
            "patents",
            "Patents",
            "Patente",
            "Brevets",
            Aggregation::Sum,
            Missing::Error,
            0,
        ),
        (
            "quality",
            "Quality score",
            "Qualitätswert",
            "Indice de qualité",
            Aggregation::Mean,
            Missing::Skip,
            2,
        ),
    ]
    .into_iter()
    .map(|(id, en, de, fr, aggregation, missing, decimals)| {
        let label = text(en, de, fr);
        Metric {
            id: id.into(),
            column: label.clone(),
            label,
            aggregation,
            missing,
            unit: String::new(),
            decimals,
            breakdown: None,
        }
    })
    .collect();
    let projects = [
        (
            "atlas",
            "Open learning",
            "Offenes Lernen",
            "Apprentissage ouvert",
            "discovery",
            None,
        ),
        (
            "nova",
            "Circular materials",
            "Zirkuläre Materialien",
            "Matériaux circulaires",
            "applications",
            None,
        ),
        (
            "orbit",
            "Reuse pilot",
            "Pilotprojekt Wiederverwendung",
            "Projet pilote de réemploi",
            "applications",
            Some("nova"),
        ),
    ]
    .into_iter()
    .map(|(id, en, de, fr, group, parent)| Project {
        id: id.into(),
        label: text(en, de, fr),
        group: Some(group.into()),
        category: (id != "atlas").then(|| "circularity".into()),
        parent: parent.map(String::from),
    })
    .collect();
    (
        Config {
            schema_version: SCHEMA_VERSION,
            title: text(
                "Example organization · Portfolio report",
                "Beispielorganisation · Portfoliobericht",
                "Organisation fictive · Rapport de portefeuille",
            ),
            locale: locale.into(),
            period: "2026".into(),
            annex: Some(Annex {
                sheet: text("KPIs", "Kennzahlen", "Indicateurs"),
                header_row: 1,
            }),
            groups: vec![
                Classification {
                    id: "discovery".into(),
                    label: text(
                        "Discovery and education",
                        "Forschung und Bildung",
                        "Recherche et éducation",
                    ),
                },
                Classification {
                    id: "applications".into(),
                    label: text(
                        "Industrial applications",
                        "Industrielle Anwendungen",
                        "Applications industrielles",
                    ),
                },
            ],
            categories: vec![Classification {
                id: "circularity".into(),
                label: text("Circularity", "Kreislaufwirtschaft", "Économie circulaire"),
            }],
            projects: "projects.csv".into(),
            inputs: InputRules {
                documents: vec![
                    DocumentRule {
                        label: text("Report", "Bericht", "Rapport"),
                        pattern: "part-*.pdf".into(),
                        required: true,
                    },
                    DocumentRule {
                        label: text("KPI annex", "Kennzahlenanlage", "Annexe des indicateurs"),
                        pattern: "annex.xlsx".into(),
                        required: false,
                    },
                ],
                ..InputRules::default()
            },
            contents_font: None,
            metrics,
            translations: BTreeMap::new(),
        },
        projects,
    )
}
