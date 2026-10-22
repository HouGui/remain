//! Optional and independent group/category membership, across every output.
mod common;
use anyhow::Result;
use calamine::{Data, Reader, Xlsx, open_workbook};
use lopdf::Document;
use remain_core::{
    Workspace,
    config::{Classification, Config},
};
use std::fs;

#[test]
fn completely_free_projects_work_without_classification_definitions() -> Result<()> {
    let (tmp, workspace) = common::workspace("en")?;
    let mut json = serde_json::to_value(workspace.config())?;
    let object = json.as_object_mut().expect("config");
    object.remove("groups");
    object.remove("categories");
    let mut roster = serde_json::to_value(workspace.projects())?;
    for project in roster.as_array_mut().expect("projects") {
        project.as_object_mut().expect("project").remove("group");
        project.as_object_mut().expect("project").remove("category");
    }
    let config: Config = serde_json::from_value(json)?;
    let projects = serde_json::from_value(roster)?;
    let output = tmp.path().join("free");
    let analysis = Workspace::with_projects(workspace.root(), config, projects)?.export(&output)?;
    assert_eq!(analysis.overall[0], Some(12.));
    for scopes in [&analysis.groups, &analysis.categories] {
        assert_eq!(scopes.len(), 1);
        assert_eq!(scopes[0].id, None);
        assert_eq!(scopes[0].projects, 3);
        assert_eq!(scopes[0].values, analysis.overall);
    }
    assert_eq!(
        Document::load(output.join("report.pdf"))?.get_pages().len(),
        5
    );
    assert_eq!(fs::read_dir(output.join("groups"))?.count(), 0);
    assert_eq!(fs::read_dir(output.join("categories"))?.count(), 0);
    let html = fs::read_to_string(output.join("index.html"))?;
    assert!(html.contains("Unassigned"));
    assert!(!html.contains("href=\"categories/"));
    let mut workbook: Xlsx<_> = open_workbook(output.join("annex.xlsx"))?;
    assert_eq!(
        workbook.worksheet_range("Projects")?.get_value((1, 2)),
        Some(&Data::Empty)
    );
    Ok(())
}

#[test]
fn categories_cross_group_boundaries_without_double_counting() -> Result<()> {
    let (tmp, workspace) = common::workspace("en")?;
    let mut config = workspace.config().clone();
    let mut projects = workspace.projects().to_vec();
    config.categories = vec![Classification {
        id: "series".into(),
        label: "Series".into(),
    }];
    // Every combination: group only, category only, and both.
    projects[0].category = None;
    projects[1].group = None;
    projects[1].category = Some("series".into());
    projects[2].category = Some("series".into());
    let output = tmp.path().join("mixed");
    let analysis = Workspace::with_projects(workspace.root(), config, projects)?.export(&output)?;
    assert_eq!(analysis.overall[0], Some(12.));
    assert_eq!(
        analysis.groups.last().expect("unassigned").values[0],
        Some(4.)
    );
    assert_eq!(analysis.categories[0].values[0], Some(7.));
    assert_eq!(analysis.categories[1].values[0], Some(5.));
    assert_eq!(
        Document::load(output.join("categories/series.pdf"))?
            .get_pages()
            .len(),
        4
    );
    let central = Document::load(output.join("report.pdf"))?;
    assert!(central.extract_text(&[2])?.contains("ATLAS"));
    assert!(central.extract_text(&[3])?.contains("ORBIT"));
    assert!(central.extract_text(&[4])?.contains("NOVA"));
    let mut workbook: Xlsx<_> = open_workbook(output.join("annex.xlsx"))?;
    assert_eq!(
        workbook.worksheet_range("Categories")?.get_value((1, 2)),
        Some(&Data::Float(7.))
    );
    Ok(())
}

#[test]
fn undefined_categories_and_duplicate_category_ids_are_rejected() -> Result<()> {
    let (_tmp, workspace) = common::workspace("en")?;
    let mut config = workspace.config().clone();
    let mut projects = workspace.projects().to_vec();
    projects[0].category = Some("unknown".into());
    assert!(config.validate_projects(&projects).is_err());
    projects[0].category = None;
    config.categories.push(config.categories[0].clone());
    assert!(config.validate().is_err());
    Ok(())
}
