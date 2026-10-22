//! Language changes preserve numbers, source headers, and accented report text.
mod common;

use anyhow::Result;
use calamine::{Data, Reader, Xlsx, open_workbook};
use lopdf::Document;
use remain_core::{Workspace, locale};
use std::fs;

#[test]
fn output_language_preserves_literal_names_and_source_mappings() -> Result<()> {
    for source_language in ["en", "de", "fr"] {
        let (tmp, workspace) = common::classified_workspace(source_language)?;
        let mut config = workspace.config().clone();
        let mut projects = workspace.projects().to_vec();
        config.title = "Annual review / Bilan annuel".into();
        config.groups[1].label = "Forschung und Bildung".into();
        config.categories[0].label = "Économie circulaire".into();
        projects[1].label = "Learning pilot".into();
        config.metrics[0].label = "Publications / Publikationen".into();
        config.metrics[0]
            .breakdown
            .as_mut()
            .expect("classified metric")
            .classes[0]
            .label = "Conférences".into();
        let expected = serde_json::to_value(workspace.analyze()?)?;
        for output_language in ["en", "de", "fr-CA"] {
            config.locale = output_language.into();
            let output = tmp.path().join(output_language);
            let analysis =
                Workspace::with_projects(workspace.root(), config.clone(), projects.clone())?
                    .export(&output)?;
            assert_eq!(serde_json::to_value(analysis)?, expected);
            let snapshot: serde_json::Value =
                serde_json::from_slice(&fs::read(output.join("configuration.json"))?)?;
            assert_eq!(snapshot, serde_json::to_value(&config)?);
            let html = fs::read_to_string(output.join("index.html"))?;
            for name in [
                &config.title,
                &config.groups[1].label,
                &config.categories[0].label,
                &projects[1].label,
                &config.indicators()[1].label(),
            ] {
                assert!(
                    html.contains(name),
                    "{source_language}/{output_language}: {name}"
                );
            }
            assert!(html.contains(locale::text(
                output_language,
                &config.translations,
                "period"
            )));
            assert!(html.contains(&locale::number(Some(0.7), 2, output_language)));
            let mut excel: Xlsx<_> = open_workbook(output.join("annex.xlsx"))?;
            for (sheet, row, column, text) in [
                (
                    "Summary",
                    0,
                    0,
                    locale::text(output_language, &config.translations, "group"),
                ),
                ("Summary", 3, 0, config.groups[1].label.as_str()),
                ("Categories", 1, 0, config.categories[0].label.as_str()),
                ("Projects", 1, 1, projects[1].label.as_str()),
                ("Summary", 0, 2, config.metrics[0].label.as_str()),
                ("Summary", 0, 3, &config.indicators()[1].label()),
            ] {
                assert_eq!(
                    excel.worksheet_range(sheet)?.get_value((row, column)),
                    Some(&Data::String(text.into())),
                    "{source_language}/{output_language}: {sheet}"
                );
            }
        }
    }
    Ok(())
}

#[test]
fn translated_excel_names_are_not_accepted_as_source_aliases() -> Result<()> {
    let (_tmp, workspace) = common::classified_workspace("en")?;
    let mut config = workspace.config().clone();
    config.locale = "de".into();
    config.metrics[0].label = "Publikationen".into();
    config.metrics[0]
        .breakdown
        .as_mut()
        .expect("classified metric")
        .classes[0]
        .label = "Konferenz".into();
    let workspace = Workspace::new(workspace.root(), config)?;
    workspace.analyze()?;
    for (case, expected) in [
        ("sheet", "Cannot read worksheet"),
        ("metric", "Missing header \"Publications\""),
        ("property", "Missing header \"Publication type\""),
        ("class", "unknown class value \"Konferenz\""),
    ] {
        let mut book = rust_xlsxwriter::Workbook::new();
        let sheet = book.add_worksheet().set_name(if case == "sheet" {
            "Kennzahlen"
        } else {
            "KPIs"
        })?;
        for (column, heading) in [
            if case == "metric" {
                "Publikationen"
            } else {
                "Publications"
            },
            "Patents",
            "Quality score",
            if case == "property" {
                "Publikationstyp"
            } else {
                "Publication type"
            },
            "Assessment phase",
        ]
        .iter()
        .enumerate()
        {
            sheet.write_string(0, column as u16, *heading)?;
        }
        sheet.write_number(1, 0, 3.)?;
        sheet.write_number(1, 1, 1.)?;
        sheet.write_number(1, 2, 0.8)?;
        sheet.write_string(
            1,
            3,
            if case == "class" {
                "Konferenz"
            } else {
                "Conference"
            },
        )?;
        sheet.write_string(1, 4, "Baseline")?;
        book.save(workspace.root().join("atlas/annex.xlsx"))?;
        let error = format!(
            "{:#}",
            workspace
                .analyze()
                .expect_err("translated source must not match")
        );
        assert!(error.contains(expected), "{case}: {error}");
        assert!(error.contains("atlas"), "{error}");
    }
    Ok(())
}

#[test]
fn mixed_language_pdf_parts_keep_their_content_and_order_in_every_output_language() -> Result<()> {
    let (tmp, workspace) = common::workspace("en")?;
    for (language, path) in [("de", "nova/part-1.pdf"), ("fr", "nova/part-2.pdf")] {
        let (_source_tmp, source) = common::workspace(language)?;
        fs::copy(source.root().join(path), workspace.root().join(path))?;
    }
    let inputs = [
        "atlas/part-1.pdf",
        "nova/part-1.pdf",
        "nova/part-2.pdf",
        "orbit/part-1.pdf",
    ];
    let originals = inputs
        .iter()
        .map(|path| fs::read(workspace.root().join(path)))
        .collect::<std::io::Result<Vec<_>>>()?;
    let pages = originals
        .iter()
        .map(|bytes| {
            let pdf = Document::load_mem(bytes)?;
            Ok(pdf.get_page_content(pdf.get_pages()[&1]))
        })
        .collect::<Result<Vec<_>>>()?;
    let titles = [
        "ATLAS / Part 1",
        "NOVA / Teil 1",
        "NOVA / Partie 2",
        "ORBIT / Part 1",
    ];
    for language in ["en", "de", "fr"] {
        let mut config = workspace.config().clone();
        config.locale = language.into();
        let output = tmp.path().join(language);
        Workspace::new(workspace.root(), config)?.export(&output)?;
        for (file, indices) in [
            ("report.pdf", vec![0, 1, 2, 3]),
            ("groups/applications.pdf", vec![1, 2, 3]),
            ("categories/circularity.pdf", vec![1, 2, 3]),
            ("projects/nova.pdf", vec![1, 2]),
        ] {
            let pdf = Document::load(output.join(file))?;
            let front_pages = usize::from(!file.starts_with("projects/"));
            assert_eq!(pdf.get_pages().len(), indices.len() + front_pages);
            for ((number, id), index) in pdf.get_pages().into_iter().skip(front_pages).zip(indices)
            {
                assert!(pdf.extract_text(&[number])?.contains(titles[index]));
                assert_eq!(pdf.get_page_content(id), pages[index]);
            }
        }
    }
    for (path, original) in inputs.into_iter().zip(originals) {
        assert_eq!(fs::read(workspace.root().join(path))?, original);
    }
    Ok(())
}

#[test]
fn french_regional_exports_preserve_accents_and_numeric_values() -> Result<()> {
    let (tmp, workspace) = common::workspace("fr")?;
    assert_eq!(
        workspace
            .config()
            .annex
            .as_ref()
            .expect("KPI settings")
            .sheet,
        "Indicateurs"
    );
    assert_eq!(workspace.config().metrics[1].column, "Brevets");
    assert_eq!(workspace.config().metrics[2].column, "Indice de qualité");
    for locale in ["fr", "fr-FR", "fr-CA"] {
        let mut config = workspace.config().clone();
        config.locale = locale.into();
        config
            .translations
            .insert("total".into(), "Bilan général".into());
        let output = tmp.path().join(locale);
        let analysis = Workspace::new(workspace.root(), config)?.export(&output)?;
        assert_eq!(analysis.overall[0..2], [Some(12.), Some(4.)]);
        assert!((analysis.overall[2].expect("mean") - 0.7).abs() < 1e-12);

        let html = std::fs::read_to_string(output.join("index.html"))?;
        for expected in [
            "Bilan général",
            "Période du rapport",
            "Économie circulaire",
            "Indice de qualité",
            "0,70",
        ] {
            assert!(html.contains(expected), "{locale}: {expected}");
        }
        assert!(html.contains(&format!("lang=\"{locale}\"")));

        let mut excel: Xlsx<_> = open_workbook(output.join("annex.xlsx"))?;
        let summary = excel.worksheet_range("Summary")?;
        assert_eq!(
            summary.get_value((0, 0)),
            Some(&Data::String("Groupe".into()))
        );
        assert_eq!(
            summary.get_value((1, 0)),
            Some(&Data::String("Bilan général".into()))
        );
        let Some(Data::Float(mean)) = summary.get_value((1, 4)) else {
            panic!("French Excel values must remain numeric");
        };
        assert!((mean - 0.7).abs() < 1e-12);
        let categories = excel.worksheet_range("Categories")?;
        assert_eq!(
            categories.get_value((0, 0)),
            Some(&Data::String("Catégorie".into()))
        );
        assert_eq!(
            categories.get_value((1, 0)),
            Some(&Data::String("Économie circulaire".into()))
        );

        let pdf = Document::load(output.join("report.pdf"))?;
        let text = pdf.extract_text(&[2])?;
        assert!(text.contains("ATLAS / Partie 1"));
        assert!(text.contains("Résultats et prochaines étapes"), "{text}");
        assert!(
            text.contains("créé des modules et testé des matériaux"),
            "{text}"
        );
    }
    Ok(())
}
