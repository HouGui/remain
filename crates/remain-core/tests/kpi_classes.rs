//! Class breakdowns use the same numeric rules and output order at every scope.
mod common;
use anyhow::Result;
use calamine::{Data, Reader, Xlsx, open_workbook};
use remain_core::{Workspace, analysis::Analysis, config::Aggregation};

#[test]
fn classes_partition_totals_and_preserve_weighted_means() -> Result<()> {
    let (_tmp, workspace) = common::classified_workspace("en")?;
    let analysis = workspace.analyze()?;
    assert_eq!(
        analysis.metric_ids,
        [
            "publications",
            "publications.conference",
            "publications.journal",
            "publications.book",
            "patents",
            "quality",
            "quality.baseline",
            "quality.followup"
        ]
    );
    assert_eq!(
        &analysis.overall[..5],
        &[Some(12.), Some(6.), Some(6.), None, Some(4.)]
    );
    for (value, expected) in analysis.overall[5..].iter().zip([0.7, 1.9 / 3., 0.8]) {
        assert!((value.expect("observed mean") - expected).abs() < 1e-12);
    }
    assert_eq!(analysis.observations.len(), 6);
    assert_eq!(analysis.projects.len(), 3);
    let orbit = analysis
        .projects
        .iter()
        .find(|p| p.id == "orbit")
        .expect("project");
    assert_eq!(
        &orbit.values[..5],
        &[Some(3.), Some(3.), Some(0.), None, Some(1.)]
    );
    let empty = analysis
        .groups
        .iter()
        .find(|g| g.id.as_deref() == Some("discovery"))
        .expect("empty group");
    assert_eq!(empty.projects, 0);
    assert!(empty.values.iter().all(Option::is_none));
    let free = analysis
        .groups
        .iter()
        .find(|g| g.id.is_none())
        .expect("unassigned");
    assert_eq!(
        &free.values[..5],
        &[Some(5.), Some(3.), Some(2.), None, Some(1.)]
    );
    let category = &analysis.categories[0];
    assert_eq!(
        &category.values[..5],
        &[Some(7.), Some(3.), Some(4.), None, Some(3.)]
    );
    for record in &analysis.observations {
        let present: Vec<_> = record.values[1..4].iter().flatten().copied().collect();
        assert_eq!(present, vec![record.values[0].expect("publication total")]);
    }
    Ok(())
}

#[test]
fn every_aggregation_applies_to_classes_including_zero_and_empty() -> Result<()> {
    let (_tmp, workspace) = common::classified_workspace("en")?;
    for (aggregation, expected) in [
        (Aggregation::Sum, [Some(12.), Some(6.), Some(6.), None]),
        (Aggregation::Mean, [Some(2.), Some(2.), Some(2.), None]),
        (Aggregation::Min, [Some(0.), Some(1.), Some(0.), None]),
        (Aggregation::Max, [Some(4.), Some(3.), Some(4.), None]),
        (Aggregation::Count, [Some(6.), Some(3.), Some(3.), Some(0.)]),
    ] {
        let mut config = workspace.config().clone();
        config.metrics[0].aggregation = aggregation;
        let analysis = Workspace::new(workspace.root(), config)?.analyze()?;
        assert_eq!(analysis.overall[..4], expected, "{aggregation:?}");
    }
    Ok(())
}

#[test]
fn class_exports_keep_labels_numbers_and_provenance_in_all_languages() -> Result<()> {
    for locale in ["en", "de", "fr"] {
        let (tmp, workspace) = common::classified_workspace(locale)?;
        let output = tmp.path().join("report");
        let analysis = workspace.export(&output)?;
        let summary: Analysis =
            serde_json::from_slice(&std::fs::read(output.join("summary.json"))?)?;
        assert_eq!(analysis.metric_ids, summary.metric_ids);
        assert_eq!(analysis.overall, summary.overall);
        let config: remain_core::config::Config =
            serde_json::from_slice(&std::fs::read(output.join("configuration.json"))?)?;
        assert_eq!(config.indicators()[1].id, "publications.conference");
        let html = std::fs::read_to_string(output.join("index.html"))?;
        let mut excel: Xlsx<_> = open_workbook(output.join("annex.xlsx"))?;
        for (sheet, offset, first_row) in [
            ("Summary", 2, 1),
            ("Categories", 2, 1),
            ("Projects", 7, 1),
            ("Observations", 5, 1),
        ] {
            let range = excel.worksheet_range(sheet)?;
            let indicators = config.indicators();
            for (index, indicator) in indicators.iter().enumerate() {
                assert_eq!(
                    range.get_value((0, offset + index as u32)),
                    Some(&Data::String(indicator.label()))
                );
                assert!(
                    html.contains(&indicator.label()),
                    "{locale}: {}",
                    indicator.id
                );
            }
            assert!(matches!(
                range.get_value((first_row, offset)),
                Some(Data::Float(_)) | Some(Data::Int(_))
            ));
        }
        let sheet = excel.worksheet_range("Summary")?;
        assert_eq!(sheet.get_value((1, 3)), Some(&Data::Float(6.)));
        assert!(matches!(sheet.get_value((1, 5)), None | Some(Data::Empty)));
        assert!(output.join("projects/atlas.pdf").is_file());
        assert!(!output.join("groups/discovery.pdf").exists());
    }
    Ok(())
}
