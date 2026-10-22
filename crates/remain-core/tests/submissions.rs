//! Roster import, deterministic discovery, and partial reporting boundaries.
mod common;
use anyhow::Result;
use calamine::{Data, Reader, Xlsx, open_workbook};
use lopdf::Document;
use remain_core::{
    Workspace,
    config::{Aggregation, DocumentRule, Project},
    projects,
};
use std::{fs, path::Path};

fn pdf_only(workspace: &Workspace) -> Result<Workspace> {
    let mut config = workspace.config().clone();
    config.inputs.documents.retain(|d| !d.is_kpi());
    config.annex = None;
    config.metrics.clear();
    Workspace::with_projects(workspace.root(), config, workspace.projects().to_vec())
}

#[test]
fn csv_and_json_rosters_support_imports_quoting_and_optional_memberships() -> Result<()> {
    let (tmp, workspace) = common::workspace("en")?;
    let path = tmp.path().join("import.CSV");
    fs::write(&path, "\u{feff} label ,id,owner,group,category,parent\r\n\"Étude, \\\"test\\\"\", p1 ,Manager, , , \r\n".replace("\\\"", "\"\""))?;
    let imported = projects::load(&path)?;
    assert_eq!(imported[0].id, "p1");
    assert_eq!(imported[0].label, "Étude, \"test\"");
    assert!(
        imported[0].group.is_none()
            && imported[0].category.is_none()
            && imported[0].parent.is_none()
    );
    workspace.config().validate_projects(&imported)?;
    let mut roster = workspace.projects().to_vec();
    roster[0].label = "Énergie, recherche\n\"2026\"".into();
    for extension in ["csv", "JSON"] {
        let path = tmp.path().join(format!("projects.{extension}"));
        fs::write(&path, projects::encode(&path, &roster)?)?;
        assert_eq!(
            serde_json::to_value(projects::load(&path)?)?,
            serde_json::to_value(&roster)?
        );
        fs::write(&path, projects::encode(&path, &[])?)?;
        assert!(projects::load(&path)?.is_empty());
    }
    Ok(())
}

#[test]
fn malformed_rosters_and_invalid_names_fail_with_context() -> Result<()> {
    let (tmp, workspace) = common::workspace("en")?;
    let path = tmp.path().join("projects.csv");
    for csv in [
        "",
        "id,name\np1,Name\n",
        "id,label,id\np1,Name,p1\n",
        "id,label,\np1,Name,x\n",
        "id,label\np1,Name,extra\n",
    ] {
        fs::write(&path, csv)?;
        let error = format!("{:#}", projects::load(&path).expect_err("invalid import"));
        assert!(error.contains("projects.csv"));
    }
    for label in [
        serde_json::json!(""),
        serde_json::json!("  "),
        serde_json::json!({"en":"Project"}),
        serde_json::Value::Null,
    ] {
        let value = serde_json::json!({"id":"p1", "label":label});
        if let Ok(project) = serde_json::from_value::<Project>(value) {
            assert!(workspace.config().validate_projects(&[project]).is_err());
        }
    }
    let mut config = workspace.config().clone();
    config.projects = "absent.csv".into();
    assert!(Workspace::new(workspace.root(), config).is_err());
    Ok(())
}

#[test]
fn pdf_only_workspace_ignores_unconfigured_workbooks_and_exports_contents() -> Result<()> {
    let (tmp, workspace) = common::workspace("en")?;
    let workspace = pdf_only(&workspace)?;
    fs::write(workspace.root().join("atlas/annex.xlsx"), "irrelevant")?;
    let output = tmp.path().join("pdf-only");
    let analysis = workspace.export(&output)?;
    assert!(analysis.is_complete());
    assert_eq!(analysis.projects.len(), 3);
    assert!(
        analysis.metric_ids.is_empty()
            && analysis.observations.is_empty()
            && analysis.overall.is_empty()
    );
    assert!(!output.join("annex.xlsx").exists());
    let html = fs::read_to_string(output.join("index.html"))?;
    assert!(!html.contains("href=\"annex.xlsx\""));
    let pdf = Document::load(output.join("report.pdf"))?;
    assert_eq!(pdf.get_pages().len(), 5);
    let contents = pdf.extract_text(&[1])?;
    assert!(
        contents.contains("Contents") && contents.contains("atlas") && contents.contains("nova"),
        "{contents}"
    );
    assert!(pdf.extract_text(&[2])?.contains("ATLAS"));
    let snapshot = projects::load(output.join("project-list.json"))?;
    assert_eq!(
        serde_json::to_value(snapshot)?,
        serde_json::to_value(workspace.projects())?
    );
    Ok(())
}

#[test]
fn nested_source_paths_stay_portable_in_discovery_and_exports() -> Result<()> {
    let (tmp, workspace) = common::workspace("en")?;
    let mut config = workspace.config().clone();
    config.inputs.directory = "inputs/{id}/été".into();
    for project in workspace.projects() {
        let parent = workspace.root().join("inputs").join(&project.id);
        fs::create_dir_all(&parent)?;
        fs::rename(workspace.root().join(&project.id), parent.join("été"))?;
    }
    fs::remove_file(workspace.root().join("inputs/nova/été/annex.xlsx"))?;
    fs::remove_file(workspace.root().join("inputs/orbit/été/part-1.pdf"))?;
    let workspace = Workspace::new(workspace.root(), config)?;
    // Compare serialized strings, since Path equality can hide separator differences.
    let submissions = serde_json::to_value(workspace.submissions()?)?;
    assert_eq!(submissions[0]["annex"], "inputs/atlas/été/annex.xlsx");
    assert_eq!(
        submissions[0]["reports"],
        serde_json::json!(["inputs/atlas/été/part-1.pdf"])
    );
    assert_eq!(
        submissions[1]["reports"],
        serde_json::json!(["inputs/nova/été/part-1.pdf", "inputs/nova/été/part-2.pdf"])
    );
    assert!(submissions[1]["annex"].is_null());
    let optional = format!(
        "{}: inputs/nova/été/annex.xlsx",
        workspace.config().inputs.documents[1].label
    );
    let missing = format!(
        "{}: inputs/orbit/été/part-*.pdf",
        workspace.config().inputs.documents[0].label
    );
    assert_eq!(
        submissions[1]["optional_missing"],
        serde_json::json!([optional])
    );
    assert_eq!(submissions[2]["missing"], serde_json::json!([missing]));

    let output = tmp.path().join("portable");
    workspace.export(&output)?;
    let json: serde_json::Value = serde_json::from_slice(&fs::read(output.join("summary.json"))?)?;
    assert_eq!(json["submissions"], submissions);
    let mut excel: Xlsx<_> = open_workbook(output.join("annex.xlsx"))?;
    assert_eq!(
        excel.worksheet_range("Observations")?.get_value((1, 3)),
        Some(&Data::String("inputs/atlas/été/annex.xlsx".into()))
    );
    let status = excel.worksheet_range("Submissions")?;
    assert_eq!(
        status.get_value((8, 4)),
        Some(&Data::String(optional.clone()))
    );
    assert_eq!(
        status.get_value((9, 3)),
        Some(&Data::String(missing.clone()))
    );
    let html = fs::read_to_string(output.join("index.html"))?;
    assert!(html.contains(&optional) && html.contains(&missing));
    let contents = Document::load(output.join("report.pdf"))?.extract_text(&[1])?;
    assert!(contents.contains(&missing), "{contents}");
    Ok(())
}

#[test]
fn optional_kpi_absence_is_not_zero_and_does_not_exclude_pdfs() -> Result<()> {
    let (tmp, workspace) = common::workspace("en")?;
    let mut config = workspace.config().clone();
    config.metrics[0].aggregation = Aggregation::Count;
    let workspace = Workspace::new(workspace.root(), config)?;
    fs::remove_file(workspace.root().join("atlas/annex.xlsx"))?;
    common::replace_annex(workspace.root(), |_| Ok(()))?;
    let empty = workspace.analyze()?;
    assert_eq!(empty.projects[0].values[0], Some(0.));
    fs::remove_file(workspace.root().join("atlas/annex.xlsx"))?;
    let analysis = workspace.export(tmp.path().join("optional"))?;
    assert!(analysis.is_complete());
    assert_eq!(analysis.projects.len(), 3);
    assert!(!analysis.projects[0].has_kpis);
    assert!(analysis.projects[0].values.iter().all(Option::is_none));
    assert_eq!(analysis.overall[0], Some(4.));
    assert_eq!(analysis.submissions[0].optional_missing.len(), 1);
    for project in workspace.projects().iter().skip(1) {
        fs::remove_file(workspace.root().join(&project.id).join("annex.xlsx"))?;
    }
    assert!(workspace.analyze()?.overall.iter().all(Option::is_none));
    Ok(())
}

#[test]
fn missing_required_submissions_are_visible_in_every_export() -> Result<()> {
    let (tmp, workspace) = common::workspace("en")?;
    fs::remove_file(workspace.root().join("atlas/part-1.pdf"))?;
    let output = tmp.path().join("partial");
    let analysis = workspace.export(&output)?;
    assert!(!analysis.is_complete());
    assert_eq!(analysis.projects.len(), 2);
    assert_eq!(analysis.overall[0], Some(7.));
    assert_eq!(analysis.groups[0].projects, 0);
    assert_eq!(analysis.groups[0].expected_projects, 1);
    assert!(!output.join("projects/atlas.pdf").exists());
    for file in ["report.pdf", "groups/discovery.pdf"] {
        let text = Document::load(output.join(file))?.extract_text(&[1])?;
        assert!(
            text.contains("Incomplete report") && text.contains("atlas/part-*.pdf"),
            "{text}"
        );
    }
    let html = fs::read_to_string(output.join("index.html"))?;
    assert!(html.contains("Incomplete report") && html.contains("atlas/part-*.pdf"));
    assert!(html.contains("groups/discovery.pdf"));
    let json: serde_json::Value = serde_json::from_slice(&fs::read(output.join("summary.json"))?)?;
    assert_eq!(json["complete"], false);
    assert!(
        !json["submissions"][0]["missing"]
            .as_array()
            .expect("fixture value")
            .is_empty()
    );
    let mut excel: Xlsx<_> = open_workbook(output.join("annex.xlsx"))?;
    let status = excel.worksheet_range("Submissions")?;
    assert_eq!(
        status.get_value((0, 0)),
        Some(&Data::String("Incomplete report".into()))
    );
    assert_eq!(
        status.get_value((7, 0)),
        Some(&Data::String("atlas".into()))
    );
    assert_eq!(
        excel.worksheet_range("Summary")?.get_value((1, 2)),
        Some(&Data::Float(7.))
    );
    Ok(())
}

#[test]
fn required_workbooks_and_missing_folders_exclude_whole_projects() -> Result<()> {
    let (tmp, workspace) = common::workspace("en")?;
    let mut config = workspace.config().clone();
    config
        .inputs
        .documents
        .iter_mut()
        .find(|d| d.is_kpi())
        .expect("fixture value")
        .required = true;
    fs::remove_file(workspace.root().join("atlas/annex.xlsx"))?;
    fs::rename(workspace.root().join("nova"), tmp.path().join("nova"))?;
    let workspace = Workspace::new(workspace.root(), config)?;
    let analysis = workspace.analyze()?;
    assert_eq!(analysis.projects.len(), 1);
    assert_eq!(analysis.projects[0].id, "orbit");
    assert_eq!(analysis.submissions[1].missing.len(), 2);
    assert_eq!(analysis.overall[0], Some(3.));
    // A supplied but invalid optional workbook must still fail content validation.
    fs::write(workspace.root().join("orbit/annex.xlsx"), "broken")?;
    assert!(workspace.export(tmp.path().join("invalid")).is_err());
    assert!(!tmp.path().join("invalid").exists());
    Ok(())
}

#[test]
fn empty_and_all_missing_workspaces_can_be_reviewed_but_not_exported() -> Result<()> {
    let (tmp, workspace) = common::workspace("en")?;
    let empty = Workspace::with_projects(workspace.root(), workspace.config().clone(), vec![])?;
    assert!(empty.analyze()?.projects.is_empty());
    assert!(empty.export(tmp.path().join("empty")).is_err());
    assert!(!tmp.path().join("empty").exists());
    let mut config = workspace.config().clone();
    config.inputs.directory = "not-submitted/{id}".into();
    let missing = Workspace::new(workspace.root(), config)?;
    let analysis = missing.analyze()?;
    assert!(!analysis.is_complete() && analysis.projects.is_empty());
    assert!(analysis.overall.iter().all(Option::is_none));
    assert!(missing.export(tmp.path().join("missing")).is_err());
    assert!(!tmp.path().join("missing").exists());
    Ok(())
}

#[test]
fn document_rules_control_order_and_support_any_number_of_pdf_parts() -> Result<()> {
    let (tmp, workspace) = common::workspace("en")?;
    let mut config = pdf_only(&workspace)?.config().clone();
    config.inputs.directory = "submissions/{id}/reports".into();
    config.inputs.documents = vec![
        DocumentRule {
            label: "First".into(),
            pattern: "intro.pdf".into(),
            required: true,
        },
        DocumentRule {
            label: "Appendices".into(),
            pattern: "appendix-*.pdf".into(),
            required: false,
        },
    ];
    for p in workspace.projects() {
        let dir = workspace
            .root()
            .join(format!("submissions/{}/reports", p.id));
        fs::create_dir_all(&dir)?;
        fs::copy(
            workspace.root().join(&p.id).join("part-1.pdf"),
            dir.join("intro.pdf"),
        )?;
        for n in [10, 2, 1] {
            fs::copy(
                workspace.root().join("nova/part-2.pdf"),
                dir.join(format!("appendix-{n:02}.pdf")),
            )?;
        }
        fs::write(dir.join("notes.txt"), "ignored")?;
    }
    let workspace = Workspace::new(workspace.root(), config)?;
    let submissions = workspace.submissions()?;
    let filenames: Vec<_> = submissions[0]
        .reports
        .iter()
        .map(|p| {
            p.file_name()
                .expect("filename")
                .to_str()
                .expect("UTF-8 filename")
        })
        .collect();
    assert_eq!(
        filenames,
        [
            "intro.pdf",
            "appendix-01.pdf",
            "appendix-02.pdf",
            "appendix-10.pdf"
        ]
    );
    let analysis = workspace.export(tmp.path().join("ordered"))?;
    assert_eq!(analysis.projects[0].pages, 4);
    Ok(())
}

#[test]
fn ambiguous_patterns_and_unsafe_conventions_fail_instead_of_guessing() -> Result<()> {
    let (_tmp, workspace) = common::workspace("en")?;
    for pattern in [
        "../*.pdf",
        "sub/report.pdf",
        "**.pdf",
        "report?.pdf",
        "[ab].pdf",
        "report.docx",
        "C:\\report.pdf",
        "",
    ] {
        let mut config = workspace.config().clone();
        config.inputs.documents[0].pattern = pattern.into();
        assert!(config.validate().is_err(), "{pattern}");
    }
    for directory in [
        "../{id}",
        "{id}/{id}",
        "fixed",
        "{id}/{year}",
        "/{id}",
        "{id}/*",
    ] {
        let mut config = workspace.config().clone();
        config.inputs.directory = directory.into();
        assert!(config.validate().is_err(), "{directory}");
    }
    let mut config = workspace.config().clone();
    config.inputs.documents.push(DocumentRule {
        label: "Overlaps".into(),
        pattern: "part-1.pdf".into(),
        required: true,
    });
    assert!(
        Workspace::new(workspace.root(), config)?
            .submissions()
            .is_err()
    );
    let mut config = workspace.config().clone();
    config.inputs.documents[1].pattern = "*.xlsx".into();
    fs::copy(
        workspace.root().join("atlas/annex.xlsx"),
        workspace.root().join("atlas/copy.xlsx"),
    )?;
    assert!(Workspace::new(workspace.root(), config)?.analyze().is_err());
    Ok(())
}

#[test]
fn long_unicode_names_paginate_contents_and_page_numbers_match() -> Result<()> {
    let (tmp, workspace) = common::workspace("fr")?;
    let config = pdf_only(&workspace)?.config().clone();
    let mut roster = Vec::new();
    for i in 0..55 {
        let mut project = workspace.projects()[0].clone();
        project.id = format!("p{i:02}");
        project.label = format!(
            "Étude {i:02} — Énergie et matériaux : {}",
            "connaissances ".repeat(12)
        );
        let dir = workspace.root().join(&project.id);
        fs::create_dir(&dir)?;
        fs::copy(
            workspace.root().join("atlas/part-1.pdf"),
            dir.join("part-1.pdf"),
        )?;
        roster.push(project);
    }
    let workspace = Workspace::with_projects(workspace.root(), config, roster)?;
    let output = tmp.path().join("long");
    workspace.export(&output)?;
    let pdf = Document::load(output.join("report.pdf"))?;
    let front = pdf.get_pages().len() - 55;
    assert!(front > 1);
    let contents = pdf.extract_text(&(1..=front as u32).collect::<Vec<_>>())?;
    assert!(
        contents.contains("Étude 00") && contents.contains("Étude 54"),
        "{contents}"
    );
    // Each project starts on the page printed beside its first contents line.
    let lines: Vec<_> = contents.lines().collect();
    for i in 0..55 {
        let label = format!("Étude {i:02}");
        let row = lines
            .iter()
            .position(|line| line.contains(&label))
            .expect("contents entry");
        assert_eq!(
            lines[row + 1].trim(),
            (front + i + 1).to_string(),
            "{label}: {}",
            lines[row + 1]
        );
        assert!(
            pdf.extract_text(&[(front + i + 1) as u32])?
                .contains("ATLAS")
        );
    }
    Ok(())
}

#[test]
fn contents_font_errors_are_actionable_and_never_publish_partial_files() -> Result<()> {
    let (tmp, workspace) = common::workspace("en")?;
    let mut config = workspace.config().clone();
    config.title = "研究報告".into();
    let error = Workspace::new(workspace.root(), config)?
        .export(tmp.path().join("unsupported"))
        .expect_err("missing glyph");
    assert!(format!("{error:#}").contains("contents_font"));
    assert!(!tmp.path().join("unsupported").exists());
    fs::write(workspace.root().join("font.ttf"), "broken")?;
    let mut config = workspace.config().clone();
    config.contents_font = Some("font.ttf".into());
    assert!(
        Workspace::new(workspace.root(), config.clone())?
            .export(tmp.path().join("bad-font"))
            .is_err()
    );
    let bundled = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/Lato-Regular.ttf");
    fs::copy(bundled, workspace.root().join("font.ttf"))?;
    Workspace::new(workspace.root(), config)?.export(tmp.path().join("custom-font"))?;
    Ok(())
}

#[test]
fn progress_follows_actual_operations_and_omits_absent_kpi_work() -> Result<()> {
    use remain_core::progress::Progress;
    let (tmp, workspace) = common::workspace("en")?;
    let workspace = pdf_only(&workspace)?;
    fs::remove_file(workspace.root().join("atlas/part-1.pdf"))?;
    let mut events = Vec::new();
    workspace.export_with_progress(tmp.path().join("progress"), |progress| {
        events.push(progress)
    })?;
    assert_eq!(
        events,
        [
            Progress::Discovering,
            Progress::ReadingProject {
                current: 1,
                total: 2
            },
            Progress::ReadingProject {
                current: 2,
                total: 2
            },
            Progress::Summarizing,
            Progress::AssemblingPdfs,
            Progress::WritingSummary,
            Progress::Publishing,
        ]
    );
    Ok(())
}

#[cfg(unix)]
#[test]
fn roster_font_and_project_folder_symlinks_cannot_escape_the_workspace() -> Result<()> {
    use std::os::unix::fs::symlink;
    let (tmp, workspace) = common::workspace("en")?;
    let external_roster = tmp.path().join("projects.csv");
    fs::copy(workspace.root().join("projects.csv"), &external_roster)?;
    symlink(external_roster, workspace.root().join("external.csv"))?;
    let mut config = workspace.config().clone();
    config.projects = "external.csv".into();
    assert!(Workspace::new(workspace.root(), config).is_err());
    let external_font = tmp.path().join("font.ttf");
    fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/NotoSans-Regular.ttf"),
        &external_font,
    )?;
    symlink(external_font, workspace.root().join("font.ttf"))?;
    let mut config = workspace.config().clone();
    config.contents_font = Some("font.ttf".into());
    assert!(
        Workspace::new(workspace.root(), config)?
            .export(tmp.path().join("output"))
            .is_err()
    );
    fs::rename(workspace.root().join("nova"), tmp.path().join("nova"))?;
    symlink(tmp.path().join("nova"), workspace.root().join("nova"))?;
    assert!(workspace.submissions().is_err());
    Ok(())
}
