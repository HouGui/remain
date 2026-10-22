//! Complete bundle integration tests using real generated PDFs and workbooks.
mod common;
use anyhow::Result;
use calamine::{Data, Reader, Xlsx, open_workbook};
use lopdf::Document;
use remain_core::Workspace;
use std::fs;

#[test]
fn localized_bundles_reopen_with_expected_results() -> Result<()> {
    for (language, publications, mean) in [
        ("en", "Publications", "0.70"),
        ("de", "Publikationen", "0,70"),
        ("fr", "Publications", "0,70"),
    ] {
        let (tmp, workspace) = common::workspace(language)?;
        let output = tmp.path().join("report");
        let analysis = workspace.export(&output)?;
        assert_eq!(analysis.overall[0..2], [Some(12.), Some(4.)]);
        assert_eq!(analysis.observations.len(), 6);
        assert_eq!(analysis.projects.iter().map(|p| p.pages).sum::<usize>(), 4);
        let pdf = Document::load(output.join("report.pdf"))?;
        assert_eq!(pdf.get_pages().len(), 5);
        for (page, expected) in [(2, "ATLAS"), (3, "NOVA"), (4, "NOVA"), (5, "ORBIT")] {
            let text = pdf.extract_text(&[page])?;
            assert!(text.contains(expected), "page {page}: {text:?}");
        }
        assert_eq!(
            Document::load(output.join("groups/applications.pdf"))?
                .get_pages()
                .len(),
            4
        );
        assert_eq!(
            Document::load(output.join("projects/nova.pdf"))?
                .get_pages()
                .len(),
            2
        );
        let mut excel: Xlsx<_> = open_workbook(output.join("annex.xlsx"))?;
        assert_eq!(
            excel.sheet_names(),
            [
                "Submissions",
                "Summary",
                "Categories",
                "Projects",
                "Observations"
            ]
        );
        let summary = excel.worksheet_range("Summary")?;
        assert_eq!(summary.get_value((1, 2)), Some(&Data::Float(12.)));
        let observations = excel.worksheet_range("Observations")?;
        assert_eq!(
            observations.get_value((1, 3)),
            Some(&Data::String("atlas/annex.xlsx".into()))
        );
        assert_eq!(observations.get_value((6, 7)), Some(&Data::Empty));
        let decoded: remain_core::analysis::Analysis =
            serde_json::from_slice(&fs::read(output.join("summary.json"))?)?;
        assert_eq!(decoded.overall, analysis.overall);
        let html = fs::read_to_string(output.join("index.html"))?;
        assert!(html.contains(publications));
        assert!(html.contains(mean));
        assert!(output.join("configuration.json").is_file());
    }
    Ok(())
}

#[test]
fn output_order_is_group_then_project_then_part_order() -> Result<()> {
    let (tmp, workspace) = common::workspace("en")?;
    let mut config = workspace.config().clone();
    config.groups.reverse();
    config.inputs.documents[0].pattern = "part-1.pdf".into();
    config.inputs.documents.insert(
        0,
        remain_core::config::DocumentRule {
            label: "Extra report".into(),
            pattern: "part-2.pdf".into(),
            required: false,
        },
    );
    let workspace = Workspace::new(workspace.root(), config)?;
    let output = tmp.path().join("ordered");
    workspace.export(&output)?;
    let pdf = Document::load(output.join("report.pdf"))?;
    let first = pdf.extract_text(&[2])?;
    assert!(first.contains("NOVA / Part 2"), "{first:?}");
    assert!(pdf.extract_text(&[3])?.contains("NOVA / Part 1"));
    assert!(pdf.extract_text(&[4])?.contains("ORBIT"));
    assert!(pdf.extract_text(&[5])?.contains("ATLAS"));
    Ok(())
}

#[test]
fn parent_references_do_not_duplicate_observations() -> Result<()> {
    let (_tmp, workspace) = common::workspace("en")?;
    let with_parent = workspace.analyze()?;
    let mut projects = workspace.projects().to_vec();
    projects[2].parent = None;
    let without_parent =
        Workspace::with_projects(workspace.root(), workspace.config().clone(), projects)?
            .analyze()?;
    assert_eq!(with_parent.overall, without_parent.overall);
    assert_eq!(
        with_parent.observations.len(),
        without_parent.observations.len()
    );
    Ok(())
}

#[test]
fn existing_destinations_and_invalid_inputs_leave_existing_data_untouched() -> Result<()> {
    let (tmp, workspace) = common::workspace("en")?;
    let output = tmp.path().join("existing");
    fs::create_dir(&output)?;
    fs::write(output.join("sentinel"), "keep")?;
    assert!(workspace.export(&output).is_err());
    assert_eq!(fs::read_to_string(output.join("sentinel"))?, "keep");
    fs::write(workspace.root().join("atlas/part-1.pdf"), "not a PDF")?;
    let missing = tmp.path().join("failed");
    assert!(workspace.export(&missing).is_err());
    assert!(!missing.exists());
    assert!(
        !fs::read_dir(tmp.path())?
            .any(|e| e.is_ok_and(|e| e.file_name().to_string_lossy().starts_with(".remain-")))
    );
    Ok(())
}

#[test]
fn export_revalidates_inputs_after_previous_analysis() -> Result<()> {
    let (tmp, workspace) = common::workspace("en")?;
    workspace.analyze()?;
    fs::write(
        workspace.root().join("orbit/annex.xlsx"),
        "invalid workbook",
    )?;
    assert!(workspace.export(tmp.path().join("report")).is_err());
    Ok(())
}

#[test]
fn empty_groups_have_no_phantom_pdf_or_broken_html_link() -> Result<()> {
    let (tmp, workspace) = common::workspace("en")?;
    let mut config = workspace.config().clone();
    config.groups.push(remain_core::config::Classification {
        id: "empty".into(),
        label: "Empty".into(),
    });
    let output = tmp.path().join("report");
    let analysis = Workspace::new(workspace.root(), config)?.export(&output)?;
    assert_eq!(analysis.groups[2].projects, 0);
    assert_eq!(analysis.groups[2].values, vec![None; 3]);
    assert!(!output.join("groups/empty.pdf").exists());
    assert!(!fs::read_to_string(output.join("index.html"))?.contains("groups/empty.pdf"));
    Ok(())
}

#[test]
fn labels_are_html_escaped_and_excel_strings_never_become_formulas() -> Result<()> {
    let (tmp, workspace) = common::workspace("en")?;
    let mut config = workspace.config().clone();
    config.title = "<script>alert('x')</script>".into();
    config.metrics[0].label = "=SUM(1,2)".into();
    let output = tmp.path().join("report");
    Workspace::new(workspace.root(), config)?.export(&output)?;
    let html = fs::read_to_string(output.join("index.html"))?;
    assert!(!html.contains("<script>"));
    assert!(html.contains("&lt;script&gt;"));
    let mut excel: Xlsx<_> = open_workbook(output.join("annex.xlsx"))?;
    assert!(
        excel
            .worksheet_formula("Summary")?
            .used_cells()
            .all(|(_, _, formula)| formula.is_empty())
    );
    assert_eq!(
        excel.worksheet_range("Summary")?.get_value((0, 2)),
        Some(&Data::String("=SUM(1,2)".into()))
    );
    Ok(())
}

#[test]
fn demo_creation_never_overwrites_and_does_not_accept_unknown_locales() -> Result<()> {
    let (tmp, workspace) = common::workspace("en")?;
    let before = fs::read(workspace.root().join("workspace.json"))?;
    assert!(remain_core::demo::create(workspace.root(), "en").is_err());
    assert_eq!(fs::read(workspace.root().join("workspace.json"))?, before);
    assert!(remain_core::demo::create(tmp.path().join("other"), "zz").is_err());
    Ok(())
}

#[test]
fn checked_in_workspaces_match_generated_configurations_and_results() -> Result<()> {
    let examples = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples");
    for (name, language, classified) in [
        ("demo-en", "en", false),
        ("demo-de", "de", false),
        ("demo-fr", "fr", false),
        ("demo-classes", "en", true),
    ] {
        let (_tmp, generated) = if classified {
            common::classified_workspace(language)?
        } else {
            common::workspace(language)?
        };
        let fixture = Workspace::load(examples.join(name).join("workspace.json"))?;
        assert_eq!(
            serde_json::to_value(fixture.config())?,
            serde_json::to_value(generated.config())?,
            "{name}"
        );
        assert_eq!(
            serde_json::to_value(fixture.analyze()?)?,
            serde_json::to_value(generated.analyze()?)?,
            "{name}"
        );
    }
    Ok(())
}
